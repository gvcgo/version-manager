use std::fs;
use crate::common;

/// Finds installed SDK versions and the current (symlinked) version.
pub struct InstalledVersionFinder {
    pub plugin_name: String,
    pub installed_versions: Vec<String>,
    pub current_version: String,
}

impl InstalledVersionFinder {
    pub fn new(plugin_name: &str) -> Self {
        InstalledVersionFinder {
            plugin_name: plugin_name.to_string(),
            installed_versions: Vec::new(),
            current_version: String::new(),
        }
    }

    /// Find current version from the symlink target.
    fn find_current_version(&mut self, symbol_path: &std::path::Path) {
        if !symbol_path.exists() {
            return;
        }
        // Read symlink target
        if let Ok(target) = fs::read_link(symbol_path) {
            if let Some(fname) = target.file_name().and_then(|n| n.to_str()) {
                let prefix = format!("{}-", self.plugin_name);
                if let Some(ver) = fname.strip_prefix(&prefix) {
                    self.current_version = ver.to_string();
                }
            }
        }
    }

    /// Find all installed versions and current version.
    /// Returns (installed_versions, current_version).
    pub fn find_all(&mut self, sdk_name: &str) -> (&[String], &str) {
        let version_dir = common::get_sdk_version_dir(sdk_name);
        if !version_dir.exists() {
            return (&self.installed_versions, &self.current_version);
        }

        // Find current from symlink
        let symbol_path = version_dir.join(sdk_name);
        self.find_current_version(&symbol_path);

        let prefix = format!("{}-", self.plugin_name);
        if let Ok(entries) = fs::read_dir(&version_dir) {
            for entry in entries.flatten() {
                if let Ok(ft) = entry.file_type() {
                    if ft.is_dir() {
                        let name = entry.file_name();
                        let name_str = name.to_string_lossy();
                        if let Some(ver) = name_str.strip_prefix(&prefix) {
                            self.installed_versions.push(ver.to_string());
                        }
                    }
                }
            }
        }

        (&self.installed_versions, &self.current_version)
    }

    /// Uninstall all versions for this plugin.
    pub fn uninstall_all_versions(&self, sdk_name: &str) {
        let version_dir = common::get_sdk_version_dir(sdk_name);
        if version_dir.exists() {
            let _ = fs::remove_dir_all(&version_dir);
        }
    }
}
