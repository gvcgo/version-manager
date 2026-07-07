use std::fs;

/// Manages cached downloaded SDK files.
pub struct CachedFileFinder {
    pub plugin_name: String,
    pub version_name: Option<String>,
}

impl CachedFileFinder {
    pub fn new(plugin_name: &str, version_name: Option<&str>) -> Self {
        CachedFileFinder {
            plugin_name: plugin_name.to_string(),
            version_name: version_name.map(|s| s.to_string()),
        }
    }

    /// Delete cached files.
    /// If version_name is None, deletes all cached versions for this plugin.
    /// If version_name is Some, deletes only that specific version.
    pub fn delete(&self) {
        let cache_dir = vmr_config::paths::get_cache_dir();

        match &self.version_name {
            Some(vname) => {
                // Delete specific version cache
                let d = cache_dir.join(&self.plugin_name).join(vname);
                let _ = fs::remove_dir_all(&d);
            }
            None => {
                // Delete all cached files for this plugin but keep the plugin dir
                let plugin_cache = cache_dir.join(&self.plugin_name);
                if let Ok(entries) = fs::read_dir(&plugin_cache) {
                    for entry in entries.flatten() {
                        if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                            let _ = fs::remove_dir_all(entry.path());
                        }
                    }
                }
            }
        }
    }
}
