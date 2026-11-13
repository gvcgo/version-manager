//! Directory contract (plan.md §4.1): `~/.vmr` and its subdirectories.
//!
//! `VMR_SDK_INSTALLATION_DIR` overrides versions/cache; temp always lives under
//! `~/.vmr/temp` (mirrors Go).

use std::env;
use std::fs;
use std::path::PathBuf;

use crate::VMR_WORK_DIR_NAME;
use crate::envs;

/// User home directory; `.` when it cannot be resolved.
pub fn home_dir() -> PathBuf {
    dirs::home_dir().unwrap_or_else(|| PathBuf::from("."))
}

fn ensure_dir(p: &PathBuf) -> PathBuf {
    let _ = fs::create_dir_all(p);
    p.clone()
}

/// `~/.vmr`, the vmr installation directory (no side effects).
pub fn work_dir_path() -> PathBuf {
    home_dir().join(VMR_WORK_DIR_NAME)
}

/// `~/.vmr`, the vmr installation directory.
pub fn work_dir() -> PathBuf {
    ensure_dir(&work_dir_path())
}

/// Primary config file: `~/.vmr/conf.toml` (no side effects).
pub fn conf_file_default_path() -> PathBuf {
    work_dir_path().join("conf.toml")
}

/// XDG-style config file: `~/.config/vmr/conf.toml`, read only as a fallback.
///
/// Deliberately home-relative on every platform so the same path works on Linux, macOS and
/// Windows; `XDG_CONFIG_HOME` is not consulted.
pub fn xdg_conf_file_path() -> PathBuf {
    home_dir().join(".config").join("vmr").join("conf.toml")
}

/// Config file to read: `~/.vmr/conf.toml` when it exists, else the XDG fallback.
///
/// `None` when neither exists. No side effects: nothing is created.
pub fn conf_file_path() -> Option<PathBuf> {
    let primary = conf_file_default_path();
    if primary.is_file() {
        return Some(primary);
    }
    let fallback = xdg_conf_file_path();
    fallback.is_file().then_some(fallback)
}

/// Config file to write: the one [`conf_file_path`] resolved, else the primary location
/// (created on demand).
pub fn conf_file_write_path() -> PathBuf {
    conf_file_path().unwrap_or_else(|| work_dir().join("conf.toml"))
}

/// Version installation directory (no side effects): with `VMR_SDK_INSTALLATION_DIR` set,
/// `versions` under it; otherwise `~/.vmr/versions`.
pub fn versions_dir_path() -> PathBuf {
    let base = match env::var(envs::SDK_INSTALLATION_DIR) {
        Ok(d) if !d.is_empty() => PathBuf::from(d),
        _ => work_dir_path(),
    };
    base.join("versions")
}

/// Version installation directory: with `VMR_SDK_INSTALLATION_DIR` set, `versions` under it;
/// otherwise `~/.vmr/versions`.
pub fn versions_dir() -> PathBuf {
    ensure_dir(&versions_dir_path())
}

/// Cache directory (no side effects): `cache` under versions' parent directory (follows the
/// override).
pub fn cache_dir_path() -> PathBuf {
    let base = versions_dir_path()
        .parent()
        .map(PathBuf::from)
        .unwrap_or_else(work_dir_path);
    base.join("cache")
}

/// Cache directory: `cache` under versions' parent directory (follows the override).
pub fn cache_dir() -> PathBuf {
    ensure_dir(&cache_dir_path())
}

/// Temporary extraction directory (no side effects): always `~/.vmr/temp`.
pub fn temp_dir_path() -> PathBuf {
    work_dir_path().join("temp")
}

/// Temporary extraction directory: always `~/.vmr/temp`.
pub fn temp_dir() -> PathBuf {
    ensure_dir(&temp_dir_path())
}

/// Shell cd-hook state: the `.vmr.lock` directory applied by the last `vmr cdhook` run.
///
/// A child process cannot export variables to the parent shell, so `vmr cdhook` writes the
/// resolved project here and the generated hook reads it back into `VMR_LOCKFILE_DIR`. The
/// path is rendered into the hook by `vmr init`, so it must not move.
pub fn lockfile_dir_file() -> PathBuf {
    work_dir().join("cd_hook.lockfile_dir")
}

/// Destination recorded by a `.vmr.lock` session when its shell `cd`s out of the project.
///
/// The child shell exits immediately and cannot move the parent, so it writes the target
/// directory here; the outer cd hook reads it back after `vmr cdhook` returns and `cd`s
/// there before looking up the new directory. The path is rendered into the hook by
/// `vmr init`, so it must not move.
pub fn session_exit_file() -> PathBuf {
    work_dir().join("session_exit")
}

/// Plugin directory: `~/.vmr/plugins`.
pub fn plugin_dir() -> PathBuf {
    ensure_dir(&work_dir().join("plugins"))
}
