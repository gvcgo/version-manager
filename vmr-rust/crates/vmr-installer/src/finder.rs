//! 已装版本发现 / 缓存清理（对齐 Go `installed.go`、`cached.go`）。

use std::fs;
use std::path::PathBuf;

use vmr_core::paths;

use crate::common::{install_dir, sdk_version_dir, symbol_link_path};

/// 已装版本与当前版本。
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

/// 发现某 SDK（按 sdk_name 版本根）已装版本与当前版本。
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

/// 卸载单个版本目录；若是当前版本则同时删符号链接。
pub fn uninstall_version(sdk_name: &str, plugin_name: &str, version: &str) {
    let dir = install_dir(sdk_name, plugin_name, version);
    let _ = fs::remove_dir_all(&dir);
    let info = find_all(sdk_name, plugin_name);
    if info.current.as_deref() == Some(version) {
        let _ = fs::remove_dir_all(symbol_link_path(sdk_name));
    }
}

/// 卸载全部版本（删版本根 + 符号链接）。
pub fn uninstall_all(sdk_name: &str) {
    let _ = fs::remove_dir_all(sdk_version_dir(sdk_name));
}

/// 删除缓存：version 为空时清整个 `<cache>/<plugin>/` 下版本目录（对齐 cached.go）。
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
