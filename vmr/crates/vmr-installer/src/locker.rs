//! `.vmr.lock` project lock (mirrors Go `installer/locker.go`).
//!
//! File format: JSON `{"sdk":"version"}` (also accepts the legacy `sdk@version`
//! single-line text and node aliases).

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

pub const LOCKER_FILE_NAME: &str = ".vmr.lock";

#[derive(Debug, Clone, Default)]
pub struct VersionLocker {
    pub versions: HashMap<String, String>,
}

impl VersionLocker {
    /// Searches upward from the current directory for the lock file
    /// (including the starting directory).
    pub fn find_locker_file(start: Option<&Path>) -> Option<PathBuf> {
        let mut d = start
            .map(Path::to_path_buf)
            .or_else(|| std::env::current_dir().ok())?;
        loop {
            let p = d.join(LOCKER_FILE_NAME);
            if p.exists() {
                return Some(p);
            }
            if !d.pop() {
                return None;
            }
        }
    }

    /// Loads the lock (searching upward; empty when not found).
    pub fn load_from(dir: Option<&Path>) -> Self {
        let mut v = VersionLocker::default();
        let Some(p) = Self::find_locker_file(dir) else {
            return v;
        };
        let Ok(data) = fs::read_to_string(&p) else {
            return v;
        };
        v.parse(&data);
        v
    }

    pub fn parse(&mut self, content: &str) {
        let content = content.trim();
        if content.is_empty() {
            return;
        }
        if content.starts_with('{') {
            if let Ok(map) = serde_json::from_str::<HashMap<String, String>>(content) {
                self.versions.extend(map);
            }
        } else if let Some((sdk, ver)) = content.split_once('@') {
            self.versions
                .insert(sdk.trim().to_string(), ver.trim().to_string());
        }
        // node alias compatibility (mirrors Go).
        for (k, v) in self.versions.clone() {
            if k == "nodejs" || k == "node.js" {
                self.versions.insert("node".to_string(), v);
            }
        }
    }

    /// Saves: writes to the current/nearest lock-file location (creates one in the
    /// current directory when none exists).
    pub fn save(&mut self, dir: Option<&Path>, sdk: &str, version: &str) {
        let path = match Self::find_locker_file(dir) {
            Some(p) => p,
            None => {
                let base = dir
                    .map(Path::to_path_buf)
                    .or_else(|| std::env::current_dir().ok())
                    .unwrap_or_else(|| PathBuf::from("."));
                base.join(LOCKER_FILE_NAME)
            }
        };
        self.versions.insert(sdk.to_string(), version.to_string());
        let data = serde_json::to_string_pretty(&self.versions).unwrap_or_default();
        let _ = fs::write(path, data);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_formats_and_node_alias() {
        let mut v = VersionLocker::default();
        v.parse(r#"{"go":"1.22.1"}"#);
        assert_eq!(v.versions.get("go").map(|s| s.as_str()), Some("1.22.1"));

        let mut v2 = VersionLocker::default();
        v2.parse("nodejs@18.1.0");
        assert_eq!(v2.versions.get("node").map(|s| s.as_str()), Some("18.1.0"));
    }
}
