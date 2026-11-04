//! Installed-version discovery / cache cleanup (mirrors Go `installed.go`, `cached.go`).

use std::fs;
use std::path::PathBuf;

use vmr_core::paths;

use crate::common::{install_dir, sdk_version_dir, symbol_link_path};

/// Installed versions and the current version.
pub struct InstalledInfo {
    pub installed: Vec<String>,
    pub current: Option<String>,
}

fn find_current(plugin: &str, sym_path: &PathBuf) -> Option<String> {
    let target = fs::read_link(sym_path).ok()?;
    let name = target.file_name()?.to_string_lossy().into_owned();
    let prefix = format!("{plugin}-");
    name.strip_prefix(&prefix).map(|s| s.to_string())
}

/// Finds an SDK's installed versions and current version (under the sdk_name version root).
pub fn find_all(sdk_name: &str, plugin_name: &str) -> InstalledInfo {
    let sym = symbol_link_path(sdk_name);
    let version_root = sdk_version_dir(sdk_name);
    let current = find_current(plugin_name, &sym);
    let prefix = format!("{plugin_name}-");
    let mut installed = Vec::new();
    if let Ok(entries) = fs::read_dir(&version_root) {
        for e in entries.flatten() {
            let name = e.file_name().to_string_lossy().into_owned();
            let is_dir = e.file_type().map(|t| t.is_dir()).unwrap_or(false);
            if is_dir && name.starts_with(&prefix) {
                installed.push(name.trim_start_matches(&prefix).to_string());
            }
        }
    }
    installed.sort();
    InstalledInfo { installed, current }
}

/// Uninstalls a single version directory; if it is the current version, also removes the symlink.
pub fn uninstall_version(sdk_name: &str, plugin_name: &str, version: &str) {
    let dir = install_dir(sdk_name, plugin_name, version);
    let _ = fs::remove_dir_all(&dir);
    let info = find_all(sdk_name, plugin_name);
    if info.current.as_deref() == Some(version) {
        let _ = fs::remove_dir_all(symbol_link_path(sdk_name));
    }
}

/// Uninstalls all versions (removes the version root + symlink).
pub fn uninstall_all(sdk_name: &str) {
    let _ = fs::remove_dir_all(sdk_version_dir(sdk_name));
}

/// Deletes cached files: when version is empty, clears every version directory under
/// `<cache>/<plugin>/` (mirrors cached.go).
pub fn delete_cached_files(plugin_name: &str, version: Option<&str>) {
    match version {
        Some(v) => {
            let p = paths::cache_dir()
                .join(plugin_name)
                .join(v.trim_end_matches("<current>"));
            let _ = fs::remove_dir_all(p);
        }
        None => {
            let root = paths::cache_dir().join(plugin_name);
            if let Ok(entries) = fs::read_dir(&root) {
                for e in entries.flatten() {
                    if e.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                        let _ = fs::remove_dir_all(e.path());
                    }
                }
            }
        }
    }
}
