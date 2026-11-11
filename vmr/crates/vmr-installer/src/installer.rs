//! Installer main dispatch (mirrors Go `installer.go` +
//! `install/{unarchiver,executable,conda,coursier}.go`).
//!
//! Disk contract: `<versions>/<sdk>_versions/<plugin>-<version>` + `<sdk>` symlink.
//! Modes: Globally (symlink + global env) / Sessionly / ToLock (writes .vmr.lock);
//! the latter two return `Action::RunSession`, letting the caller spawn a child shell
//! (PTY belongs to vmr-pty).

use std::path::{Path, PathBuf};

use vmr_core::paths;
use vmr_lua::types::{InstallerConfig, Item, installer_kind};
use vmr_utils::copy::copy_directory;
use vmr_utils::extract::extract;
use vmr_utils::find_dir::HomeDirFinder;
use vmr_utils::symlink::create_sym_link;

use crate::common::{install_dir, sdk_version_dir, symbol_link_path};
use crate::download::download_to_cache;
use crate::locker::VersionLocker;
use crate::pixi_env;
use crate::post::run_post_install;

pub const ADD_TO_PATH_TEMPORARILY_ENV: &str = "VMR_ADD_TO_PATH_TEMPORARILY";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InvokeMode {
    Globally,
    Sessionly,
    ToLock,
}

/// Result of an install action.
#[derive(Debug)]
pub enum Action {
    /// Done (global mode).
    Done,
    /// Requires a session child shell (session/lock modes; lock written as needed).
    RunSession,
}

/// A single install request.
pub struct InstallRequest {
    pub sdk_name: String,
    pub plugin_name: String,
    pub version_name: String,
    pub version: Item,
    pub ic: InstallerConfig,
    pub mode: InvokeMode,
}

impl InstallRequest {
    pub fn install_dir(&self) -> PathBuf {
        install_dir(&self.sdk_name, &self.plugin_name, &self.version_name)
    }

    pub fn symbol_path(&self) -> PathBuf {
        symbol_link_path(&self.sdk_name)
    }

    fn is_installed(&self) -> bool {
        std::fs::read_dir(self.install_dir())
            .map(|mut it| it.next().is_some())
            .unwrap_or(false)
    }

    /// Installs when not yet installed; returns whether it was newly installed.
    fn ensure_installed(&self) -> Result<bool, String> {
        if self.is_installed() {
            return Ok(false);
        }
        match self.version.installer.as_str() {
            installer_kind::CONDA | installer_kind::CONDA_FORGE => {
                self.install_via_pixi()?;
            }
            installer_kind::COURSIER => {
                self.install_via_coursier()?;
            }
            installer_kind::EXECUTABLE | installer_kind::DPKG | installer_kind::RPM => {
                self.install_via_executable()?;
            }
            _ => self.install_via_unarchiver()?,
        }
        run_post_install(&self.plugin_name, &self.install_dir(), &self.version)?;
        Ok(true)
    }

    fn install_via_unarchiver(&self) -> Result<(), String> {
        let cached = download_to_cache(&self.plugin_name, &self.version_name, &self.version)?;
        let temp = paths::temp_dir();
        // Clean up leftovers from the last run (only vmr-rust-owned subdirectories).
        let work = temp.join(format!("extract-{}", self.plugin_name.replace('.', "_")));
        let _ = std::fs::remove_dir_all(&work);
        std::fs::create_dir_all(&work).map_err(|e| e.to_string())?;

        let result = (|| -> Result<(), String> {
            extract(&cached, &work).map_err(|e| format!("extract failed: {e}"))?;
            patch_extracted_single_file(&work, &self.sdk_name);

            let mut finder = HomeDirFinder::new(flag_files_for(&self.ic, current_os()));
            finder.set_flag_dir_excepted(self.ic.flag_dir_excepted);
            finder.find(&work);
            let home = finder.get_dir_name().ok_or("can't find dir to copy")?;

            copy_directory(&home, &self.install_dir()).map_err(|e| format!("copy failed: {e}"))?;
            Ok(())
        })();
        let _ = std::fs::remove_dir_all(&work);
        result
    }

