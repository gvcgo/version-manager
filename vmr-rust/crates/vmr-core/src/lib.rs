//! vmr-core：路径约定、配置读写与网络策略。
//!
//! 对齐 `vmr-go/internal/cnf` 的行为与磁盘/环境变量契约（见 plan.md §3.1、§4）。
//! 本 crate 是无 vmr 内部依赖的叶子，不含任何网络请求。

pub mod conf;
pub mod paths;
pub mod policy;

pub use conf::VMRConf;
pub use paths::*;
pub use policy::*;

pub const VMR_WORK_DIR_NAME: &str = ".vmr";
pub const DEFAULT_DOMAIN: &str = "vmr.dpdns.org";
pub const DEFAULT_HOST_URL: &str = "https://raw.githubusercontent.com/gvcgo/vsources/main";

/// 默认反代前缀，对齐 Go `DefaultReverseProxy`。
pub fn default_reverse_proxy() -> String {
    format!("https://proxy.{DEFAULT_DOMAIN}/proxy/")
}

/// 环境变量名契约（plan.md §4.3 全量）。
///
/// Go 侧 const 名为 `VMRDonwloadThreadEnv`（Donwload 拼写 quirk），
/// 实际 env 名 `VMR_DOWNLOAD_THREADS` 是正确的，此处以 env 名为准。
pub mod envs {
    pub const SDK_INSTALLATION_DIR: &str = "VMR_SDK_INSTALLATION_DIR";
    pub const HOST_URL: &str = "VMR_HOST";
    pub const REVERSE_PROXY: &str = "VMR_REVERSE_PROXY";
    pub const LOCAL_PROXY: &str = "VMR_LOCAL_PROXY";
    pub const DOWNLOAD_THREADS: &str = "VMR_DOWNLOAD_THREADS";
    pub const USE_CUSTOMED_MIRRORS: &str = "VMR_USE_CUSTOMED_MIRRORS";
    pub const ALLOW_NESTED_SESSIONS: &str = "VMR_ALLOW_NESTED_SESSIONS";
    pub const VM_DISABLE: &str = "VM_DISABLE";
    pub const ADD_TO_PATH_TEMPORARILY: &str = "VMR_ADD_TO_PATH_TEMPORARILY";
    pub const CD_INIT: &str = "VMR_CD_INIT";
    pub const VERSIONS: &str = "VMR_VERSIONS";
    pub const GVC_DEFAULT_PROXY: &str = "GVC_DEFAULT_PROXY";
    pub const VCOLLECTOR_PROXY: &str = "VCOLLECTOR_PROXY";

    /// 对齐 Go `os.Setenv`：进程级环境变量写入。
    ///
    /// # Safety
    /// 仅在进程启动的单线程阶段调用（vmr-core 的 conf 回写时机），
    /// 不会与其它线程的 `env::var` 并发。
    pub fn set(key: &str, value: &str) {
        unsafe { std::env::set_var(key, value) };
    }
}
