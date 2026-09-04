//! Find the "home directory" among extraction results (mirrors Go `vmr-go/internal/utils/find_dir.go`).
//!
//! After the installer unpacks an archive into temp, the result may sit one directory layer deep,
//! e.g. `xxx-1.0/bin/...`, so the real install root has to be located. Matching rules (identical
//! to Go, quirks included):
//! - The detection target is the **string concatenation of directory-entry names**; flags are
//!   **substring** matches (not equality).
//! - The concatenation covers all entry names; when `except_dir` is set only **non-directory**
//!   entries are concatenated (symlinks included, since they do not count as directories by type).
//! - Each directory self-tests first, hitting when all flags match (DFS pre-order, **stops at the
//!   first hit**); on a miss it recurses into subdirectories, skipping `__MACOSX`.
//! - Directory traversal is **sorted by file name** (mirroring the ordering behavior of Go
//!   `os.ReadDir`) to guarantee determinism.
//! - Symlinked directories are not expanded (`file_type` does not follow links, mirroring Go
//!   `DirEntry.IsDir`).
//! - With an empty flag list the start directory itself matches (same in Go: empty flags all hit).
//! - If already matched before `find`, return directly (the Go `if h.home != "" { return }` guard).

use std::fs;
use std::path::{Path, PathBuf};

/// Home directory finder.
pub struct HomeDirFinder {
    home: Option<PathBuf>,
    flag_files: Vec<String>,
    except_dir: bool,
}

impl HomeDirFinder {
    pub fn new(flag_files: Vec<String>) -> Self {
        HomeDirFinder {
            home: None,
            flag_files,
            except_dir: false,
        }
    }

    /// Reset the flag files (mirrors Go `SetFlags`).
    pub fn set_flags(&mut self, flag_files: Vec<String>) {
        self.flag_files = flag_files;
    }

    /// When set, only **file** names (non-directory entries) take part in matching
    /// (mirrors Go `SetFlagDirExcepted`).
    pub fn set_flag_dir_excepted(&mut self, ok: bool) {
        self.except_dir = ok;
    }

    /// Reset to the initial state (mirrors Go `Clear`).
    pub fn clear(&mut self) {
        self.home = None;
        self.flag_files.clear();
        self.except_dir = false;
    }

    /// DFS search starting from `start_dir`; once matched, further calls do not change the
    /// result (stops at the first hit).
    pub fn find(&mut self, start_dir: &Path) {
        if self.home.is_some() {
            return;
        }
        let entries = match fs::read_dir(start_dir) {
            Ok(iter) => iter,
            Err(_) => return,
        };
        // Only the name and whether it is a directory matter; Go `os.ReadDir` sorts by file name,
        // replicated here for determinism.
        let mut names: Vec<(String, bool)> = entries
            .filter_map(|e| e.ok())
            .map(|e| {
                let is_dir = e.file_type().map(|t| t.is_dir()).unwrap_or(false);
                let name = e.file_name().to_string_lossy().into_owned();
                (name, is_dir)
            })
            .collect();
        names.sort_by(|a, b| a.0.cmp(&b.0));

        // Concatenate all entry names of the current directory (the substring-match target).
        let mut concat = String::new();
        for (name, is_dir) in &names {
            if self.except_dir && *is_dir {
                continue;
            }
            concat.push_str(name);
        }

        // Self-test the current directory: all flags match (substring matching, including the
        // cross-name-boundary concatenation quirk).
        if self.flag_files.iter().all(|f| concat.contains(f.as_str())) {
            self.home = Some(start_dir.to_path_buf());
            return;
        }
        // On a miss, recurse into subdirectories (skipping __MACOSX, consistent with Go).
        for (name, is_dir) in &names {
            if *is_dir && name != "__MACOSX" {
                self.find(&start_dir.join(name));
                if self.home.is_some() {
                    return;
                }
            }
        }
    }

