//! 目录契约（plan.md §4.1）：`~/.vmr` 及其子目录。
//!
//! `VMR_SDK_INSTALLATION_DIR` 覆盖 versions/cache；temp 恒在 `~/.vmr/temp`（对齐 Go）。

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

/// `~/.vmr`，vmr 安装目录。
pub fn work_dir() -> PathBuf {
    ensure_dir(&home_dir().join(VMR_WORK_DIR_NAME))
}

/// `~/.vmr/conf.toml`。
pub fn conf_file_path() -> PathBuf {
    work_dir().join("conf.toml")
}

/// 版本安装目录：`VMR_SDK_INSTALLATION_DIR` 覆盖时为其下 `versions`，
/// 否则 `~/.vmr/versions`。
pub fn versions_dir() -> PathBuf {
    let base = match env::var(envs::SDK_INSTALLATION_DIR) {
        Ok(d) if !d.is_empty() => PathBuf::from(d),
        _ => work_dir(),
    };
    ensure_dir(&base.join("versions"))
}

/// 缓存目录：versions 的父目录下 `cache`（跟随 override）。
pub fn cache_dir() -> PathBuf {
    let base = versions_dir()
        .parent()
        .map(PathBuf::from)
        .unwrap_or_else(work_dir);
    ensure_dir(&base.join("cache"))
}

/// 临时解压目录：恒在 `~/.vmr/temp`。
pub fn temp_dir() -> PathBuf {
    ensure_dir(&work_dir().join("temp"))
}

/// Lua 插件目录：`~/.vmr/plugins`。
pub fn plugin_dir() -> PathBuf {
    ensure_dir(&work_dir().join("plugins"))
}

/// 镜像表文件路径：`~/.vmr/customed_mirrors.toml`。
pub fn customed_mirrors_file_path() -> PathBuf {
    work_dir().join("customed_mirrors.toml")
}

/// 镜像表缺失时的拉取地址（下载由 vmr-net 负责，vmr-core 保持无网络叶子）。
pub fn customed_mirrors_url() -> String {
    format!(
        "{}{}/mirrors/customed_mirrors.toml",
        default_reverse_proxy(),
        DEFAULT_HOST_URL
    )
}