    fn install_via_executable(&self) -> Result<(), String> {
        let cached = download_to_cache(&self.plugin_name, &self.version_name, &self.version)?;
        let dir = self.install_dir();
        std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
        let name = cached.file_name().ok_or("bad file")?;
        let dest = dir.join(name);
        vmr_utils::copy::copy_a_file(&cached, &dest).map_err(|e| e.to_string())?;
        if !cfg!(windows) {
            chmod_x(&dest);
        }
        // BinaryRename: rename the executable file.
        if let Some(br) = &self.ic.binary_rename {
            if !br.name_flag.is_empty() {
                if let Ok(entries) = std::fs::read_dir(&dir) {
                    for e in entries.flatten() {
                        let p = e.path();
                        if e.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                            continue;
                        }
                        let fname = e.file_name().to_string_lossy().into_owned();
                        if fname.contains(&br.name_flag) {
                            let mut new_name = br.rename_to.clone();
                            if cfg!(windows) {
                                new_name.push_str(".exe");
                            }
                            let _ = std::fs::rename(p, dir.join(new_name));
                        }
                    }
                }
            }
        }
        Ok(())
    }

    fn install_via_coursier(&self) -> Result<(), String> {
        let cs = std::env::var("VMR_COURSIER_PATH").unwrap_or_else(|_| "cs".to_string());
        let version = self
            .version_name
            .trim_end_matches("-LTS")
            .trim_end_matches("-lts");
        let args: Vec<String> = vec![
            cs,
            "install".to_string(),
            "-q".to_string(),
            format!("--install-dir={}", self.install_dir().display()),
            format!("{}:{version}", self.plugin_name),
        ];
        run_cmd(&args, &paths::work_dir()).map_err(|e| format!("coursier failed: {e}"))
    }

    fn install_via_pixi(&self) -> Result<(), String> {
        // Pixi/rattler resolves the complete dependency closure and installs it into the VMR
        // version directory. No local conda/miniconda executable is involved.
        let records = vmr_pixi::resolve_packages(&self.plugin_name, &self.version_name)?;
        if records.is_empty() {
            return Err("package not found in the pixi channel".to_string());
        }
        vmr_pixi::install_packages(&records, &self.install_dir())
    }

    /// OS name of the current platform.
    fn collect_bundle(&self, base: &Path) -> vmr_env::EnvBundle {
        vmr_env::collect(&self.ic, base.to_str().unwrap_or(""), &current_os())
    }

    fn set_env_globally(&self) -> Result<(), String> {
        let shell = vmr_shell::Shell::detect();
        // Make sure the shell actually sources `~/.vmr/vmr.sh` (or vmr.fish). This is also
        // required here so direct `vmr use` calls set up the shell integration on their own.
        shell.write_vm_env_to_shell();
        let bundle = self.collect_bundle(&self.symbol_path());
        vmr_env::set_globally(&shell, &self.sdk_name, &bundle)
    }

    fn unset_env_bundle(&self, bundle: &vmr_env::EnvBundle) {
        let shell = vmr_shell::Shell::detect();
        vmr_env::unset_globally(&shell, &self.sdk_name, bundle);
    }

    fn add_envs_temporarily(&self) {
        let install = self.install_dir();
        let bundle = self.collect_bundle(&install);
        vmr_env::add_temporarily(&bundle);
    }
}

/// Session/lock modes: remove the SDK symlink prefix from the process PATH and prepare
/// temporary env (mirrors the non-global branch of Go `Install()`).
fn prepare_session_envs(req: &InstallRequest, to_lock: bool) {
    if to_lock {
        let mut locker = VersionLocker::default();
        locker.save(None, &req.plugin_name, &req.version_name);
    }
    remove_sdk_path_from_process(&req.sdk_name);
    unsafe { std::env::set_var(ADD_TO_PATH_TEMPORARILY_ENV, "1") };
    req.add_envs_temporarily();
}

