//! Network policy: reverse-proxy decision and download thread count.
//!
//! Priority chain (plan.md §3.3): gitee gets neither a reverse proxy nor a local proxy, and a
//! non-empty local proxy disables the reverse proxy; otherwise `VMR_REVERSE_PROXY` or the
//! GitHub default applies. vmr-core only makes the policy decision; the actual requests
//! happen in vmr-net.

use std::env;

use crate::default_reverse_proxy;
use crate::envs;

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
