use std::path::PathBuf;

use crate::common;

// ---------------------------------------------------------------------------
// CoursierInstaller — install JVM languages via coursier
// ---------------------------------------------------------------------------

/// Environment variable name for a custom coursier binary path.
pub const COURSIER_PATH_ENV_NAME: &str = "VMR_COURSIER_PATH";

/// Installs JVM-based SDKs (Scala, etc.) using the coursier `cs` command.
///
/// Equivalent to running:
/// ```text
/// cs install --install-dir=<install_dir> <plugin>:<version>
/// ```
pub struct CoursierInstaller {
    pub plugin_name: String,
    pub sdk_name: String,
    pub version_name: String,
    pub version: vmr_download::Item,
}

impl CoursierInstaller {
    pub fn new() -> Self {
        CoursierInstaller {
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

    /// Strip `-LTS` / `-lts` suffix from the version name (coursier does not
    /// use those suffixes in its install command).
    fn clean_version(version: &str) -> String {
        version
            .trim_end_matches("-LTS")
            .trim_end_matches("-lts")
            .to_string()
    }

    // -----------------------------------------------------------------------
    // Main install entry point
    // -----------------------------------------------------------------------

    /// Runs `cs install --install-dir=<install_dir> <plugin>:<version>`.
    ///
    /// Respects `VMR_COURSIER_PATH` if set; otherwise defaults to `cs`.
    pub fn install(&self) {
        if self.version.url.is_empty() {
            eprintln!("[coursier] empty download URL, skipping");
            return;
        }

        let home_dir = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/tmp"));
        let version = Self::clean_version(&self.version_name);
        let install_dir = self.get_install_dir();
        let install_dir_str = install_dir.to_string_lossy().to_string();

        let coursier_cmd = std::env::var(COURSIER_PATH_ENV_NAME)
            .unwrap_or_else(|_| "cs".to_string());

        let status = std::process::Command::new(&coursier_cmd)
            .args([
                "install",
                "-q",
                &format!("--install-dir={}", install_dir_str),
                &format!("{}:{}", self.plugin_name, version),
            ])
            .current_dir(&home_dir)
            .status();

        match status {
            Ok(s) if s.success() => {}
            Ok(s) => {
                eprintln!(
                    "[coursier] cs install exited with code {:?} for {}",
                    s.code(),
                    self.plugin_name
                );
            }
            Err(e) => {
                eprintln!(
                    "[coursier] failed to run {} install: {}",
                    coursier_cmd, e
                );
            }
        }
    }
}

impl Default for CoursierInstaller {
    fn default() -> Self {
        Self::new()
    }
}
