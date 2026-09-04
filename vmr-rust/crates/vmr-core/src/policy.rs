//! Network policy: reverse-proxy decision, mirror substitution, and download thread count.
//!
//! Priority chain (plan.md §3.3): mirror substitution runs before the reverse proxy; the
//! reverse proxy is applied only when the mirror left the URL unchanged; gitee gets neither a
//! reverse proxy nor a local proxy. vmr-core only makes the policy decision; the actual
//! requests happen in vmr-net.

use std::collections::HashMap;

use std::env;
use std::fs;

use crate::default_reverse_proxy;
use crate::envs;
use crate::paths;

/// Reverse-proxy decision (mirrors Go `GetReverseProxyUri`):
/// a non-empty local proxy or a URL containing `gitee.com` → no reverse proxy; no reverse
/// proxy set in env and the URL contains the `github` substring → default reverse proxy;
/// a trailing `/` is always appended to the result.
pub fn get_reverse_proxy_uri(d_url: &str, local_proxy: &str) -> String {
    if !local_proxy.is_empty() {
        return String::new();
    }
    if d_url.contains("gitee.com") {
        return String::new();
    }
    let mut rp = env::var(envs::REVERSE_PROXY).unwrap_or_default();
    if rp.is_empty() && d_url.contains("github") {
        rp = default_reverse_proxy();
    }
    if !rp.ends_with('/') {
        rp.push('/');
    }
    rp
}

/// Download thread count (mirrors Go `GetDownloadThreadNum`): 1 when env parsing fails or
/// the value is <1.
pub fn get_download_thread_num() -> i32 {
    let num = env::var(envs::DOWNLOAD_THREADS)
        .ok()
        .and_then(|s| s.parse::<i32>().ok())
        .unwrap_or(0);
    if num < 1 { 1 } else { num }
}

/// Reads the mirror table; a missing file or a parse failure → an empty table (fetching the
/// table is handled by vmr-net).
pub fn load_customed_mirror() -> HashMap<String, String> {
    match fs::read_to_string(paths::customed_mirrors_file_path()) {
        Ok(content) => toml::from_str(&content).unwrap_or_default(),
        Err(_) => HashMap::new(),
    }
}

/// Mirror-substitution core (pure function). Keys are matched in descending length order so
/// specific domains win over generic ones; Go's map iteration order is random, so matching is
/// made deterministic here. The gradle branch mirrors Go: when the URL starts with
/// `https://gradle.org/releases` and the mirror value contains `%s`, the `version` query
/// parameter is substituted in; when absent, the URL is returned unchanged.
pub fn apply_customed_mirror(d_url: &str, mirrors: &HashMap<String, String>) -> String {
    // Keys are matched in descending length order (ties broken by key order, deterministic);
    // iterate key/value pairs rather than indices.
    let mut entries: Vec<(&String, &String)> = mirrors.iter().collect();
    entries.sort_by(|a, b| b.0.len().cmp(&a.0.len()).then_with(|| a.0.cmp(b.0)));
    let mut result = d_url.to_string();
    for (k, v) in entries {
        if !result.contains(k.as_str()) {
            continue;
        }
        if result.starts_with("https://gradle.org/releases") && v.contains("%s") {
            return match gradle_version(&result) {
                Some(version) => v.replace("%s", &version),
                None => result,
            };
        }
        result = result.replace(k.as_str(), v.as_str());
    }
    result
}

/// Extracts the `version` query parameter from a gradle releases URL (no percent-decoding;
/// version numbers contain no special characters).
fn gradle_version(url: &str) -> Option<String> {
    let query = url.split_once('?')?.1;
    let query = query.split('#').next().unwrap_or(query);
    query.split('&').find_map(|pair| {
        let (k, v) = pair.split_once('=')?;
        (k == "version" && !v.is_empty()).then(|| v.to_string())
    })
}

/// Mirror-substitution entry point (mirrors Go `UseCustomedMirrorUrl`): the URL is returned
/// unchanged when the env switch is off.
pub fn use_customed_mirror_url(d_url: &str) -> String {
    if !env_bool(envs::USE_CUSTOMED_MIRRORS) {
        return d_url.to_string();
    }
    let mirrors = load_customed_mirror();
    apply_customed_mirror(d_url, &mirrors)
}

/// Lenient boolean env parsing (mirrors the accepted values of Go's gconv.Bool).
fn env_bool(key: &str) -> bool {
    match env::var(key) {
        Ok(v) => matches!(v.to_lowercase().as_str(), "true" | "1" | "t" | "yes" | "on"),
        Err(_) => false,
    }
}