    /// The matched directory; `None` when unmatched (mirrors Go `GetDirName`, where an empty
    /// string means no match).
    pub fn get_dir_name(&self) -> Option<PathBuf> {
        self.home.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process;

    /// Temporary directory for tests; cleaned up automatically on drop.
    struct TempDir(PathBuf);

    impl TempDir {
        fn new(name: &str) -> Self {
            let p =
                std::env::temp_dir().join(format!("vmr-utils-finddir-{}-{name}", process::id()));
            let _ = fs::remove_dir_all(&p);
            fs::create_dir_all(&p).unwrap();
            TempDir(p)
        }

        fn path(&self) -> &Path {
            &self.0
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    /// Create an empty file (parent directories are created automatically).
    fn touch(p: &Path) {
        fs::create_dir_all(p.parent().unwrap()).unwrap();
        fs::write(p, "").unwrap();
    }

    fn flags(list: &[&str]) -> Vec<String> {
        list.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn finds_home_in_top_level_subdir() {
        let t = TempDir::new("top-subdir");
        let home = t.path().join("somepkg-1.0");
        touch(&home.join("bin/tool"));
        touch(&home.join("README"));

        let mut f = HomeDirFinder::new(flags(&["bin"]));
        f.find(t.path());
        assert_eq!(f.get_dir_name().unwrap(), home);
    }

    #[test]
    fn requires_all_flags_present() {
        let t = TempDir::new("all-flags");
        // bin is present but lib is not → the top-level subdirectory misses, and drilling down into
        // bin itself still lacks lib.
        touch(&t.path().join("pkg/bin/x"));
        let mut f = HomeDirFinder::new(flags(&["bin", "lib"]));
        f.find(t.path());
        assert_eq!(f.get_dir_name(), None);
    }

    #[test]
    fn skips_macosx() {
        let t = TempDir::new("macosx");
        // The only match point hides inside __MACOSX → it must be skipped.
        touch(&t.path().join("__MACOSX/zzz"));
        let mut f = HomeDirFinder::new(flags(&["zzz"]));
        f.find(t.path());
        assert_eq!(f.get_dir_name(), None);
    }

    #[test]
    fn except_dir_matches_files_only() {
        let t = TempDir::new("except-dir");
        // LICENSE under root is a **directory**: with except_dir set it is left out of the
        // concatenation → root does not match; without it, root matches.
        fs::create_dir_all(t.path().join("LICENSE")).unwrap();
        let mut f = HomeDirFinder::new(flags(&["LICENSE"]));
        f.find(t.path());
        assert_eq!(f.get_dir_name().unwrap(), t.path());

        let mut f2 = HomeDirFinder::new(flags(&["LICENSE"]));
        f2.set_flag_dir_excepted(true);
        f2.find(t.path());
        assert_eq!(f2.get_dir_name(), None);
    }

    #[test]
    fn recursion_visits_dirs_in_sorted_order() {
        let t = TempDir::new("sorted");
        // Both candidate subdirectories contain mark; with name-sorted DFS, a is hit before b.
        touch(&t.path().join("b/mark"));
        touch(&t.path().join("a/mark"));
        let mut f = HomeDirFinder::new(flags(&["mark"]));
        f.find(t.path());
        assert_eq!(f.get_dir_name().unwrap(), t.path().join("a"));
    }

    #[cfg(unix)]
    #[test]
    fn symlink_dir_is_not_followed() {
        let t = TempDir::new("symlink");
        let target = t.path().join("real");
        touch(&target.join("mark"));
        std::os::unix::fs::symlink(&target, t.path().join("ln")).unwrap();
        let mut f = HomeDirFinder::new(flags(&["mark"]));
        f.find(t.path());
        // Root holds only the real/ln directory entries: the real subdirectory contains mark,
        // so the hit is real.
        assert_eq!(f.get_dir_name().unwrap(), target);
    }

    #[test]
    fn empty_flags_match_start_dir_itself() {
        let t = TempDir::new("empty-flags");
        touch(&t.path().join("x"));
        let mut f = HomeDirFinder::new(flags(&[]));
        f.find(t.path());
        assert_eq!(f.get_dir_name().unwrap(), t.path());
    }

    #[test]
    fn first_hit_is_sticky_and_clear_resets() {
        let t = TempDir::new("sticky");
        touch(&t.path().join("pkg/mark"));
        let mut f = HomeDirFinder::new(flags(&["mark"]));
        f.find(t.path());
        assert!(f.get_dir_name().is_some());
        let first = f.get_dir_name().unwrap().clone();
        // Once matched, find again (with a different root) does not change the result.
        let t2 = TempDir::new("sticky-other");
        f.find(t2.path());
        assert_eq!(f.get_dir_name().unwrap(), first);

        // clear resets: search again with different flags.
        f.clear();
        f.set_flags(flags(&["other"]));
        f.find(t2.path());
        assert_eq!(f.get_dir_name(), None);
    }
}
