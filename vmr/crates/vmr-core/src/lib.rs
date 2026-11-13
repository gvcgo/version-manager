//! vmr-core: path conventions, config read/write, and network policy.
//!
//! Mirrors the behavior and disk/env-var contracts of `vmr-go/internal/cnf` (see plan.md §3.1, §4).
//! This crate is a network-free leaf with no internal vmr dependencies and makes no network
//! requests.

pub mod conf;
pub mod names;
pub mod paths;
pub mod policy;

pub use conf::VMRConf;
pub use names::*;
pub use paths::*;
pub use policy::*;

pub const VMR_WORK_DIR_NAME: &str = ".vmr";
pub const DEFAULT_DOMAIN: &str = "vmr.dpdns.org";
pub const DEFAULT_HOST_URL: &str = "https://raw.githubusercontent.com/gvcgo/vsources/main";

/// Default reverse-proxy prefix, mirroring Go `DefaultReverseProxy`.
pub fn default_reverse_proxy() -> String {
    format!("https://proxy.{DEFAULT_DOMAIN}/proxy/")
}

/// Environment variable name contract (the full plan.md §4.3 set).
///
/// Go's const is named `VMRDonwloadThreadEnv` (the Donwload spelling quirk);
/// the actual env name `VMR_DOWNLOAD_THREADS` is correct, so the env name wins here.
pub mod envs {
    pub const SDK_INSTALLATION_DIR: &str = "VMR_SDK_INSTALLATION_DIR";
    pub const HOST_URL: &str = "VMR_HOST";
    pub const REVERSE_PROXY: &str = "VMR_REVERSE_PROXY";
    pub const LOCAL_PROXY: &str = "VMR_LOCAL_PROXY";
    pub const DOWNLOAD_THREADS: &str = "VMR_DOWNLOAD_THREADS";
    pub const ALLOW_NESTED_SESSIONS: &str = "VMR_ALLOW_NESTED_SESSIONS";
    pub const VM_DISABLE: &str = "VM_DISABLE";
    pub const ADD_TO_PATH_TEMPORARILY: &str = "VMR_ADD_TO_PATH_TEMPORARILY";
    pub const CDHOOK_ACTIVE: &str = "VMR_CDHOOK_ACTIVE";
    pub const LOCKFILE_DIR: &str = "VMR_LOCKFILE_DIR";
    /// Project root of the active `.vmr.lock` child shell (set by `vmr cdhook`); the
    /// session hook leaves that shell when `cd` goes outside the directory.
    pub const SESSION_PROJECT: &str = "VMR_SESSION_PROJECT";
    /// Set by the `.vmr.lock` session shell after it installs its hook, so subshells started
    /// from it do not install a second hook (they cannot end the outer session anyway).
    pub const SESSION_ACTIVE: &str = "VMR_SESSION_ACTIVE";
    pub const VERSIONS: &str = "VMR_VERSIONS";
    pub const PIXI_CHANNEL: &str = "VMR_PIXI_CHANNEL";
    pub const CONDA_CHANNEL: &str = "VMR_CONDA_CHANNEL";
    pub const COURSIER_PATH: &str = "VMR_COURSIER_PATH";
    pub const MCP_SERVER: &str = "VMR_MCP_SERVER";
    pub const GVC_DEFAULT_PROXY: &str = "GVC_DEFAULT_PROXY";
    pub const VCOLLECTOR_PROXY: &str = "VCOLLECTOR_PROXY";
    /// GitHub API token, preferred over `GITHUB_TOKEN` and under `conf.toml`'s `GithubToken`.
    pub const VMR_GITHUB_TOKEN: &str = "VMR_GITHUB_TOKEN";
    pub const GITHUB_TOKEN: &str = "GITHUB_TOKEN";

    /// Mirrors Go `os.Setenv`: process-level environment variable writes.
    ///
    /// # Safety
    /// Call this only during the single-threaded startup phase (when vmr-core writes conf
    /// back to env), so it never races with other threads' `env::var` calls.
    pub fn set(key: &str, value: &str) {
        unsafe { std::env::set_var(key, value) };
    }
}
