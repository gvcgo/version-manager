//! Plugin data structures (mirrors Go `lua_global/version.go` and `installer.go`).

use serde::{Deserialize, Serialize};

/// A single version entry (mirrors Go `Item`).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Item {
    #[serde(default)]
    pub url: String,
    #[serde(default)]
    pub arch: String,
    #[serde(default)]
    pub os: String,
    #[serde(default)]
    pub sum: String,
    #[serde(default)]
    pub sum_type: String,
    #[serde(default)]
    pub size: i64,
    #[serde(default)]
    pub installer: String,
    #[serde(default)]
    pub lts: String,
    #[serde(default)]
    pub extra: String,
}

/// The set of candidate entries for a single version (mirrors Go `SDKVersion`).
pub type SDKVersion = Vec<Item>;

/// The version table returned by crawl (mirrors Go `VersionList`).
pub type VersionList = std::collections::HashMap<String, SDKVersion>;

/// Per-platform file list (mirrors Go `FileItems`; json key `darwin`).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct FileItems {
    #[serde(default)]
    pub windows: Vec<String>,
    #[serde(default)]
    pub linux: Vec<String>,
    #[serde(default)]
    pub darwin: Vec<String>,
}

/// A path group (a set of relative paths = one bin directory / env path).
pub type DirPath = Vec<String>;

/// Per-platform path group lists (mirrors Go `DirItems`).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct DirItems {
    #[serde(default)]
    pub windows: Vec<DirPath>,
    #[serde(default)]
    pub linux: Vec<DirPath>,
    #[serde(default)]
    pub darwin: Vec<DirPath>,
}

/// Additional environment variables (mirrors Go `AdditionalEnv`).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct AdditionalEnv {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub value: Vec<DirPath>,
    #[serde(default)]
    pub version: String,
}

/// Binary rename (mirrors Go `BinaryRename`; unused by current plugins, kept for contract).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct BinaryRename {
    #[serde(default)]
    pub name_flag: String,
    #[serde(default)]
    pub rename_to: String,
}

/// Installer configuration (mirrors Go `InstallerConfig`; the Lua global variable name is `ic`).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct InstallerConfig {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub flag_files: Option<FileItems>,
    #[serde(default)]
    pub flag_dir_excepted: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub binary_dirs: Option<DirItems>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub binary_rename: Option<BinaryRename>,
    #[serde(default)]
    pub additional_envs: Vec<AdditionalEnv>,
}

impl InstallerConfig {
    /// Mirrors Go `NewInstallerConfig`: null pointer fields become empty structs (the Lua side appends per platform).
    pub fn new() -> Self {
        InstallerConfig {
            flag_files: Some(FileItems::default()),
            flag_dir_excepted: false,
            binary_dirs: Some(DirItems::default()),
            binary_rename: Some(BinaryRename::default()),
            additional_envs: Vec::new(),
        }
    }
}

/// Per-platform field selection (mirrors the pick-by-platform logic in Go `CollectEnvs` and elsewhere).
pub fn file_items_for<'a>(fi: &'a FileItems, os: &str) -> &'a Vec<String> {
    match os {
        "windows" => &fi.windows,
        "darwin" => &fi.darwin,
        _ => &fi.linux,
    }
}

pub fn dir_items_for<'a>(di: &'a DirItems, os: &str) -> &'a Vec<DirPath> {
    match os {
        "windows" => &di.windows,
        "darwin" => &di.darwin,
        _ => &di.linux,
    }
}

/// installer kind constants (mirrors Go `lua_global/version.go`).
pub mod installer_kind {
    pub const CONDA: &str = "conda";
    pub const CONDA_FORGE: &str = "conda-forge";
    pub const COURSIER: &str = "coursier";
    pub const UNARCHIVER: &str = "unarchiver";
    pub const EXECUTABLE: &str = "executable";
    pub const DPKG: &str = "dpkg";
    pub const RPM: &str = "rpm";
}