/// Removes the SDK symlink path prefix from the process PATH
/// (mirrors Go `RemoveGlobalSDKPathTemporarily`).
pub fn remove_sdk_path_from_process(sdk_name: &str) {
    let sep = if cfg!(windows) { ';' } else { ':' };
    let symbolic = sdk_version_dir(sdk_name).join(sdk_name);
    let prefix = symbolic.to_string_lossy().into_owned();
    let prefix_cmp = if cfg!(windows) {
        prefix.to_lowercase()
    } else {
        prefix.clone()
    };
    let cur = std::env::var("PATH").unwrap_or_default();
    let parts: Vec<&str> = cur
        .split(sep)
        .filter(|p| {
            let matches = if cfg!(windows) {
                p.to_lowercase().starts_with(&prefix_cmp)
            } else {
                p.starts_with(&prefix)
            };
            !matches
        })
        .collect();
    unsafe { std::env::set_var("PATH", parts.join(&sep.to_string())) };
}

/// Downloads/extracts the version when missing and records the pixi backend; returns whether
/// it was newly installed. No symlink, global env or session side effects.
fn install_version(req: &InstallRequest) -> Result<bool, String> {
    if req.version.installer == installer_kind::COURSIER {
        // Prerequisite: the coursier binary must exist (Go behavior intentionally kept).
        ensure_command("cs").map_err(|_| "coursier is not installed".to_string())?;
    }
    let newly = req.ensure_installed()?;
    // conda/conda-forge packages are always installed through the pixi/rattler backend.
    // Record the backend before any env work: `conda-meta/` alone is not a marker because a
    // plugin-installed Miniconda prefix also has one. Marking an already-installed prefix
    // here also upgrades prefixes created before the marker existed.
    if matches!(
        req.version.installer.as_str(),
        installer_kind::CONDA | installer_kind::CONDA_FORGE
    ) {
        pixi_env::mark_prefix(&req.install_dir())?;
    }
    Ok(newly)
}

/// Installs the version when missing without activating it (no symlink, global env or
/// session). Used by the `cd` hook to satisfy `.vmr.lock` entries.
pub fn install_only(req: &InstallRequest) -> Result<(), String> {
    install_version(req).map(|_| ())
}

/// Main entry: install + mode handling.
pub fn install(req: &InstallRequest) -> Result<Action, String> {
    let newly = install_version(req)?;
    match req.mode {
        InvokeMode::Globally => {
            if !newly {
                eprintln!(
                    "{} {} is already installed.",
                    req.plugin_name, req.version_name
                );
            }
            // Rebuild the symlink + global env.
            let sym = req.symbol_path();
            remove_path(&paths::versions_dir(), &sym);
            if req.install_dir().exists() {
                create_sym_link(req.install_dir().to_str().unwrap(), sym.to_str().unwrap())
                    .map_err(|e| format!("create symlink failed: {e}"))?;
            }
            req.set_env_globally()?;
            req.add_envs_temporarily();
            Ok(Action::Done)
        }
        InvokeMode::Sessionly => {
            prepare_session_envs(req, false);
            Ok(Action::RunSession)
        }
        InvokeMode::ToLock => {
            prepare_session_envs(req, true);
            Ok(Action::RunSession)
        }
    }
}

/// Uninstall: removes the version directory; for the current version also removes the
/// symlink and the env entries. The env bundle is collected **before** the version dir is
/// removed, because env collection only keeps paths that still exist.
pub fn uninstall(req: &InstallRequest) -> Result<(), String> {
    let dir = req.install_dir();
    let sym = req.symbol_path();
    let is_current = std::fs::read_link(&sym)
        .map(|target| paths_equal(&target, &dir))
        .unwrap_or(false);
    let bundle = if is_current {
        Some(req.collect_bundle(&sym))
    } else {
        None
    };
    let root = paths::versions_dir();
    remove_path(&root, &dir);
    if is_current {
        remove_path(&root, &sym);
        if let Some(bundle) = bundle {
            req.unset_env_bundle(&bundle);
        }
    }
    Ok(())
}

