//! 插件数据结构（对齐 Go `lua_global/version.go` 与 `installer.go`）。

use serde::{Deserialize, Serialize};

/// 单个版本条目（对齐 Go `Item`）。
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

/// 单个版本的候选条目集合（对齐 Go `SDKVersion`）。
pub type SDKVersion = Vec<Item>;

/// crawl 返回的版本表（对齐 Go `VersionList`）。
pub type VersionList = std::collections::HashMap<String, SDKVersion>;

/// 平台文件列表（对齐 Go `FileItems`；json 键 darwin）。
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct FileItems {
    #[serde(default)]
    pub windows: Vec<String>,
    #[serde(default)]
    pub linux: Vec<String>,
    #[serde(default)]
    pub darwin: Vec<String>,
}

/// 路径组（一组相对路径 = 一个 bin 目录 / env 路径）。
pub type DirPath = Vec<String>;

/// 各平台路径组列表（对齐 Go `DirItems`）。
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct DirItems {
    #[serde(default)]
    pub windows: Vec<DirPath>,
    #[serde(default)]
    pub linux: Vec<DirPath>,
    #[serde(default)]
    pub darwin: Vec<DirPath>,
}

/// 附加环境变量（对齐 Go `AdditionalEnv`）。
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct AdditionalEnv {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub value: Vec<DirPath>,
    #[serde(default)]
    pub version: String,
}

/// 二进制改名（对齐 Go `BinaryRename`；当前插件未用，保留契约）。
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct BinaryRename {
    #[serde(default)]
    pub name_flag: String,
    #[serde(default)]
    pub rename_to: String,
}

/// 安装配置（对齐 Go `InstallerConfig`；Lua 全局变量名 `ic`）。
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
    /// 对齐 Go `NewInstallerConfig`：空指针字段置空结构（Lua 端按平台 append）。
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

/// 平台路径选择（对齐 Go `CollectEnvs` 等处的按平台取字段逻辑）。
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

/// installer 类型常量（对齐 Go `lua_global/version.go`）。
pub mod installer_kind {
    pub const CONDA: &str = "conda";
    pub const CONDA_FORGE: &str = "conda-forge";
    pub const COURSIER: &str = "coursier";
    pub const UNARCHIVER: &str = "unarchiver";
    pub const EXECUTABLE: &str = "executable";
    pub const DPKG: &str = "dpkg";
    pub const RPM: &str = "rpm";
}
