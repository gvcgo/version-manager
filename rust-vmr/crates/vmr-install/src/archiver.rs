use std::path::{Path, PathBuf};

use crate::common::{self, InstallerConfig};

// ---------------------------------------------------------------------------
// ArchiverInstaller — install SDKs distributed as compressed archives
// ---------------------------------------------------------------------------

/// Installs an SDK version by downloading a compressed archive, extracting it,
/// locating the SDK home directory via flag files, and copying it to the
/// version-specific installation directory.
pub struct ArchiverInstaller {
    pub plugin_name: String,
    pub sdk_name: String,
    pub version_name: String,
    pub version: vmr_download::Item,
    pub install_conf: Option<InstallerConfig>,
    pub dir_finder: vmr_utils::fs::HomeDirFinder,
}

impl ArchiverInstaller {
    pub fn new() -> Self {
        ArchiverInstaller {
            plugin_name: String::new(),
            sdk_name: String::new(),
            version_name: String::new(),
            version: vmr_download::Item {
                url: String::new(),
                arch: String::new(),
                os: String::new(),
                installer: String::new(),
                sum: String::new(),
                sum_type: String::new(),
                size: 0,
            },
            install_conf: None,
            dir_finder: vmr_utils::fs::HomeDirFinder::new(Vec::new()),
        }
    }

    /// Returns the installation directory:
    /// `{sdk_version_dir}/{plugin_name}-{version_name}`
    pub fn get_install_dir(&self) -> PathBuf {
        let d = common::get_sdk_version_dir(&self.sdk_name);
        d.join(format!("{}-{}", self.plugin_name, self.version_name))
    }

    /// Returns the symlink path: `{sdk_version_dir}/{sdk_name}`
    pub fn get_symbol_link_path(&self) -> PathBuf {
        let d = common::get_sdk_version_dir(&self.sdk_name);
        d.join(&self.sdk_name)
    }

    /// Prepare the `HomeDirFinder` with flag files from the install config.
    fn prepare_dir_finder(&mut self) {
        self.dir_finder.clear();
        if let Some(ref conf) = self.install_conf {
            if let Some(ref ff) = conf.flag_files {
                let flags = ff.for_current_os();
                self.dir_finder.set_flags(flags);
                self.dir_finder.set_flag_dir_excepted(conf.flag_dir_excepted);
            }
        }
    }

    /// Handle git's self-extracting `.7z.exe` archives on Windows by renaming
    /// to plain `.7z` so the extraction pipeline can handle them.
    fn handle_archived_file(fpath: &Path) -> PathBuf {
        let fname = fpath.to_string_lossy();
        if fname.contains("git") && fname.ends_with(".7z.exe") {
            let new_name = fname.trim_end_matches(".exe").to_string();
            let new_path = PathBuf::from(&new_name);
            let _ = std::fs::rename(fpath, &new_path);
            return new_path;
        }
        fpath.to_path_buf()
    }

    /// Patch the file name of a single-file extracted archive (e.g. renames
    /// the standalone executable to match the SDK name and makes it executable
    /// on Unix).
    fn patch_file_name(&self) {
        let temp_dir = vmr_config::paths::get_temp_dir();
        let entries: Vec<_> = match std::fs::read_dir(&temp_dir) {
            Ok(d) => d.filter_map(|e| e.ok()).collect(),
            Err(_) => return,
        };

        let mut new_name = self.sdk_name.clone();
        if cfg!(target_os = "windows") {
            new_name.push_str(".exe");
        }

        if entries.len() == 1
            && !entries[0]
                .file_type()
                .map(|t| t.is_dir())
                .unwrap_or(true)
        {
            let dd = &entries[0];
            let fname = dd.file_name().to_string_lossy().to_string();
            if fname.contains(&self.sdk_name) && fname != new_name {
                let old_path = temp_dir.join(&fname);
                let new_path = temp_dir.join(&new_name);
                let _ = std::fs::rename(&old_path, &new_path);
                // Make executable on Unix
                if !cfg!(target_os = "windows") {
                    #[cfg(unix)]
                    {
                        use std::os::unix::fs::PermissionsExt;
                        let _ = std::fs::set_permissions(
                            &new_path,
                            std::fs::Permissions::from_mode(0o755),
                        );
                    }
                    #[cfg(not(unix))]
                    let _ = ();
                }
            }
        }
    }

    // -----------------------------------------------------------------------
    // Main install entry point
    // -----------------------------------------------------------------------

    /// Downloads, extracts, locates, and copies the SDK archive.
    pub fn install(&mut self) {
        if self.version.url.is_empty() {
            eprintln!("[archiver] empty download URL, skipping");
            return;
        }

        // 1. Download archived file.
        let mut dd = vmr_download::Downloader::new();
        let fpath_str =
            dd.download(&self.plugin_name, &self.version_name, self.version.clone());
        if fpath_str.is_empty() {
            eprintln!(
                "[archiver] download failed for {}@{}",
                self.plugin_name, self.version_name
            );
            return;
        }
        let fpath = Self::handle_archived_file(Path::new(&fpath_str));

        // 2. Extract to temp directory.
        let temp_dir = vmr_config::paths::get_temp_dir();
        if let Err(e) = vmr_utils::archive::extract(&fpath, &temp_dir) {
            eprintln!("[archiver] extract failed: {}", e);
            if let Some(parent) = fpath.parent() {
                let _ = std::fs::remove_dir_all(parent);
            }
            return;
        }

        // 3. Patch filename for single-file archives.
        self.patch_file_name();

        // 4. Use HomeDirFinder to locate the SDK home directory.
        self.prepare_dir_finder();
        self.dir_finder.find(&temp_dir);
        let dir_to_copy = match self.dir_finder.get_dir_name() {
            Some(d) => d.clone(),
            None => {
                eprintln!("[archiver] cannot find SDK home dir in extracted content");
                if let Some(parent) = fpath.parent() {
                    let _ = std::fs::remove_dir_all(parent);
                }
                return;
            }
        };

        // 5. Copy to install directory.
        let install_dir = self.get_install_dir();
        if let Err(e) = vmr_utils::fs::copy_directory(&dir_to_copy, &install_dir) {
            eprintln!("[archiver] copy directory failed: {}", e);
            if let Some(parent) = fpath.parent() {
                let _ = std::fs::remove_dir_all(parent);
            }
            return;
        }

        // 6. Clean up temp directory.
        let _ = std::fs::remove_dir_all(&temp_dir);
    }
}

impl Default for ArchiverInstaller {
    fn default() -> Self {
        Self::new()
    }
}
