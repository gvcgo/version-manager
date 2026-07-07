use std::path::PathBuf;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

pub const VERSION_DIR_SUFFIX: &str = "_versions";
pub const VERSION_INSTALL_DIR_PATTERN: &str = "%s-%s";

// SDK names used by executable installer
pub const MINICONDA_SDK_NAME: &str = "miniconda";
pub const ERLANG_SDK_NAME: &str = "erlang";
pub const ELIXIR_SDK_NAME: &str = "elixir";
pub const VSCODE_SDK_NAME: &str = "vscode";

// ---------------------------------------------------------------------------
// SDK version directory helpers
// ---------------------------------------------------------------------------

/// Returns the SDK version directory: `{versions_dir}/{sdk_name}_versions`
pub fn get_sdk_version_dir(sdk_name: &str) -> PathBuf {
    let d = vmr_config::paths::get_versions_dir()
        .join(format!("{}{}", sdk_name, VERSION_DIR_SUFFIX));
    let _ = std::fs::create_dir_all(&d);
    d
}

/// Check whether an SDK has installed versions.
///
/// Reads the SDK version directory; if it contains zero subdirectories the
/// directory itself is removed and `false` is returned.
pub fn is_sdk_installed_by_vmr(sdk_name: &str) -> bool {
    let vd = get_sdk_version_dir(sdk_name);
    if let Ok(entries) = std::fs::read_dir(&vd) {
        let count = entries
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false))
            .count();
        if count == 0 {
            let _ = std::fs::remove_dir_all(&vd);
        }
        count > 0
    } else {
        false
    }
}

// ---------------------------------------------------------------------------
// InstallerConfig — installation configuration
// ---------------------------------------------------------------------------

/// Platform-specific file lists.
#[derive(Debug, Clone, Default)]
pub struct FileItems {
    pub windows: Vec<String>,
    pub linux: Vec<String>,
    pub macos: Vec<String>,
}

impl FileItems {
    /// Returns the file list for the current operating system.
    pub fn for_current_os(&self) -> Vec<String> {
        if cfg!(target_os = "windows") {
            self.windows.clone()
        } else if cfg!(target_os = "macos") {
            self.macos.clone()
        } else {
            self.linux.clone()
        }
    }
}

/// Binary rename configuration.
#[derive(Debug, Clone, Default)]
pub struct BinaryRename {
    pub name_flag: String,
    pub rename_to: String,
}

/// Installation configuration used by archiver / executable / coursier
/// installers.
///
/// This is a minimal placeholder — the full version (derived from Lua plugins)
/// will be added later.
#[derive(Debug, Clone, Default)]
pub struct InstallerConfig {
    /// Flag files / directories used by `HomeDirFinder` to locate the extracted
    /// SDK home directory.
    pub flag_files: Option<FileItems>,
    /// When `true`, `HomeDirFinder` skips directory entries (only considers
    /// files) while searching.
    pub flag_dir_excepted: bool,
    /// Binary rename rules (applied after install for standalone executables).
    pub binary_rename: Option<BinaryRename>,
    // Placeholder — additional fields (binary_dirs, additional_envs, …) will
    // be populated by the Lua plugin system in a future iteration.
}