/// Uninstalls every version of an SDK and removes its global env entries.
pub fn uninstall_all(sdk_name: &str, plugin_name: &str) {
    let sym = symbol_link_path(sdk_name);
    // Collect the bundle before deleting: `vmr_env::collect` drops directories that no
    // longer exist, and an empty bundle cannot remove the recorded `vmr.sh` lines. The
    // current version symlink is the installed prefix, so the same marker/plugin decision
    // as `vmr use` applies here.
    let plugin_ic = {
        let mut plugins = vmr_lua::Plugins::new();
        plugins
            .get_by_plugin_name(plugin_name)
            .and_then(|mut p| p.get_installer_config().ok())
    };
    let bundle = sym
        .is_dir()
        .then(|| pixi_env::collect_bundle(&sym, plugin_ic.as_ref()));
    let root = paths::versions_dir();
    remove_path(&root, &sdk_version_dir(sdk_name));
    remove_path(&root, &sym);
    if let Some(bundle) = bundle {
        let shell = vmr_shell::Shell::detect();
        vmr_env::unset_globally(&shell, sdk_name, &bundle);
    }
}

/// Compares two paths, tolerating case/format differences on Windows.
fn paths_equal(a: &Path, b: &Path) -> bool {
    if let (Ok(a), Ok(b)) = (std::fs::canonicalize(a), std::fs::canonicalize(b)) {
        #[cfg(windows)]
        {
            return a
                .to_string_lossy()
                .eq_ignore_ascii_case(&b.to_string_lossy());
        }
        #[cfg(not(windows))]
        {
            return a == b;
        }
    }
    #[cfg(windows)]
    {
        a.to_string_lossy()
            .eq_ignore_ascii_case(&b.to_string_lossy())
    }
    #[cfg(not(windows))]
    {
        a == b
    }
}

/// Reconstructs `path` only when it is a direct descendant of `root` with normal
/// components. This rejects absolute paths, `..`, and root prefixes that a plugin or
/// `VMR_SDK_INSTALLATION_DIR` could otherwise use to escape the versions directory.
fn scoped_path(root: &Path, path: &Path) -> Option<PathBuf> {
    let rel = path.strip_prefix(root).ok()?;
    if rel.as_os_str().is_empty() {
        return None;
    }
    let mut out = PathBuf::from(root);
    for component in rel.components() {
        match component {
            std::path::Component::Normal(part) => out.push(part),
            _ => return None,
        }
    }
    Some(out)
}

/// Removes a file, symlink or directory under `root` without following symlinks.
/// Paths outside `root` (or containing `..`) are ignored.
pub fn remove_path(root: &Path, path: &Path) {
    let Some(path) = scoped_path(root, path) else {
        return;
    };
    remove_path_raw(&path);
}

fn remove_path_raw(path: &Path) {
    let Ok(meta) = std::fs::symlink_metadata(path) else {
        return;
    };
    if meta.file_type().is_symlink() {
        // On Windows the version link is a directory junction (or a directory symlink).
        // `remove_file` rejects those with ERROR_ACCESS_DENIED, so delete the reparse
        // point as a directory; the link target itself is left untouched.
        #[cfg(windows)]
        {
            use std::os::windows::fs::FileTypeExt;
            if meta.file_type().is_symlink_dir() {
                let _ = std::fs::remove_dir(path);
            } else {
                let _ = std::fs::remove_file(path);
            }
        }
        #[cfg(not(windows))]
        {
            let _ = std::fs::remove_file(path);
        }
    } else if meta.is_file() {
        let _ = std::fs::remove_file(path);
    } else {
        let _ = std::fs::remove_dir_all(path);
    }
}

