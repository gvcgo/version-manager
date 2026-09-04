//! vmr-lua: the plugin system (plan.md §3.4, requirements 1/3/4 core).
//!
//! - `bindings`: mlua(lua54) runtime initialization and registration of every `vmr*` function plus installer constants
//!   (fixes the gap where the vmrInstaller* constants were not registered on the Go side).
//! - `req/html/json/utils_bind/version/github_bind/installer_conf/conda_bridge`:
//!   50 vmr* global function bindings (same name, same semantics).
//! - `plugin`: plugin lifecycle — load/crawl/platform filtering/version cache/ic/custom install callbacks.
//! - `plugins_update`: auto-update when the plugin directory is missing (gvcgo/vmr_plugins).
//! - `types`: the Item/VersionList/InstallerConfig disk contracts.

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
