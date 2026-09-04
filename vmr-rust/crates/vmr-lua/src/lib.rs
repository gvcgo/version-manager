//! vmr-lua：插件系统（plan.md §3.4，要求 1/3/4 核心）。
//!
//! - `bindings`：mlua(lua54) 运行时初始化与全部 `vmr*` 注册 + installer 常量
//!   （修复 Go 侧 vmrInstaller* 常量未注册的缺口）。
//! - `req/html/json/utils_bind/version/github_bind/installer_conf/conda_bridge`：
//!   50 个 vmr* 全局函数绑定（同名同语义）。
//! - `plugin`：插件生命周期——加载/crawl/平台过滤/版本缓存/ic/自定义安装回调。
//! - `plugins_update`：插件目录缺失自动更新（gvcgo/vmr_plugins）。
//! - `types`：Item/VersionList/InstallerConfig 磁盘契约。

pub mod bindings;
pub mod conda_bridge;
pub mod github_bind;
pub mod html;
pub mod installer_conf;
pub mod json;
pub mod plugin;
pub mod plugins_update;
pub mod req;
pub mod types;
pub mod utils_bind;
pub mod version;

pub use bindings::new_runtime;
pub use plugin::{Plugin, Plugins, run_content_crawl};
pub use types::{
    AdditionalEnv, DirItems, FileItems, InstallerConfig, Item, SDKVersion, VersionList,
};
