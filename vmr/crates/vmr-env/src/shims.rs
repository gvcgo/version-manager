//! Windows shim directory: `PATH` keeps a single entry (`~/.vmr/bin`) and every command of
//! an active SDK is a hardlink to the running `vmr.exe` (or a generated batch wrapper).
//!
//! The registry used to receive one entry per SDK binary directory, so a few hundred pixi
//! packages could push the user `PATH` past the 32,767-character environment-variable limit.
//! Shims keep the registry value constant: the shim re-creates the SDK's `PATH` and extra
//! variables for the child process only.
//!
//! Dispatch: `main` calls [`dispatch`] before clap. A hardlink launched under a command name
//! (`python.exe`, `go.exe`, ...) resolves the mapping in `~/.vmr/shims.json` and runs the real
//! executable with the prefix's directories prepended to `PATH`. `.cmd`/`.bat` targets are
//! generated as batch files instead (a hardlink to a PE would not be executed by `cmd`).

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::UNIX_EPOCH;

use serde::{Deserialize, Serialize};

use crate::EnvBundle;
use vmr_core::paths;

/// Directory added to `PATH`; every active command lives here.
pub const SHIM_DIR_NAME: &str = "bin";
const STATE_FILE: &str = "shims.json";
/// One real copy of `vmr.exe` inside the shim directory, used when hardlinks across volumes
/// are impossible (copying the executable once instead of once per command).
const HOST_COPY: &str = ".vmr-shim-host.exe";
const PROBE: &str = ".vmr-link-probe.exe";
/// A shim may never shadow the manager itself.
const RESERVED: [&str; 1] = ["vmr"];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
enum Kind {
    /// `.exe` / `.com`: hardlink to the shim host, dispatched by [`dispatch`].
    Exe,
    /// `.cmd` / `.bat`: generated batch file that injects the environment itself.
    Batch,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ShimEntry {
    sdk: String,
    /// File name inside the shim directory (`python.exe`, `npm.cmd`, ...).
    name: String,
    /// Absolute target inside the SDK's version junction.
    target: String,
    /// Prefix-relative directories that must be on the child's `PATH`.
    #[serde(default)]
    dirs: Vec<String>,
    #[serde(default)]
    env: BTreeMap<String, String>,
    kind: Kind,
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct State {
    /// Shim host identity; all `Exe` shims are relinked when it changes (a vmr upgrade or a
    /// switched debug/release build would otherwise keep the old inode).
    #[serde(default)]
    host: String,
    #[serde(default)]
    host_len: u64,
    #[serde(default)]
    host_mtime: u64,
    #[serde(default)]
    commands: BTreeMap<String, ShimEntry>,
}

struct Candidate {
    name: String,
    target: String,
    kind: Kind,
}

/// `~/.vmr/bin` (created on demand by [`sync`]).
pub fn shim_dir() -> PathBuf {
    paths::work_dir().join(SHIM_DIR_NAME)
}

fn state_path() -> PathBuf {
    paths::work_dir().join(STATE_FILE)
}

/// Synchronizes the shims of one SDK: removes its previous commands, then exposes every
/// executable found in `bundle.path_dirs`. A name provided by another SDK is taken over
/// (last `vmr use` wins, mirroring the old prepend order of the registry entries).
pub fn sync(sdk: &str, bundle: &EnvBundle) -> Result<(), String> {
    sync_at(&shim_dir(), &state_path(), sdk, bundle)
}

/// Removes every shim owned by `sdk` (uninstall / uninstall-all).
pub fn remove(sdk: &str) {
    remove_at(&shim_dir(), &state_path(), sdk);
}

/// Entry point for `vmr.exe` hardlinks: returns `None` when the process was started as
/// `vmr` itself, otherwise the child exit code.
pub fn dispatch() -> Option<i32> {
    let stem = invocation_name()?;
    if stem.starts_with(".vmr-shim-host") {
        return None;
    }
    let state = load(&state_path());
    let Some(entry) = state.commands.get(&stem) else {
        eprintln!("vmr: unknown shim command: {stem}");
        return Some(127);
    };
    // `Batch` entries are generated batch files; `cmd` runs them without reaching this code.
    let mut cmd = Command::new(&entry.target);
    cmd.args(std::env::args_os().skip(1));
    if !entry.dirs.is_empty() {
        let mut path = entry.dirs.join(";");
        if let Ok(cur) = std::env::var("PATH") {
            if !cur.is_empty() {
                path.push(';');
                path.push_str(&cur);
            }
        }
        cmd.env("PATH", path);
    }
    for (key, value) in &entry.env {
        cmd.env(key, value);
    }
    match cmd.status() {
        Ok(status) => Some(status.code().unwrap_or(1)),
        Err(err) => {
            eprintln!("vmr: failed to run {}: {err}", entry.target);
            Some(127)
        }
    }
}

/// Command name the shim was started under. `argv[0]` is authoritative for hardlinks; the
/// executable path is the fallback. `None` when this really is `vmr` itself.
fn invocation_name() -> Option<String> {
    if let Some(arg0) = std::env::args_os().next() {
        if let Some(stem) = Path::new(&arg0).file_stem() {
            let stem = stem.to_string_lossy().to_lowercase();
            if stem != "vmr" && !stem.is_empty() {
                return Some(stem);
            }
        }
    }
    let exe = std::env::current_exe().ok()?;
    let stem = exe.file_stem()?.to_string_lossy().to_lowercase();
    if stem == "vmr" { None } else { Some(stem) }
}

fn sync_at(dir: &Path, state_file: &Path, sdk: &str, bundle: &EnvBundle) -> Result<(), String> {
    fs::create_dir_all(dir).map_err(|e| format!("create {} failed: {e}", dir.display()))?;
    let exe = std::env::current_exe().map_err(|e| format!("current_exe failed: {e}"))?;
    let (host_len, host_mtime) = identity(&exe)?;
    let mut state = load(state_file);
    let host_changed = state.host != exe.to_string_lossy()
        || state.host_len != host_len
        || state.host_mtime != host_mtime;
    let link_src = link_source(&exe, dir)?;

    let candidates = collect_commands(&bundle.path_dirs);
    let dirs = bundle.path_dirs.clone();
    let env: BTreeMap<String, String> = bundle.env_vars.iter().cloned().collect();

    // Drop this SDK's old commands and any command this sync takes over.
    for (key, entry) in state.commands.clone() {
        if entry.sdk == sdk || candidates.contains_key(&key) {
            remove_file(dir, &entry.name);
        }
    }
    state
        .commands
        .retain(|key, entry| entry.sdk != sdk && !candidates.contains_key(key));

    let mut added = Vec::with_capacity(candidates.len());
    for (key, candidate) in candidates {
        added.push((
            key,
            ShimEntry {
                sdk: sdk.to_string(),
                name: candidate.name,
                target: candidate.target,
                dirs: dirs.clone(),
                env: env.clone(),
                kind: candidate.kind,
            },
        ));
    }
    for (key, entry) in &added {
        state.commands.insert(key.clone(), entry.clone());
    }

    if host_changed {
        // Relink everything: a stale hardlink would keep running the previous binary.
        for entry in state.commands.values() {
            write_shim(dir, &link_src, entry)?;
        }
    } else {
        for entry in state.commands.values().filter(|e| e.sdk == sdk) {
            write_shim(dir, &link_src, entry)?;
        }
    }

    state.host = exe.to_string_lossy().into_owned();
    state.host_len = host_len;
    state.host_mtime = host_mtime;
    save(state_file, &state)
}

fn remove_at(dir: &Path, state_file: &Path, sdk: &str) {
    let mut state = load(state_file);
    for (_, entry) in state.commands.clone() {
        if entry.sdk == sdk {
            remove_file(dir, &entry.name);
        }
    }
    state.commands.retain(|_, entry| entry.sdk != sdk);
    let _ = save(state_file, &state);
}

/// Executables offered by `dirs`, first directory wins for duplicate names.
fn collect_commands(dirs: &[String]) -> BTreeMap<String, Candidate> {
    let mut out = BTreeMap::new();
    for dir in dirs {
        let Ok(entries) = fs::read_dir(dir) else {
            continue;
        };
        let mut entries: Vec<_> = entries.flatten().collect();
        entries.sort_by_key(|e| e.file_name());
        for entry in entries {
            if !entry.file_type().map(|t| t.is_file()).unwrap_or(false) {
                continue;
            }
            let name = entry.file_name().to_string_lossy().into_owned();
            let lower = name.to_lowercase();
            let kind = if lower.ends_with(".exe") || lower.ends_with(".com") {
                Kind::Exe
            } else if lower.ends_with(".cmd") || lower.ends_with(".bat") {
                Kind::Batch
            } else {
                continue;
            };
            let Some((stem, _)) = name.rsplit_once('.') else {
                continue;
            };
            if stem.is_empty() {
                continue;
            }
            let key = stem.to_lowercase();
            if RESERVED.contains(&key.as_str()) || out.contains_key(&key) {
                continue;
            }
            out.insert(
                key,
                Candidate {
                    name,
                    target: entry.path().to_string_lossy().into_owned(),
                    kind,
                },
            );
        }
    }
    out
}

fn write_shim(dir: &Path, link_src: &Path, entry: &ShimEntry) -> Result<(), String> {
    let dst = dir.join(&entry.name);
    let _ = fs::remove_file(&dst);
    let result = match entry.kind {
        Kind::Exe => link_or_copy(link_src, &dst),
        Kind::Batch => fs::write(&dst, batch_script(entry))
            .map_err(|e| format!("write {} failed: {e}", dst.display())),
    };
    match result {
        // The old file is locked by a running process (Windows refuses to delete a running
        // executable). It still dispatches through the same mapping, so keep it and move on.
        Err(err) if dst.exists() => {
            eprintln!("vmr: keeping locked shim {}: {err}", dst.display());
            Ok(())
        }
        other => other,
    }
}

fn remove_file(dir: &Path, name: &str) {
    let _ = fs::remove_file(dir.join(name));
}

fn link_or_copy(src: &Path, dst: &Path) -> Result<(), String> {
    if fs::hard_link(src, dst).is_ok() {
        return Ok(());
    }
    fs::copy(src, dst)
        .map(|_| ())
        .map_err(|e| format!("link/copy {} failed: {e}", dst.display()))
}

/// Hardlinks require one volume; fall back to a single in-tree copy of the shim host.
fn link_source(exe: &Path, dir: &Path) -> Result<PathBuf, String> {
    let probe = dir.join(PROBE);
    let _ = fs::remove_file(&probe);
    if fs::hard_link(exe, &probe).is_ok() {
        let _ = fs::remove_file(&probe);
        return Ok(exe.to_path_buf());
    }
    let _ = fs::remove_file(&probe);
    let copy = dir.join(HOST_COPY);
    let same = match (fs::metadata(exe), fs::metadata(&copy)) {
        (Ok(a), Ok(b)) => a.len() == b.len() && mtime(&a) == mtime(&b),
        _ => false,
    };
    if !same {
        fs::copy(exe, &copy).map_err(|e| format!("copy shim host failed: {e}"))?;
    }
    Ok(copy)
}

fn identity(path: &Path) -> Result<(u64, u64), String> {
    let meta = fs::metadata(path).map_err(|e| format!("stat {} failed: {e}", path.display()))?;
    Ok((meta.len(), mtime(&meta)))
}

fn mtime(meta: &fs::Metadata) -> u64 {
    meta.modified()
        .ok()
        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

fn batch_script(entry: &ShimEntry) -> String {
    let esc = |s: &str| s.replace('%', "%%");
    let mut out = String::from("@echo off\r\nsetlocal\r\n");
    if !entry.dirs.is_empty() {
        out.push_str(&format!(
            "set \"PATH={};%PATH%\"\r\n",
            esc(&entry.dirs.join(";"))
        ));
    }
    for (key, value) in &entry.env {
        out.push_str(&format!("set \"{key}={}\"\r\n", esc(value)));
    }
    out.push_str(&format!(
        "call \"{}\" %*\r\nexit /b %errorlevel%\r\n",
        esc(&entry.target)
    ));
    out
}

fn load(path: &Path) -> State {
    fs::read_to_string(path)
        .ok()
        .and_then(|data| serde_json::from_str(&data).ok())
        .unwrap_or_default()
}

fn save(path: &Path, state: &State) -> Result<(), String> {
    let data =
        serde_json::to_string_pretty(state).map_err(|e| format!("serialize shims failed: {e}"))?;
    fs::write(path, data).map_err(|e| format!("write {} failed: {e}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("vmr-shims-{}-{name}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn bundle(dirs: Vec<PathBuf>, env: Vec<(&str, &str)>) -> EnvBundle {
        EnvBundle {
            path_dirs: dirs
                .iter()
                .map(|d| d.to_string_lossy().into_owned())
                .collect(),
            env_vars: env
                .into_iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
        }
    }

    #[test]
    fn collect_skips_non_executables_reserved_and_duplicates() {
        let a = test_dir("a");
        let b = test_dir("b");
        fs::write(a.join("foo.exe"), b"x").unwrap();
        fs::write(a.join("bar.cmd"), b"x").unwrap();
        fs::write(a.join("notes.txt"), b"x").unwrap();
        fs::write(a.join("vmr.exe"), b"x").unwrap();
        fs::write(b.join("foo.exe"), b"y").unwrap();

        let found = collect_commands(&[
            a.to_string_lossy().into_owned(),
            b.to_string_lossy().into_owned(),
        ]);
        assert_eq!(found.len(), 2);
        assert_eq!(
            found["foo"].target,
            a.join("foo.exe").to_string_lossy().into_owned()
        );
        assert_eq!(found["bar"].kind, Kind::Batch);
        assert!(!found.contains_key("vmr"));
        let _ = fs::remove_dir_all(&a);
        let _ = fs::remove_dir_all(&b);
    }

    #[test]
    fn batch_wrapper_injects_prefix_path_and_env() {
        let entry = ShimEntry {
            sdk: "node".into(),
            name: "npm.cmd".into(),
            target: r"C:\v\npm.cmd".into(),
            dirs: vec![r"C:\v\node".into(), r"C:\v\Scripts".into()],
            env: [("GOROOT".to_string(), r"C:\go".to_string())]
                .into_iter()
                .collect(),
            kind: Kind::Batch,
        };
        let script = batch_script(&entry);
        assert!(script.contains(r#"set "PATH=C:\v\node;C:\v\Scripts;%PATH%""#));
        assert!(script.contains(r#"set "GOROOT=C:\go""#));
        assert!(script.contains(r#"call "C:\v\npm.cmd" %*"#));
        assert!(script.contains("exit /b %errorlevel%"));
    }

    #[test]
    fn sync_takes_over_names_and_remove_clears_them() {
        let root = test_dir("sync");
        let shims = root.join("bin");
        let state = root.join("shims.json");
        let a = root.join("a");
        let b = root.join("b");
        fs::create_dir_all(&a).unwrap();
        fs::create_dir_all(&b).unwrap();
        fs::write(a.join("tool.exe"), b"a").unwrap();
        fs::write(b.join("tool.exe"), b"b").unwrap();

        sync_at(&shims, &state, "a", &bundle(vec![a.clone()], vec![])).unwrap();
        assert!(shims.join("tool.exe").exists());
        assert_eq!(load(&state).commands["tool"].sdk, "a");

        sync_at(&shims, &state, "b", &bundle(vec![b.clone()], vec![])).unwrap();
        assert_eq!(load(&state).commands["tool"].sdk, "b");

        remove_at(&shims, &state, "b");
        assert!(load(&state).commands.is_empty());
        assert!(!shims.join("tool.exe").exists());
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn sync_removes_commands_dropped_by_a_new_version() {
        let root = test_dir("dropped");
        let shims = root.join("bin");
        let state = root.join("shims.json");
        let sdk = root.join("sdk");
        fs::create_dir_all(&sdk).unwrap();
        fs::write(sdk.join("old.exe"), b"x").unwrap();
        sync_at(&shims, &state, "s", &bundle(vec![sdk.clone()], vec![])).unwrap();
        assert!(shims.join("old.exe").exists());

        fs::remove_file(sdk.join("old.exe")).unwrap();
        fs::write(sdk.join("new.exe"), b"x").unwrap();
        sync_at(&shims, &state, "s", &bundle(vec![sdk.clone()], vec![])).unwrap();
        assert!(!shims.join("old.exe").exists());
        assert!(shims.join("new.exe").exists());
        let _ = fs::remove_dir_all(&root);
    }
}