/// Resolves a `.vmr.lock` entry to its install prefix and optional plugin `InstallerConfig`.
///
/// A lock entry can come from:
/// - a pixi install (usually no plugin): `<sdk>_versions/<sdk>-<version>`, marker set;
/// - a plugin install: `<sdk>_versions/<plugin>-<version>`, plugin `InstallerConfig`.
fn locked_entry(
    plugins: &mut vmr_lua::Plugins,
    sdk: &str,
    version: &str,
) -> (PathBuf, Option<InstallerConfig>) {
    match plugins
        .get_by_sdk_name(sdk)
        .or_else(|| plugins.get_by_plugin_name(sdk))
    {
        Some(mut plugin) => {
            let prefix = install_dir(&plugin.sdk_name, &plugin.plugin_name, version);
            let ic = plugin.get_installer_config().ok();
            (prefix, ic)
        }
        None => (install_dir(sdk, sdk, version), None),
    }
}

/// `.vmr.lock` entries whose install directory is absent, sorted by SDK name.
///
/// `vmr cdhook` uses this to offer installing the pinned versions before injecting the
/// project environment. Entries keep their lock-file SDK name (which may be an alias).
pub fn missing_locked_versions() -> Vec<(String, String)> {
    let locker = VersionLocker::load_from(None);
    if locker.versions.is_empty() {
        return Vec::new();
    }
    let mut plugins = vmr_lua::Plugins::new();
    let mut missing = Vec::new();
    for (sdk, version) in &locker.versions {
        let (prefix, _) = locked_entry(&mut plugins, sdk, version);
        if !prefix.is_dir() {
            missing.push((sdk.clone(), version.clone()));
        }
    }
    missing.sort();
    missing
}

/// `vmr cdhook`: injects env per the lock and returns RunSession (mirrors Go HookForCdCommand).
pub fn hook_for_cd_command() -> Result<Action, String> {
    let locker = VersionLocker::load_from(None);
    if locker.versions.is_empty() {
        return Ok(Action::Done);
    }
    unsafe { std::env::set_var(ADD_TO_PATH_TEMPORARILY_ENV, "1") };
    let mut plugins = vmr_lua::Plugins::new();
    for (sdk, version) in &locker.versions {
        remove_sdk_path_from_process(sdk);
        let (prefix, plugin_ic) = locked_entry(&mut plugins, sdk, version);
        if !prefix.is_dir() {
            continue;
        }
        let bundle = pixi_env::collect_bundle(&prefix, plugin_ic.as_ref());
        vmr_env::add_temporarily(&bundle);
    }
    Ok(Action::RunSession)
}

// ---- Internal utilities ----

/// Renames a single extracted executable to the SDK name (mirrors Go `ArchiverInstaller.patchFileName`).
fn patch_extracted_single_file(work: &Path, sdk_name: &str) {
    let Ok(entries) = std::fs::read_dir(work) else {
        return;
    };
    let files: Vec<_> = entries
        .flatten()
        .filter(|e| e.file_type().map(|t| !t.is_dir()).unwrap_or(false))
        .collect();
    if files.len() != 1 {
        return;
    }
    let file = &files[0];
    let name = file.file_name().to_string_lossy().into_owned();
    let new_name = if cfg!(windows) {
        format!("{sdk_name}.exe")
    } else {
        sdk_name.to_string()
    };
    if name.contains(sdk_name) && name != new_name {
        let _ = std::fs::rename(file.path(), work.join(&new_name));
        chmod_x(&work.join(new_name));
    } else if !cfg!(windows) && name == new_name {
        chmod_x(&file.path());
    }
}

fn current_os() -> String {
    if cfg!(windows) {
        "windows".to_string()
    } else if cfg!(target_os = "macos") {
        "darwin".to_string()
    } else {
        "linux".to_string()
    }
}

fn flag_files_for(ic: &InstallerConfig, os: String) -> Vec<String> {
    let Some(ff) = &ic.flag_files else {
        return Vec::new();
    };
    let v = match os.as_str() {
        "windows" => &ff.windows,
        "darwin" => &ff.darwin,
        _ => &ff.linux,
    };
    v.clone()
}

