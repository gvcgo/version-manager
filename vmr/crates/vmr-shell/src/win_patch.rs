//! Windows mingw/git-bash compatibility layer (mirrors Go `internal/shell/win_patch.go`).
//!
//! Windows has no `~/.vmr/vmr.sh`: the global environment lives in `HKCU\Environment`. This
//! module keeps the git-bash profile (`%USERPROFILE%\.bashrc`) in sync with it: the cd hook
//! is (re)written there, and legacy per-SDK `export PATH="${PATH}:<mingw path>"` lines are
//! removed. Those lines were written by older builds (whose registry entries used the
//! `%VMR_VERSIONS%\...` form); the shim directory (`~/.vmr/bin`) replaced them, so nothing is
//! mirrored any more.

use std::fs;
use std::path::PathBuf;

use vmr_core::paths;

use crate::convert_to_mingw;
use crate::scripts;
use crate::win_reg;

const VMR_VERSIONS_ENV: &str = "VMR_VERSIONS";
const VMR_VERSIONS_PREFIX: &str = "%VMR_VERSIONS%";

fn user_home() -> PathBuf {
    std::env::var("USERPROFILE")
        .or_else(|_| std::env::var("HOME"))
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("."))
}

fn bashrc_path() -> PathBuf {
    user_home().join(".bashrc")
}

fn versions_dir() -> String {
    paths::versions_dir().to_string_lossy().into_owned()
}

fn read_bashrc() -> String {
    fs::read_to_string(bashrc_path()).unwrap_or_default()
}

fn write_bashrc(content: &str) {
    let _ = fs::write(bashrc_path(), content);
}

/// The mingw `export PATH` line for a versions-related path, whatever form it is in
/// (absolute, `%VMR_VERSIONS%`, or already mingw). `None` for unrelated paths.
fn export_line_for(path_env: &str) -> Option<String> {
    let versions = versions_dir();
    let mingw_versions = convert_to_mingw(&versions);
    if let Some(rest) = path_env.strip_prefix(VMR_VERSIONS_PREFIX) {
        let p = format!("{versions}{rest}");
        Some(format!(
            "export PATH=\"${{PATH}}:{}\"",
            convert_to_mingw(&p)
        ))
    } else if path_env.starts_with(&versions) {
        Some(format!(
            "export PATH=\"${{PATH}}:{}\"",
            convert_to_mingw(path_env)
        ))
    } else if path_env.starts_with(&mingw_versions) {
        Some(format!("export PATH=\"${{PATH}}:{path_env}\""))
    } else {
        None
    }
}

/// Removes `line` (and exact duplicates) from the rc text, preserving the final newline.
fn remove_export_line(old: &str, line: &str) -> String {
    let mut out = old
        .lines()
        .filter(|l| l.trim() != line.trim())
        .collect::<Vec<_>>()
        .join("\n");
    if !out.is_empty() && old.ends_with('\n') {
        out.push('\n');
    }
    out
}

fn remove_one_path_env(path_env: &str) {
    let Some(line) = export_line_for(path_env) else {
        return;
    };
    let content = read_bashrc();
    write_bashrc(&remove_export_line(&content, &line));
}

fn mingw_cd_hook(install: &str) -> String {
    let cache = crate::script_path(
        crate::Kind::Bash,
        &paths::cd_cache_file().to_string_lossy(),
        &crate::dirs_home(),
    );
    let exit = crate::script_path(
        crate::Kind::Bash,
        &paths::session_exit_file().to_string_lossy(),
        &crate::dirs_home(),
    );
    let session = crate::fill(scripts::SESSION_HOOK_SH, &[("__VMR_SESSION_EXIT__", &exit)]);
    crate::fill(
        scripts::HOOK_GITBASH,
        &[
            ("__VMR_INSTALL__", install),
            ("__VMR_CD_CACHE__", &cache),
            ("__VMR_SESSION_EXIT__", &exit),
            ("__VMR_SESSION_HOOK__", &session),
        ],
    )
}

fn patch_cd_hook() {
    let content = read_bashrc();
    let hook = mingw_cd_hook(&convert_to_mingw(&paths::work_dir().to_string_lossy()));
    // Replace an existing marked hook (Go-era hooks call the removed `vmr use -E`), else append.
    let out = match (
        content.find("# cd hook start"),
        content.find("# cd hook end"),
    ) {
        (Some(start), Some(end)) if end > start => {
            let end = end + "# cd hook end".len();
            format!("{}{}{}", &content[..start], hook, &content[end..])
        }
        _ if content.is_empty() => hook,
        _ => format!("{content}\n{hook}"),
    };
    write_bashrc(&out);
}

fn register_versions_env() {
    let versions = versions_dir();
    if win_reg::get_env(VMR_VERSIONS_ENV).as_deref() != Some(versions.as_str()) {
        win_reg::set_env(VMR_VERSIONS_ENV, &versions);
    }
}

/// Converts a versions-related path to the `%VMR_VERSIONS%\...` registry form.
fn registry_form(path: &str) -> String {
    let versions = versions_dir();
    if path.contains(&versions) {
        path.replace(&versions, VMR_VERSIONS_PREFIX)
    } else {
        path.to_string()
    }
}

/// Expands a `%VMR_VERSIONS%\...` entry to an absolute path (for registry comparisons).
pub(crate) fn expand_versions(path: &str) -> String {
    match path.strip_prefix(VMR_VERSIONS_PREFIX) {
        Some(rest) => format!("{}{rest}", versions_dir()),
        None => path.to_string(),
    }
}

/// Prepares a path for `SetPath`: registers `VMR_VERSIONS`, keeps the git-bash cd hook in
/// place, and returns the `%VMR_VERSIONS%\...` value for the registry.
///
/// Per-SDK PATH entries are no longer mirrored into `~/.bashrc`: the shim directory is the
/// only vmr PATH entry, and the old append-only `export PATH="${PATH}:..."` lines both
/// duplicated entries on every source and are cleaned up by [`clear_version_exports`].
pub fn tidy_path(path: &str) -> String {
    register_versions_env();
    patch_cd_hook();
    registry_form(path)
}

/// Drops every legacy `export PATH` line that points into the versions directory. Older
/// vmr builds appended one such line per SDK binary directory.
pub(crate) fn clear_version_exports() {
    let content = read_bashrc();
    if content.is_empty() {
        return;
    }
    let versions = versions_dir();
    let mingw_versions = convert_to_mingw(&versions);
    let out: Vec<&str> = content
        .lines()
        .filter(|line| {
            let line = line.trim();
            if !line.starts_with("export PATH=") {
                return true;
            }
            !(line.contains(&versions)
                || line.contains(&mingw_versions)
                || line.contains(VMR_VERSIONS_PREFIX))
        })
        .collect();
    let mut joined = out.join("\n");
    if content.ends_with('\n') && !joined.is_empty() {
        joined.push('\n');
    }
    if joined != content {
        write_bashrc(&joined);
    }
}

/// Prepares a path for `UnsetPath`: remove its mingw `export PATH` line instead of adding
/// one, then return the `%VMR_VERSIONS%\...` registry form to delete.
pub fn untidy_path(path: &str) -> String {
    remove_one_path_env(path);
    registry_form(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn removes_only_the_matching_export_line() {
        let line = "export PATH=\"${PATH}:/c/Users/me/.vmr/versions/go/go/bin\"";
        let old = format!("export A=1\n{line}\nexport B=2\n");
        let out = remove_export_line(&old, line);
        assert_eq!(out, "export A=1\nexport B=2\n");
        // Absent line: unchanged.
        assert_eq!(remove_export_line(&out, line), out);
    }
}
