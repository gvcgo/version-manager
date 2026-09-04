//! `VMRConf`: conf.toml read/write plus env-var write-back (env is the runtime authority).

use std::fs;

use serde::{Deserialize, Serialize};

use crate::envs;
use crate::paths;

/// conf.toml structure.
///
/// Keys must be PascalCase: Go's struct tags are broken `json,toml:"..."` tags, so the toml
/// library falls back to serializing with field names; the `SDKIntallationDir` spelling quirk
/// is preserved too. Unknown keys are ignored and missing keys take defaults, mirroring Go
/// `toml.Unmarshal` behavior.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct VMRConf {
    pub proxy_uri: String,
    pub reverse_proxy: String,
    #[serde(rename = "SDKIntallationDir")]
    pub sdk_installation_dir: String,
    pub version_host_url: String,
    #[serde(rename = "ThreadNum")]
    pub download_thread_num: i32,
    pub use_customed_mirrors: bool,
    pub allow_nested_sessions: bool,
    pub github_token: String,
    pub cache_retention_time: i64,
    pub disable_cache: bool,
}

impl VMRConf {
    /// Mirrors Go `NewVMRConf`: after reading the file, non-empty fields are written back to
    /// environment variables.
    pub fn new() -> Self {
        let mut v = Self::default();
        v.reload();
        if !v.sdk_installation_dir.is_empty() {
            envs::set(envs::SDK_INSTALLATION_DIR, &v.sdk_installation_dir);
        }
        if !v.version_host_url.is_empty() {
            envs::set(envs::HOST_URL, v.version_host_url.trim_end_matches('/'));
        }
        if !v.proxy_uri.is_empty() {
            envs::set(envs::LOCAL_PROXY, &v.proxy_uri);
        }
        if v.download_thread_num > 1 {
            envs::set(envs::DOWNLOAD_THREADS, &v.download_thread_num.to_string());
        }
        if v.use_customed_mirrors {
            envs::set(envs::USE_CUSTOMED_MIRRORS, "true");
        } else {
            envs::set(envs::USE_CUSTOMED_MIRRORS, "false");
        }
        if !v.reverse_proxy.is_empty() {
            envs::set(envs::REVERSE_PROXY, &v.reverse_proxy);
        }
        if v.allow_nested_sessions {
            envs::set(envs::ALLOW_NESTED_SESSIONS, "true");
        }
        v
    }

    /// Reloads from disk; a missing file or a parse failure keeps the current values
    /// (mirrors Go ignoring the error).
    pub fn reload(&mut self) {
        if let Ok(content) = fs::read_to_string(paths::conf_file_path()) {
            if let Ok(v) = toml::from_str::<VMRConf>(&content) {
                *self = v;
            }
        }
    }

    /// Writes conf.toml back to disk.
    pub fn save(&self) -> std::io::Result<()> {
        let content = toml::to_string(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        fs::write(paths::conf_file_path(), content)
    }

    // The setters below mirror Go semantics: reload the file first, then modify and save
    // (env write-back happens only in `new()`).

    pub fn set_proxy_uri(&mut self, uri: &str) {
        if uri.is_empty() {
            return;
        }
        self.reload();
        self.proxy_uri = uri.to_string();
        let _ = self.save();
    }

    pub fn set_reverse_proxy(&mut self, uri: &str) {
        if uri.is_empty() {
            return;
        }
        self.reload();
        self.reverse_proxy = uri.to_string();
        let _ = self.save();
    }

    pub fn set_version_host_url(&mut self, url: &str) {
        if url.is_empty() {
            return;
        }
        self.reload();
        self.version_host_url = url.to_string();
        let _ = self.save();
    }

    pub fn set_download_thread_num(&mut self, num: i32) {
        self.reload();
        self.download_thread_num = if num < 1 { 1 } else { num };
        let _ = self.save();
    }

    pub fn toggle_use_customed_mirrors(&mut self) {
        self.reload();
        self.use_customed_mirrors = !self.use_customed_mirrors;
        let _ = self.save();
    }

    /// Returns the value after toggling.
    pub fn toggle_allow_nested_sessions(&mut self) -> bool {
        self.reload();
        self.allow_nested_sessions = !self.allow_nested_sessions;
        let _ = self.save();
        self.allow_nested_sessions
    }

    pub fn set_github_token(&mut self, token: &str) {
        self.reload();
        if token.is_empty() {
            return;
        }
        self.github_token = token.to_string();
        let _ = self.save();
    }

    pub fn set_cache_retention_time(&mut self, t: i64) {
        self.reload();
        if t > 0 {
            self.cache_retention_time = t;
        }
        let _ = self.save();
    }

    pub fn toggle_cache(&mut self) {
        self.reload();
        self.disable_cache = !self.disable_cache;
        let _ = self.save();
    }
}

/// Reads the GitHub token (Go `GetGithubToken`).
pub fn get_github_token() -> String {
    VMRConf::new().github_token
}

/// Cache retention time in seconds, 86400 when unset (Go `GetCacheRetentionTime`).
pub fn get_cache_retention_time() -> i64 {
    let t = VMRConf::new().cache_retention_time;
    if t == 0 { 86400 } else { t }
}

/// Whether the cache is disabled (Go `GetCacheDisabled`).
pub fn get_cache_disabled() -> bool {
    VMRConf::new().disable_cache
}