#[cfg(unix)]
fn chmod_x(path: &std::path::Path) {
    use std::os::unix::fs::PermissionsExt;
    if let Ok(meta) = std::fs::metadata(path) {
        let mode = meta.permissions().mode();
        let _ = std::fs::set_permissions(path, std::fs::Permissions::from_mode(mode | 0o111));
    }
}

#[cfg(not(unix))]
fn chmod_x(_: &std::path::Path) {}

fn run_cmd(args: &[String], cwd: &std::path::Path) -> Result<(), String> {
    use std::io::IsTerminal;
    let mut cmd = std::process::Command::new(&args[0]);
    cmd.args(&args[1..]);
    cmd.current_dir(cwd);
    // Post-install scripts must not read the MCP JSON-RPC stream (or a piped stdin they were
    // not meant to consume); they only get stdin when a terminal is attached.
    if std::io::stdin().is_terminal() {
        cmd.stdin(std::process::Stdio::inherit());
    } else {
        cmd.stdin(std::process::Stdio::null());
    }
    // Under `vmr mcp`, a child inheriting stdout would corrupt the protocol stream.
    if std::env::var_os("VMR_MCP_SERVER").is_some() {
        cmd.stdout(std::process::Stdio::null());
    } else {
        cmd.stdout(std::process::Stdio::inherit());
    }
    cmd.stderr(std::process::Stdio::inherit());
    let status = cmd.status().map_err(|e| e.to_string())?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("command failed: {status}"))
    }
}

fn ensure_command(name: &str) -> Result<(), String> {
    let probe = std::process::Command::new(name)
        .arg("--help")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status();
    match probe {
        Ok(_) => Ok(()),
        Err(e) => Err(e.to_string()),
    }
}

#[cfg(test)]
fn test_dir(name: &str) -> PathBuf {
    // Compile-time base: CodeQL treats runtime temp_dir() as user input, which would make
    // the test-only filesystem calls in this file look like production path injection.
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target")
        .join("remove-path-tests")
        .join(format!("{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&p);
    std::fs::create_dir_all(&p).unwrap();
    p
}

#[cfg(all(test, windows))]
mod tests {
    use super::remove_path;
    use super::test_dir;
    use std::fs;
    use std::process::Command;

    #[test]
    fn remove_path_removes_junction_only() {
        let base = test_dir("junction");
        let target = base.join("target");
        let link = base.join("link");
        fs::create_dir_all(&target).unwrap();
        fs::write(target.join("keep.txt"), "keep").unwrap();
        let status = Command::new("cmd")
            .args([
                "/c",
                "mklink",
                "/j",
                link.to_str().unwrap(),
                target.to_str().unwrap(),
            ])
            .status()
            .unwrap();
        assert!(status.success());

        remove_path(&base, &link);
        assert!(!link.exists(), "junction should be removed");
        assert_eq!(
            fs::read_to_string(target.join("keep.txt")).unwrap(),
            "keep",
            "link target must not be deleted"
        );
        let _ = fs::remove_dir_all(&base);
    }
}

#[cfg(test)]
mod remove_path_tests {
    use super::remove_path;
    use super::test_dir;
    use std::fs;

    #[test]
    fn removes_a_descendant_inside_root() {
        let base = test_dir("inside");
        let root = base.join("versions");
        let child = root.join("go_versions/go-1.22");
        fs::create_dir_all(&child).unwrap();
        fs::write(child.join("file"), "x").unwrap();

        remove_path(&root, &child);
        assert!(!child.exists());
        let _ = fs::remove_dir_all(&base);
    }

    #[test]
    fn rejects_outside_paths_and_parent_traversal() {
        let base = test_dir("outside");
        let root = base.join("versions");
        fs::create_dir_all(&root).unwrap();
        let outside = base.join("outside.txt");
        fs::write(&outside, "keep").unwrap();

        // Not under root at all.
        remove_path(&root, &outside);
        assert!(outside.exists());

        // Lexically under root, but escapes via `..`.
        remove_path(&root, &root.join("..").join("outside.txt"));
        assert!(outside.exists());

        let _ = fs::remove_dir_all(&base);
    }
}
