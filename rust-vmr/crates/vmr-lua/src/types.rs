use serde::{Deserialize, Serialize};

/// Installer type constants — mirrors Go's lua_global constants
pub const CONDA: &str = "conda";
pub const CONDA_FORGE: &str = "conda-forge";
pub const COURSIER: &str = "coursier";
pub const UNARCHIVER: &str = "unarchiver";
pub const EXECUTABLE: &str = "executable";
pub const DPKG: &str = "dpkg";
pub const RPM: &str = "rpm";

/// A single SDK version item — mirrors Go's lua_global.Item
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
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

/// Map of version name → Item (for current OS/Arch)
pub type VersionList = std::collections::HashMap<String, Item>;

/// Map for pre-filtered versions — ported from Go's VersionList (which is multi-arch)
/// The Lua crawler returns map[string][]Item; we filter for current OS/Arch.
pub type RawVersionList = std::collections::HashMap<String, Vec<Item>>;

/// Plugin metadata (loaded from Lua globals)
#[derive(Debug, Clone, Default)]
pub struct PluginMeta {
    pub file_name: String,
    pub plugin_name: String,
    pub plugin_version: String,
    pub sdk_name: String,
    pub prequisite: String,
    pub homepage: String,
}

/// Lua config item names (port from fromlua.go)
pub mod lua_items {
    pub const SDK_NAME: &str = "sdk_name";
    pub const PLUGIN_NAME: &str = "plugin_name";
    pub const PLUGIN_VERSION: &str = "plugin_version";
    pub const PREQUISITE: &str = "prequisite";
    pub const HOMEPAGE: &str = "homepage";
    pub const CRAWLER: &str = "crawl";
    pub const INSTALLER_CONFIG: &str = "ic";
    pub const POST_INSTALL: &str = "postInstall";
    pub const CUSTOM_INSTALL: &str = "install";
}

/// Platform-specific file lists
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FileItems {
    #[serde(default)]
    pub windows: Vec<String>,
    #[serde(default)]
    pub linux: Vec<String>,
    #[serde(default, rename = "darwin")]
    pub macos: Vec<String>,
}

/// Binary rename configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BinaryRename {
    #[serde(default)]
    pub name_flag: String,
    #[serde(default)]
    pub rename_to: String,
}

/// Additional environment variable configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AdditionalEnv {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub value: Vec<Vec<String>>,
    #[serde(default)]
    pub version: String,
}

/// Installer configuration (from Lua plugin)
#[derive(Debug, Clone, Default)]
pub struct InstallerConfig {
    pub flag_files: Option<FileItems>,
    pub flag_dir_excepted: bool,
    pub binary_dirs: Option<FileItems>,
    pub binary_rename: Option<BinaryRename>,
    pub additional_envs: Vec<AdditionalEnv>,
}
