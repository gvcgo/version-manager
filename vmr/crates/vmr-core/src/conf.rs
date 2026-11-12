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
        if !v.reverse_proxy.is_empty() {
            envs::set(envs::REVERSE_PROXY, &v.reverse_proxy);
        }
        if v.allow_nested_sessions {
            envs::set(envs::ALLOW_NESTED_SESSIONS, "true");
        }
        v
    }

    /// Reloads from disk; a missing file or a parse failure keeps the current values
    /// (mirrors Go ignoring the error). Reads `~/.vmr/conf.toml`, falling back to the XDG
    /// location when the primary file does not exist.
    pub fn reload(&mut self) {
        if let Some(path) = paths::conf_file_path()
            && let Ok(content) = fs::read_to_string(path)
            && let Ok(v) = toml::from_str::<VMRConf>(&content)
        {
            *self = v;
        }
    }

    /// Writes conf.toml back to disk: to the file [`reload`](Self::reload) would read, else
    /// `~/.vmr/conf.toml`.
    pub fn save(&self) -> std::io::Result<()> {
        let content = toml::to_string(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        let path = paths::conf_file_write_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, content)
    }

    // The setters below mirror Go semantics: reload the file first, then modify and save
    // (env write-back happens only in `new()`).

    pub fn set_version_host_url(&mut self, url: &str) {
        if url.is_empty() {
            return;
        }
        self.reload();
        self.version_host_url = url.to_string();
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

/// Top-level conf.toml keys understood by [`VMRConf`] (PascalCase, matching serde).
const CONF_KEYS: [&str; 9] = [
    "ProxyUri",
    "ReverseProxy",
    "SDKIntallationDir",
    "VersionHostUrl",
    "ThreadNum",
    "AllowNestedSessions",
    "GithubToken",
    "CacheRetentionTime",
    "DisableCache",
];

/// Reads and parses `conf.toml` without writing anything back to the environment.
///
/// `VMRConf::new` silently falls back to defaults on a parse error, so diagnostics need a
/// separate entry point that reports the error.
pub fn read_conf_file(path: &std::path::Path) -> Result<VMRConf, String> {
    let content = fs::read_to_string(path).map_err(|e| e.to_string())?;
    toml::from_str(&content).map_err(|e| e.to_string())
}

/// Top-level keys in `conf.toml` that [`VMRConf`] does not define (serde ignores them).
pub fn unknown_conf_keys(path: &std::path::Path) -> Result<Vec<String>, String> {
    let content = fs::read_to_string(path).map_err(|e| e.to_string())?;
    let value: toml::Value = toml::from_str(&content).map_err(|e| e.to_string())?;
    let mut out: Vec<String> = value
        .as_table()
        .map(|table| {
            table
                .keys()
                .filter(|key| !CONF_KEYS.contains(&key.as_str()))
                .cloned()
                .collect()
        })
        .unwrap_or_default();
    out.sort();
    Ok(out)
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
