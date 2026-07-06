use std::sync::OnceLock;

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Constants – env var names
// ---------------------------------------------------------------------------

pub const VMR_SDK_INSTALLATION_DIR: &str = "VMR_SDK_INSTALLATION_DIR";
pub const VMR_HOST: &str = "VMR_HOST";
pub const VMR_REVERSE_PROXY: &str = "VMR_REVERSE_PROXY";
pub const VMR_LOCAL_PROXY: &str = "VMR_LOCAL_PROXY";
pub const VMR_DOWNLOAD_THREADS: &str = "VMR_DOWNLOAD_THREADS";
pub const VMR_USE_CUSTOMED_MIRRORS: &str = "VMR_USE_CUSTOMED_MIRRORS";
pub const VMR_ALLOW_NESTED_SESSIONS: &str = "VMR_ALLOW_NESTED_SESSIONS";

// ---------------------------------------------------------------------------
// Constants – URL patterns
// ---------------------------------------------------------------------------

pub const DEFAULT_DOMAIN: &str = "vmr.dpdns.org";
pub const DEFAULT_HOST_URL: &str = "https://raw.githubusercontent.com/gvcgo/vsources/main";
pub const SDK_NAME_LIST_FILE_URL: &str = "/sdk-list.version.json";
pub const VERSION_FILE_URL_PATTERN: &str = "/%s.version.json";
pub const SDK_INSTALLATION_URL_PATTERN: &str = "install/%s.toml";
pub const VMR_WORK_DIR_NAME: &str = ".vmr";

/// Computed default reverse-proxy URL
pub fn default_reverse_proxy() -> String {
    format!("https://proxy.{}/proxy/", DEFAULT_DOMAIN)
}

// ---------------------------------------------------------------------------
// VMRConf – TOML-backed configuration
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize, Serialize, Default, Clone)]
pub struct VMRConf {
    #[serde(default)]
    pub proxy_uri: Option<String>,
    #[serde(default)]
    pub reverse_proxy: Option<String>,
    #[serde(default)]
    pub sdk_installation_dir: Option<String>,
    #[serde(default)]
    pub version_host_url: Option<String>,
    #[serde(default)]
    pub download_thread_num: Option<u32>,
    #[serde(default)]
    pub use_customed_mirrors: Option<bool>,
    #[serde(default)]
    pub allow_nested_sessions: Option<bool>,
    #[serde(default)]
    pub github_token: Option<String>,
    #[serde(default)]
    pub cache_retention_time: Option<i64>,
    #[serde(default)]
    pub disable_cache: Option<bool>,
}

// ---------------------------------------------------------------------------
// Global default config singleton
// ---------------------------------------------------------------------------

static DEFAULT_CONFIG: OnceLock<VMRConf> = OnceLock::new();

/// Returns a reference to the lazily-initialised default VMRConf singleton.
pub fn default_config() -> &'static VMRConf {
    DEFAULT_CONFIG.get_or_init(|| VMRConf::new())
}

// ---------------------------------------------------------------------------
// VMRConf methods
// ---------------------------------------------------------------------------

impl VMRConf {
    /// Creates a new VMRConf – loads from disk and applies env overrides.
    pub fn new() -> Self {
        let mut conf = VMRConf::default();
        conf.load();
        // Apply env vars from loaded config values (mirrors Go init behaviour)
        if let Some(ref dir) = conf.sdk_installation_dir {
            std::env::set_var(VMR_SDK_INSTALLATION_DIR, dir);
        }
        if let Some(ref url) = conf.version_host_url {
            std::env::set_var(VMR_HOST, url.trim_end_matches('/'));
        }
        if let Some(ref proxy) = conf.proxy_uri {
            std::env::set_var(VMR_LOCAL_PROXY, proxy);
        }
        if let Some(num) = conf.download_thread_num {
            if num > 1 {
                std::env::set_var(VMR_DOWNLOAD_THREADS, num.to_string());
            }
        }
        match conf.use_customed_mirrors {
            Some(v) => std::env::set_var(VMR_USE_CUSTOMED_MIRRORS, if v { "true" } else { "false" }),
            None => {}
        }
        if let Some(ref rp) = conf.reverse_proxy {
            std::env::set_var(VMR_REVERSE_PROXY, rp);
        }
        if let Some(true) = conf.allow_nested_sessions {
            std::env::set_var(VMR_ALLOW_NESTED_SESSIONS, "true");
        }
        conf
    }

    /// Reads ~/.vmr/conf.toml and deserialises into self.
    pub fn load(&mut self) {
        let path = crate::paths::get_vmr_conf_file_path();
        if let Ok(content) = std::fs::read_to_string(&path) {
            if let Ok(loaded) = toml::from_str::<VMRConf>(&content) {
                *self = loaded;
            }
        }
    }

    /// Serialises self to ~/.vmr/conf.toml.
    pub fn save(&self) -> std::io::Result<()> {
        let path = crate::paths::get_vmr_conf_file_path();
        let content = toml::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;
        std::fs::write(&path, content)
    }

    // -----------------------------------------------------------------------
    // Setter methods (each does: load → mutate → save)
    // -----------------------------------------------------------------------

    pub fn set_proxy_uri(&mut self, s_uri: &str) {
        if s_uri.is_empty() {
            return;
        }
        self.load();
        self.proxy_uri = Some(s_uri.to_string());
        let _ = self.save();
    }

