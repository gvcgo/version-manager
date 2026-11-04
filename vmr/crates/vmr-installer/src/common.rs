//! Directory conventions (mirrors Go `install/common.go`; disk contract in plan.md §4).
//!
//! - Version root: `<versions>/<sdk>_versions`
//! - Version directory: `<version root>/<plugin>-<version>`
//! - Current version: `<version root>/<sdk>` symlink (Windows junction)

use std::fs;
use std::path::PathBuf;

use vmr_core::paths;

pub const VERSION_DIR_SUFFIX: &str = "_versions";

/// Version root directory of an SDK (auto-created).
pub fn sdk_version_dir(sdk_name: &str) -> PathBuf {
    let dir = paths::versions_dir().join(format!("{sdk_name}{VERSION_DIR_SUFFIX}"));
    let _ = fs::create_dir_all(&dir);
    dir
}

/// Version install directory.
pub fn install_dir(sdk_name: &str, plugin_name: &str, version: &str) -> PathBuf {
    sdk_version_dir(sdk_name).join(format!("{plugin_name}-{version}"))
}

/// Symlink path of the current version (not created).
pub fn symbol_link_path(sdk_name: &str) -> PathBuf {
    sdk_version_dir(sdk_name).join(sdk_name)
}

/// Whether the SDK is already installed by vmr (a subdirectory exists under the version root).
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
