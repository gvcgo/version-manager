use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

const LOCKER_FILE_NAME: &str = ".vmr.lock";

/// Lock SDK versions for a project directory.
/// Searches upward from cwd for a .vmr.lock file.
pub struct VersionLocker {
    pub versions: HashMap<String, String>,
}

impl VersionLocker {
    pub fn new() -> Self {
        VersionLocker {
            versions: HashMap::new(),
        }
    }

    /// Find .vmr.lock file by walking up from current dir.
    pub fn find_locker_file(&self, dir: Option<&std::path::Path>) -> Option<PathBuf> {
        let current = if let Some(d) = dir {
            d.to_path_buf()
        } else {
            std::env::current_dir().unwrap_or_default()
        };

        if current.parent().is_none() || current.as_os_str().is_empty() {
            return None; // Reached root
        }

        let lock_path = current.join(LOCKER_FILE_NAME);
        if lock_path.exists() {
            Some(lock_path)
        } else {
            self.find_locker_file(current.parent())
        }
    }

    /// Load version locks from .vmr.lock file.
    pub fn load(&mut self) {
        let lock_path = match self.find_locker_file(None) {
            Some(p) => p,
            None => return,
        };

        if let Ok(content) = fs::read_to_string(&lock_path) {
            let content = content.trim();
            if content.is_empty() {
                return;
            }
            // Try JSON format first
            if content.starts_with('{') {
                if let Ok(map) = serde_json::from_str::<HashMap<String, String>>(content) {
                    self.versions = map;
                }
            } else {
                // Try old format: "sdk@version"
                if let Some((sdk, ver)) = content.split_once('@') {
                    self.versions.insert(sdk.to_string(), ver.to_string());
                }
            }
        }
    }

    /// Save a version lock. If lock file exists, update it; otherwise create one in cwd.
    pub fn save(&mut self, sdk_name: &str, version_name: &str) {
        let lock_path = self
            .find_locker_file(None)
            .unwrap_or_else(|| std::env::current_dir().unwrap_or_default().join(LOCKER_FILE_NAME));

        // Load existing first
        self.load();

        if !sdk_name.is_empty() && !version_name.is_empty() {
            self.versions.insert(sdk_name.to_string(), version_name.to_string());
        }

        if let Ok(json) = serde_json::to_string_pretty(&self.versions) {
            let _ = fs::write(&lock_path, json);
        }
    }
}
