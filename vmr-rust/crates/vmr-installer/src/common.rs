//! 目录约定（对齐 Go `install/common.go`，磁盘契约见 plan.md §4）。
//!
//! - 版本根：`<versions>/<sdk>_versions`
//! - 版本目录：`<版本根>/<plugin>-<version>`
//! - 当前版本：`<版本根>/<sdk>` 符号链接（Windows junction）

use std::fs;
use std::path::PathBuf;

use vmr_core::paths;

pub const VERSION_DIR_SUFFIX: &str = "_versions";

/// 某 SDK 的版本根目录（自动创建）。
pub fn sdk_version_dir(sdk_name: &str) -> PathBuf {
    let dir = paths::versions_dir().join(format!("{sdk_name}{VERSION_DIR_SUFFIX}"));
    let _ = fs::create_dir_all(&dir);
    dir
}

/// 版本安装目录。
pub fn install_dir(sdk_name: &str, plugin_name: &str, version: &str) -> PathBuf {
    sdk_version_dir(sdk_name).join(format!("{plugin_name}-{version}"))
}

/// 当前版本符号链接路径（未创建）。
pub fn symbol_link_path(sdk_name: &str) -> PathBuf {
    sdk_version_dir(sdk_name).join(sdk_name)
}

/// 该 SDK 是否已由 vmr 安装（版本根下存在子目录）。
pub fn is_sdk_installed_by_vmr(sdk_name: &str) -> bool {
    let vd = sdk_version_dir(sdk_name);
    let count = fs::read_dir(&vd)
        .map(|it| {
            it.filter(|e| {
                e.as_ref()
                    .map(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false))
                    .unwrap_or(false)
            })
            .count()
        })
        .unwrap_or(0);
    if count == 0 {
        let _ = fs::remove_dir_all(&vd);
    }
    count > 0
}
