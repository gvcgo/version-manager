//! `.vmr.lock` project lock (mirrors Go `installer/locker.go`).
//!
//! File format: JSON `{"sdk":"version"}` (also accepts the legacy `sdk@version`
//! single-line text and node aliases).

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use vmr_core::names::is_valid_name;
use vmr_core::paths;

pub const LOCKER_FILE_NAME: &str = ".vmr.lock";

#[derive(Debug, Clone, Default)]
pub struct VersionLocker {
    pub versions: HashMap<String, String>,
    /// Lock file the entries were loaded from (`None` when no `.vmr.lock` was found).
    file: Option<PathBuf>,
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
        v.file = Some(p.clone());
        let Ok(data) = fs::read_to_string(&p) else {
            return v;
        };
        v.parse(&data);
        v
    }

    /// Directory that owns the loaded lock file (the project root), if one was found.
    pub fn lock_dir(&self) -> Option<&Path> {
        self.file.as_deref().and_then(Path::parent)
    }

    /// Records the applied project for the shell cd hook.
    ///
    /// A child process cannot export variables to the parent shell, so `vmr cdhook` writes
    /// the resolved lock directory to [`paths::cd_cache_file`]; the generated hook reads it
    /// back into `VMR_CD_CACHE` and skips further lookups while the shell stays inside that
    /// project.
    pub fn write_cd_cache(&self) {
        let _ = fs::write(paths::cd_cache_file(), cd_cache_content(self.lock_dir()));
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
        // The lock file is read from any directory the user `cd`s into, so its names are untrusted:
        // keep only entries that are safe as path components (see `vmr_core::names`).
        self.versions
            .retain(|sdk, ver| is_valid_name(sdk) && is_valid_name(ver));
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

/// Cd-hook cache contents: the lock directory plus a newline, or an empty line when the
/// lookup found no lock above the working directory.
pub fn cd_cache_content(lock_dir: Option<&Path>) -> String {
    match lock_dir {
        Some(dir) => format!("{}\n", dir.to_string_lossy()),
        None => "\n".to_string(),
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

    #[test]
    fn parse_drops_names_that_are_not_path_components() {
        let mut v = VersionLocker::default();
        v.parse(r#"{"../../projects/app":"1","node":"20.11.0","ok":"1/../x"}"#);
        assert_eq!(v.versions.len(), 1);
        assert_eq!(v.versions.get("node").map(|s| s.as_str()), Some("20.11.0"));

        let mut v2 = VersionLocker::default();
        v2.parse("../../projects/app@1");
        assert!(v2.versions.is_empty());
    }

    #[test]
    fn cd_cache_content_records_the_lock_directory() {
        assert_eq!(cd_cache_content(None), "\n");
        assert_eq!(
            cd_cache_content(Some(Path::new("/work/app"))),
            "/work/app\n"
        );
    }

    #[test]
    fn load_from_remembers_the_lock_directory() {
        let dir = std::env::temp_dir().join(format!("vmr-locker-test-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join("sub")).unwrap();
        fs::write(dir.join(LOCKER_FILE_NAME), r#"{"go":"1.22.1"}"#).unwrap();

        let v = VersionLocker::load_from(Some(&dir.join("sub")));
        assert_eq!(v.lock_dir(), Some(dir.as_path()));
        assert_eq!(v.versions.get("go").map(String::as_str), Some("1.22.1"));
        let _ = fs::remove_dir_all(&dir);
    }
}
