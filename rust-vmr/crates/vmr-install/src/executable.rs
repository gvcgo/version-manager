use std::path::PathBuf;

use crate::common::{self, InstallerConfig};

// ---------------------------------------------------------------------------
// ExeInstaller — install SDKs distributed as executables / installers
// ---------------------------------------------------------------------------

/// Handles various executable-based installation strategies:
/// - Standalone executables (copy + chmod)
/// - Miniconda shell installers
/// - Visual Studio Code (platform-specific packages)
/// - Windows EXE installers (Erlang, Elixir, etc.)
pub struct ExeInstaller {
    pub plugin_name: String,
    pub sdk_name: String,
    pub version_name: String,
    pub version: vmr_download::Item,
    pub install_conf: Option<InstallerConfig>,
}

impl ExeInstaller {
    pub fn new() -> Self {
        ExeInstaller {
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

    /// After install, rename binaries according to `BinaryRename` config.
    fn rename_files(&self) {
        if let Some(ref conf) = self.install_conf {
            if let Some(ref br) = conf.binary_rename {
                if br.name_flag.is_empty() {
                    return;
                }
                let install_dir = self.get_install_dir();
                if let Ok(entries) = std::fs::read_dir(&install_dir) {
                    for entry in entries.flatten() {
                        let ft = entry.file_type().map(|t| t.is_dir()).unwrap_or(true);
                        if ft {
                            continue;
                        }
                        let name = entry.file_name().to_string_lossy().to_string();
                        if name.contains(&br.name_flag) {
                            let mut new_name = br.rename_to.clone();
                            if cfg!(target_os = "windows") {
                                new_name.push_str(".exe");
                            }
                            let new_path = install_dir.join(&new_name);
                            let _ = std::fs::rename(entry.path(), &new_path);
                        }
                    }
                }
            }
        }
    }

    // -----------------------------------------------------------------------
    // Install strategies (private helpers)
    // -----------------------------------------------------------------------

    /// Install a .sh-based Miniconda installer.
    ///
    /// Linux/macOS: `bash <exePath> -b -p <installDir>`
    /// Windows: `start /wait "" <exePath> /InstallationType=JustMe /RegisterPython=0 /S /D=<installDir>`
    fn install_miniconda(&self, exe_path: &str, install_dir: &str) {
        let home_dir = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/tmp"));

        if cfg!(target_os = "windows") {
            let _ = std::process::Command::new("cmd")
                .args([
                    "/C",
                    "start",
                    "/wait",
                    "",
                    exe_path,
                    "/InstallationType=JustMe",
                    "/RegisterPython=0",
                    "/S",
                    &format!("/D={}", install_dir),
                ])
                .current_dir(&home_dir)
                .status();
        } else {
            // Make executable first
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let _ = std::fs::set_permissions(
                    std::path::Path::new(exe_path),
                    std::fs::Permissions::from_mode(0o755),
                );
            }
            let _ = std::process::Command::new("bash")
                .args([exe_path, "-b", "-p", install_dir])
                .current_dir(&home_dir)
                .status();
        }
    }

    /// Install a Windows .exe silently (Erlang / Elixir).
    ///
    /// `start /wait <exePath> /S /D=<installDir>`
    fn install_exe_for_windows(&self, exe_path: &str, install_dir: &str) {
        if !cfg!(target_os = "windows") {
            return;
        }
        let home_dir = dirs::home_dir().unwrap_or_else(|| PathBuf::from("C:\\"));
        let _ = std::process::Command::new("cmd")
            .args([
                "/C",
                "start",
                "/wait",
                exe_path,
                "/S",
                &format!("/D={}", install_dir),
            ])
            .current_dir(&home_dir)
            .status();
    }

    /// Install Visual Studio Code.
    ///
    /// - macOS: extract .zip, locate `Visual Studio Code.app`, `sudo mv` to
    ///   `/Applications`.
    /// - Linux: `sudo dpkg -i <pkg>` or `sudo rpm -ivh <pkg>`.
    /// - Windows: run `.exe` installer with `/VERYSILENT`.
    fn install_vscode(&self, pkg_path: &str, _install_dir: &str) {
        let home_dir = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/tmp"));
        let pkg = std::path::Path::new(pkg_path);

        if cfg!(target_os = "macos") {
            let temp_dir = vmr_config::paths::get_temp_dir();
            if let Err(e) = vmr_utils::archive::extract(pkg, &temp_dir) {
                eprintln!("[exe] extract vscode failed: {}", e);
                let _ = std::fs::remove_file(pkg);
                return;
            }

            // Locate the .app bundle
            let app_name = "Visual Studio Code.app";
            let mut finder = vmr_utils::fs::HomeDirFinder::new(vec![app_name.to_string()]);
            finder.find(&temp_dir);
            if let Some(dir_name) = finder.get_dir_name() {
                let app_path = dir_name.join(app_name);
                if app_path.exists() {
                    // sudo mv to /Applications
                    let _ = std::process::Command::new("sudo")
                        .args(["mv", &app_path.to_string_lossy(), "/Applications/"])
                        .status();
                }
            }
            let _ = std::fs::remove_dir_all(&temp_dir);
        } else if cfg!(target_os = "linux") {
            let fname = pkg
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_lowercase();

            if fname.ends_with(".deb") {
                let _ = std::process::Command::new("sudo")
                    .args(["dpkg", "-i", pkg_path])
                    .current_dir(&home_dir)
                    .status();
            } else if fname.ends_with(".rpm") {
                let _ = std::process::Command::new("sudo")
                    .args(["rpm", "-ivh", pkg_path])
                    .current_dir(&home_dir)
                    .status();
            }
        } else if cfg!(target_os = "windows") {
            let fname = pkg
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_lowercase();
            if fname.ends_with(".exe") {
                let _ = std::process::Command::new(pkg_path)
                    .args(["/VERYSILENT", "/MERGETASKS=!runcode"])
                    .current_dir(&home_dir)
                    .status();
            }
        }
    }

    /// Install a standalone executable — copy to install dir and make it
    /// executable on Unix.
    fn install_standalone(&self, exe_path: &str) {
        let src = std::path::Path::new(exe_path);
        let fname = src
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("vmr_executable");

        let install_dir = self.get_install_dir();
        let _ = std::fs::create_dir_all(&install_dir);
        let dest_path = install_dir.join(fname);

        if src.exists() {
            if let Err(e) = std::fs::copy(src, &dest_path) {
                eprintln!("[exe] copy standalone executable failed: {}", e);
                if let Some(parent) = src.parent() {
                    let _ = std::fs::remove_dir_all(parent);
                }
                return;
            }
            // Make executable on Unix
            if !cfg!(target_os = "windows") {
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    let _ = std::fs::set_permissions(
                        &dest_path,
                        std::fs::Permissions::from_mode(0o755),
                    );
                }
                #[cfg(not(unix))]
                let _ = ();
            }
        }
    }