    pub fn set_reverse_proxy(&mut self, s_uri: &str) {
        if s_uri.is_empty() {
            return;
        }
        self.load();
        self.reverse_proxy = Some(s_uri.to_string());
        let _ = self.save();
    }

    pub fn set_version_host_url(&mut self, h_url: &str) {
        if h_url.is_empty() {
            return;
        }
        self.load();
        self.version_host_url = Some(h_url.to_string());
        let _ = self.save();
    }

    pub fn set_download_thread_num(&mut self, num: u32) {
        self.load();
        self.download_thread_num = Some(if num < 1 { 1 } else { num });
        let _ = self.save();
    }

    pub fn toggle_use_customed_mirrors(&mut self) {
        self.load();
        self.use_customed_mirrors = Some(!self.use_customed_mirrors.unwrap_or(false));
        let _ = self.save();
    }

    /// Returns the new value after toggling.
    pub fn toggle_allow_nested_sessions(&mut self) -> bool {
        self.load();
        let new_val = !self.allow_nested_sessions.unwrap_or(false);
        self.allow_nested_sessions = Some(new_val);
        let _ = self.save();
        new_val
    }

    pub fn set_github_token(&mut self, token: &str) {
        if token.is_empty() {
            return;
        }
        self.load();
        self.github_token = Some(token.to_string());
        let _ = self.save();
    }

    pub fn set_cache_retention_time(&mut self, t: i64) {
        self.load();
        if t > 0 {
            self.cache_retention_time = Some(t);
        }
        let _ = self.save();
    }

    pub fn toggle_cache(&mut self) {
        self.load();
        self.disable_cache = Some(!self.disable_cache.unwrap_or(false));
        let _ = self.save();
    }
}

// ---------------------------------------------------------------------------
// Free functions (ported from common.go)
// ---------------------------------------------------------------------------

/// Returns the URL for the SDK list file.
pub fn get_sdk_list_file_url() -> String {
    let host = std::env::var(VMR_HOST).unwrap_or_else(|_| DEFAULT_HOST_URL.to_string());
    format!("{}{}", host, SDK_NAME_LIST_FILE_URL)
}

/// Returns the URL for a version file by SDK name.
pub fn get_version_file_url_by_sdk_name(sdk_name: &str) -> String {
    let host = std::env::var(VMR_HOST).unwrap_or_else(|_| DEFAULT_HOST_URL.to_string());
    let pattern = VERSION_FILE_URL_PATTERN.replace("%s", sdk_name);
    format!("{}{}", host, pattern)
}

/// Returns the URL for an SDK installation config file by SDK name.
pub fn get_sdk_installation_conf_file_url_by_sdk_name(sdk_name: &str) -> String {
    let host = std::env::var(VMR_HOST).unwrap_or_else(|_| DEFAULT_HOST_URL.to_string());
    let pattern = SDK_INSTALLATION_URL_PATTERN.replace("%s", sdk_name);
    format!("{}/{}", host, pattern)
}

/// Returns the reverse-proxy URI prefix for a download URL.
///
/// Returns an empty string when `local_proxy` is set *or* when the download URL
/// targets gitee.com (which does not need a reverse proxy).
pub fn get_reverse_proxy_uri(d_url: &str, local_proxy: &str) -> String {
    if !local_proxy.is_empty() {
        return String::new();
    }
    if d_url.contains("gitee.com") {
        return String::new();
    }

    let mut rp = std::env::var(VMR_REVERSE_PROXY).unwrap_or_else(|_| {
        if d_url.contains("github") {
            default_reverse_proxy()
        } else {
            String::new()
        }
    });

    if !rp.is_empty() && !rp.ends_with('/') {
        rp.push('/');
    }
    rp
}

/// Returns the number of download threads (minimum 1).
pub fn get_download_thread_num() -> u32 {
    let num = std::env::var(VMR_DOWNLOAD_THREADS)
        .ok()
        .and_then(|s| s.parse::<u32>().ok())
        .unwrap_or(1);
    if num < 1 {
        1
    } else {
        num
    }
}

/// Returns the GitHub token from the persisted config file.
pub fn get_github_token() -> String {
    let mut conf = VMRConf::default();
    conf.load();
    conf.github_token.unwrap_or_default()
}

/// Returns the cache retention time in seconds (default 86400 = 24 h).
pub fn get_cache_retention_time() -> i64 {
    let mut conf = VMRConf::default();
    conf.load();
    conf.cache_retention_time.unwrap_or(86400)
}

/// Returns whether the file cache is disabled.
pub fn get_cache_disabled() -> bool {
    let mut conf = VMRConf::default();
    conf.load();
    conf.disable_cache.unwrap_or(false)
}

/// Placeholder – applies custom mirror rewriting when enabled.
///
/// Full implementation needs an HTTP client (future crate) to fetch mirrors.
pub fn use_customed_mirror_url(d_url: &str) -> String {
    if !std::env::var(VMR_USE_CUSTOMED_MIRRORS)
        .map(|v| v == "true")
        .unwrap_or(false)
    {
        return d_url.to_string();
    }
    // TODO: load mirrors from ~/.vmr/customed_mirrors.toml (needs HTTP fetcher)
    d_url.to_string()
}
