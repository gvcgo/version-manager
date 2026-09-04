//! Directory contract (plan.md §4.1): `~/.vmr` and its subdirectories.
//!
//! `VMR_SDK_INSTALLATION_DIR` overrides versions/cache; temp always lives under
//! `~/.vmr/temp` (mirrors Go).

use std::env;
use std::fs;
use std::path::PathBuf;

use crate::envs;
use crate::{DEFAULT_HOST_URL, VMR_WORK_DIR_NAME, default_reverse_proxy};

fn home_dir() -> PathBuf {
    dirs::home_dir().unwrap_or_else(|| PathBuf::from("."))
}

fn ensure_dir(p: &PathBuf) -> PathBuf {
    let _ = fs::create_dir_all(p);
    p.clone()
}

/// `~/.vmr`, the vmr installation directory.
pub fn work_dir() -> PathBuf {
    ensure_dir(&home_dir().join(VMR_WORK_DIR_NAME))
}

/// `~/.vmr/conf.toml`。
pub fn conf_file_path() -> PathBuf {
    work_dir().join("conf.toml")
}

/// Version installation directory: with `VMR_SDK_INSTALLATION_DIR` set, `versions` under it;
/// otherwise `~/.vmr/versions`.
pub fn versions_dir() -> PathBuf {
    let base = match env::var(envs::SDK_INSTALLATION_DIR) {
        Ok(d) if !d.is_empty() => PathBuf::from(d),
        _ => work_dir(),
    };
    ensure_dir(&base.join("versions"))
}

/// Cache directory: `cache` under versions' parent directory (follows the override).
pub fn cache_dir() -> PathBuf {
    let base = versions_dir()
        .parent()
        .map(PathBuf::from)
        .unwrap_or_else(work_dir);
    ensure_dir(&base.join("cache"))
}

/// Temporary extraction directory: always `~/.vmr/temp`.
pub fn temp_dir() -> PathBuf {
    ensure_dir(&work_dir().join("temp"))
}

/// Lua plugin directory: `~/.vmr/plugins`.
pub fn plugin_dir() -> PathBuf {
    ensure_dir(&work_dir().join("plugins"))
}

/// Mirror table file path: `~/.vmr/customed_mirrors.toml`.
pub fn customed_mirrors_file_path() -> PathBuf {
    work_dir().join("customed_mirrors.toml")
}

/// Fetch URL used when the mirror table is missing (downloads are handled by vmr-net;
/// vmr-core stays a network-free leaf).
pub fn customed_mirrors_url() -> String {
    format!(
        "{}{}/mirrors/customed_mirrors.toml",
        default_reverse_proxy(),
        DEFAULT_HOST_URL
    )
}