    // -----------------------------------------------------------------------
    // Main install entry point
    // -----------------------------------------------------------------------

    /// Downloads the file and dispatches to the appropriate install strategy
    /// based on `plugin_name`.
    pub fn install(&mut self) {
        if self.version.url.is_empty() {
            eprintln!("[exe] empty download URL, skipping");
            return;
        }

        let mut dd = vmr_download::Downloader::new();
        let fpath = dd.download(&self.plugin_name, &self.version_name, self.version.clone());
        if fpath.is_empty() {
            eprintln!(
                "[exe] download failed for {}@{}",
                self.plugin_name, self.version_name
            );
            return;
        }

        let install_dir = self.get_install_dir();
        let install_dir_str = install_dir.to_string_lossy().to_string();

        match self.plugin_name.as_str() {
            common::MINICONDA_SDK_NAME => {
                self.install_miniconda(&fpath, &install_dir_str);
            }
            common::ERLANG_SDK_NAME | common::ELIXIR_SDK_NAME => {
                self.install_exe_for_windows(&fpath, &install_dir_str);
            }
            common::VSCODE_SDK_NAME => {
                self.install_vscode(&fpath, &install_dir_str);
            }
            _ => {
                self.install_standalone(&fpath);
                self.rename_files();
            }
        }
    }
}

impl Default for ExeInstaller {
    fn default() -> Self {
        Self::new()
    }
}
