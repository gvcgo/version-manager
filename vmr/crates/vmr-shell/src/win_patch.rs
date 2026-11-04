//! Windows mingw/git-bash compatibility layer (mirrors Go `internal/shell/win_patch.go`).
//!
//! Registry `PATH` entries under the versions directory are stored as
//! `%VMR_VERSIONS%\...` (the variable is registered in `HKCU\Environment`) and mirrored into
//! `%USERPROFILE%\.bashrc` as `export PATH="${PATH}:<mingw path>"` lines, because git-bash
//! does not expand `%VMR_VERSIONS%`. The same profile also gets the cd hook, since Windows
//! uses the registry instead of `~/.vmr/vmr.sh`.

use std::fs;
use std::path::PathBuf;

use vmr_core::paths;

use crate::convert_to_mingw;
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

/// Appends `line` unless it is already present.
fn add_export_line(old: &str, line: &str) -> String {
    if old.contains(line) {
        old.to_string()
    } else if old.is_empty() {
        line.to_string()
    } else {
        format!("{old}\n{line}")
    }
}

/// Adds the mingw `export PATH` line for a versions-related path (Go `AddPathEnv`).
fn add_path_env(path_env: &str, old: &str) -> String {
    let versions = versions_dir();
    let mingw_versions = convert_to_mingw(&versions);
    if let Some(rest) = path_env.strip_prefix(VMR_VERSIONS_PREFIX) {
        let p = format!("{versions}{rest}");
        add_export_line(
            old,
            &format!("export PATH=\"${{PATH}}:{}\"", convert_to_mingw(&p)),
        )
    } else if path_env.starts_with(&versions) {
        add_export_line(
            old,
            &format!("export PATH=\"${{PATH}}:{}\"", convert_to_mingw(path_env)),
        )
    } else if path_env.starts_with(&mingw_versions) {
        add_export_line(old, &format!("export PATH=\"${{PATH}}:{path_env}\""))
    } else {
        old.to_string()
    }
}

fn handle_existing_paths() {
    let mut content = read_bashrc();
    let path = std::env::var("PATH").unwrap_or_default();
    for entry in path.split(';') {
        content = add_path_env(entry, &content);
    }
    write_bashrc(&content);
}

fn add_one_path_env(path_env: &str) {
    let content = read_bashrc();
    write_bashrc(&add_path_env(path_env, &content));
}

fn mingw_cd_hook(install: &str) -> String {
    format!(
        "# cd hook start
export PATH=\"{install}:${{PATH}}\"

if [ -z \"$VM_DISABLE\" ]; then
    cdhook() {{
        if [ $# -eq 0 ]; then
            builtin cd
        else
            builtin cd \"$@\" && vmr cdhook
        fi
    }}
    alias cd='cdhook'

    if [ -z \"${{VMR_CD_INIT}}\" ]; then
        VMR_CD_INIT=\"vmr_cd_init\"
        cdhook \"$(pwd)\"
    fi
fi
# cd hook end"
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

/// Registers `VMR_VERSIONS`, mirrors versions paths into `~/.bashrc`, and rewrites registry
/// `PATH` entries under the versions dir to the `%VMR_VERSIONS%` form
/// (Go `TidyWindowsPathEnv`).
pub fn tidy_path(path: &str) -> String {
    let versions = versions_dir();
    if win_reg::get_env(VMR_VERSIONS_ENV).as_deref() != Some(versions.as_str()) {
        win_reg::set_env(VMR_VERSIONS_ENV, &versions);
    }
    handle_existing_paths();
    add_one_path_env(path);
    patch_cd_hook();
    if path.contains(&versions) {
        path.replace(&versions, VMR_VERSIONS_PREFIX)
    } else {
        path.to_string()
    }
}
