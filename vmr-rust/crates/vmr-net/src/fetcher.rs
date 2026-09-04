//! reqwest wrapper with the mirror / reverse-proxy / proxy priority chain,
//! mirroring Go `cnf/common.go GetFetcher`.
//!
//! Priority chain (plan.md §3.3, description.md §3.4):
//! 1. Mirror substitution (only when `VMR_USE_CUSTOMED_MIRRORS` is on; goes through vmr-core).
//! 2. Local proxy `VMR_LOCAL_PROXY` → reverse-proxy decision: local proxy non-empty or
//!    gitee → no reverse proxy; non-empty `VMR_REVERSE_PROXY` → use it; otherwise a URL
//!    containing `github` → default reverse proxy `https://proxy.vmr.dpdns.org/proxy/`.
//! 3. Only when the reverse proxy is non-empty **and the mirror left the URL unchanged**:
//!    `url = reverse-proxy prefix + url` (the prefix already ends in `/`; the real sample
//!    is `proxy/https://…`, with no extra `/`).
//! 4. Proxy: the local proxy is used only when the URL is **not gitee and the mirror left
//!    the URL unchanged**; when the local proxy is empty, it falls back to
//!    `GVC_DEFAULT_PROXY` (goutils behavior).
//!
//! Client defaults mirror Go/goutils: no custom UA, no retry, no timeout; supports
//! http/https/socks5 proxies.

use std::collections::HashMap;

use vmr_core::default_reverse_proxy;
use vmr_core::envs;
use vmr_core::{apply_customed_mirror, load_customed_mirror};

/// Reverse-proxy decision (mirrors Go `GetReverseProxyUri`, pure-parameter version):
/// non-empty local proxy or URL containing `gitee.com` → empty; non-empty
/// `reverse_proxy_env` → use it; otherwise a URL containing the `github` substring
/// → the default reverse proxy; the result is padded with a trailing `/`.
fn reverse_proxy_for(url: &str, local_proxy: &str, rp_env: &str) -> String {
    if !local_proxy.is_empty() || url.contains("gitee.com") {
        return String::new();
    }
    let mut rp = if rp_env.is_empty() {
        if url.contains("github") {
            default_reverse_proxy()
        } else {
            String::new()
        }
    } else {
        rp_env.to_string()
    };
    if !rp.is_empty() && !rp.ends_with('/') {
        rp.push('/');
    }
    rp
}

/// Pure-function core of the proxy-chain resolution (env and conf are passed in as
/// parameters, for easier unit testing).
///
/// Returns `(request URL, proxy to set)`. `mirror_on` indicates the mirror switch is
/// on; `mirrors` is the mirror table (an empty table is equivalent to no substitution).
pub fn chain_url(
    d_url: &str,
    mirror_on: bool,
    mirrors: &HashMap<String, String>,
    local_proxy: &str,
    reverse_proxy_env: &str,
    gvc_proxy: &str,
) -> (String, Option<String>) {
    // 1. Mirror substitution.
    let mut url = d_url.to_string();
    if mirror_on {
        url = apply_customed_mirror(&url, mirrors);
    }
    let mirror_changed = url != d_url;

    // 2. Reverse-proxy decision (made on the URL after mirroring).
    let rp = reverse_proxy_for(&url, local_proxy, reverse_proxy_env);

    // 3. Prepend the reverse proxy (only when the mirror left the URL unchanged).
    let mut proxy: Option<String> = None;
    if !rp.is_empty() && !mirror_changed {
        url = format!("{rp}{url}");
    } else if !rp.is_empty() {
        // The mirror changed the URL: keeping the local policy identical to Go — Go only
        // prepends when the mirror left the URL unchanged, so here, since the mirror
        // changed it, the reverse proxy is no longer prepended.
    }

    // 4. Proxy (only when not gitee and the mirror left the URL unchanged).
    if !url.contains("gitee.com") && !mirror_changed {
        if !local_proxy.is_empty() {
            proxy = Some(local_proxy.to_string());
        } else if !gvc_proxy.is_empty() {
            proxy = Some(gvc_proxy.to_string());
        }
    }
    (url, proxy)
}

/// Reads the current values from the env and config, then runs the priority chain.
pub fn resolve_chain(d_url: &str) -> (String, Option<String>) {
    let mirror_on = std::env::var(envs::USE_CUSTOMED_MIRRORS)
        .map(|v| matches!(v.to_lowercase().as_str(), "true" | "1" | "t" | "yes" | "on"))
        .unwrap_or(false);
    let mirrors = if mirror_on {
        load_customed_mirror()
    } else {
        HashMap::new()
    };
    let local_proxy = std::env::var(envs::LOCAL_PROXY).unwrap_or_default();
    let rp_env = std::env::var(envs::REVERSE_PROXY).unwrap_or_default();
    let gvc_proxy = std::env::var(envs::GVC_DEFAULT_PROXY).unwrap_or_default();
    chain_url(
        d_url,
        mirror_on,
        &mirrors,
        &local_proxy,
        &rp_env,
        &gvc_proxy,
    )
}

/// Request/download wrapper; each instance binds the resolved proxy (the env is the
/// runtime authority).
pub struct Fetcher {
    client: reqwest::blocking::Client,
}

impl Fetcher {
    /// Builds the client: applies the proxy (direct connection when there is none).
    /// No UA, no timeout, no retry (Go's default behavior).
    pub fn new(proxy: Option<String>) -> reqwest::Result<Self> {
        let mut builder = reqwest::blocking::Client::builder()
            .user_agent("")
            .no_proxy();
        if let Some(p) = proxy {
            // reqwest Proxy::all supports http/https/socks5 (socks feature).
            builder = builder.proxy(reqwest::Proxy::all(p)?);
        }
        Ok(Fetcher {
            client: builder.build()?,
        })
    }

    /// Builds the client after resolving the chain for the target URL (use when the
    /// proxy depends on the URL).
    pub fn for_url(d_url: &str) -> reqwest::Result<Self> {
        let (_url, proxy) = resolve_chain(d_url);
        Self::new(proxy)
    }

    pub fn client(&self) -> &reqwest::blocking::Client {
        &self.client
    }

    /// GETs text (the default UA is an empty string; callers may override the header
    /// when the server does not accept an empty UA).
    pub fn get(&self, url: &str) -> reqwest::Result<String> {
        self.client.get(url).send()?.error_for_status()?.text()
    }

    /// GETs bytes.
    pub fn get_bytes(&self, url: &str) -> reqwest::Result<Vec<u8>> {
        self.client
            .get(url)
            .send()?
            .error_for_status()?
            .bytes()
            .map(|b| b.to_vec())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mirrors() -> HashMap<String, String> {
        let mut m = HashMap::new();
        m.insert(
            "github.com".to_string(),
            "gh-mirror.example.com".to_string(),
        );
        m
    }

    const LOCAL: &str = "http://127.0.0.1:7890";
    const GVC: &str = "http://gvc:8080";
    const RP: &str = ""; // empty env → default reverse proxy

    #[test]
    fn github_gets_default_reverse_proxy_when_no_local_proxy() {
        let (url, proxy) = chain_url(
            "https://github.com/a/b/releases/download/1.0/x.zip",
            false,
            &HashMap::new(),
            "",
            RP,
            "",
        );
        assert!(
            url.starts_with("https://proxy.vmr.dpdns.org/proxy/https://github.com/"),
            "{url}"
        );
        assert_eq!(proxy, None);
    }

    #[test]
    fn local_proxy_disables_reverse_proxy_but_applies_as_proxy() {
        let (url, proxy) = chain_url(
            "https://github.com/a/b/releases/download/1.0/x.zip",
            false,
            &HashMap::new(),
            LOCAL,
            RP,
            "",
        );
        assert_eq!(url, "https://github.com/a/b/releases/download/1.0/x.zip");
        assert_eq!(proxy.as_deref(), Some(LOCAL));
    }

    #[test]
    fn gitee_no_reverse_no_proxy() {
        let (url, proxy) = chain_url(
            "https://gitee.com/a/b/releases/download/1.0/x.zip",
            false,
            &HashMap::new(),
            LOCAL,
            RP,
            GVC,
        );
        assert_eq!(url, "https://gitee.com/a/b/releases/download/1.0/x.zip");
        assert_eq!(proxy, None, "gitee 不用本地代理（含 GVC 回退）");
    }

    #[test]
    fn gvc_proxy_fallback_when_no_local() {
        let (url, proxy) = chain_url(
            "https://objects.githubusercontent.com/x/y",
            false,
            &HashMap::new(),
            "",
            RP,
            GVC,
        );
        // Not gitee and no local proxy → the GVC fallback applies; since the URL contains
        // the github substring, the default reverse proxy is used for the decision.
        assert_eq!(proxy.as_deref(), Some(GVC));
        assert!(
            url.contains("proxy.vmr.dpdns.org"),
            "含 github 子串 → 默认反代: {url}"
        );
    }

    #[test]
    fn mirror_replacement_skips_reverse_proxy_and_local_proxy() {
        let (url, proxy) = chain_url(
            "https://github.com/a/b/releases/download/1.0/x.zip",
            true,
            &mirrors(),
            LOCAL,
            RP,
            "",
        );
        assert_eq!(
            url,
            "https://gh-mirror.example.com/a/b/releases/download/1.0/x.zip"
        );
        assert_eq!(proxy, None, "镜像已改 URL → 不用本地代理/反代");
    }

    #[test]
    fn mirror_not_matching_keeps_normal_rules() {
        // Mirror on but no entry in the table matches → URL unchanged → the normal rules
        // apply: the local proxy takes effect and disables the reverse proxy.
        let (url, proxy) = chain_url(
            "https://github.com/a/b.zip",
            true,
            &HashMap::new(),
            LOCAL,
            RP,
            "",
        );
        assert_eq!(url, "https://github.com/a/b.zip");
        assert_eq!(proxy, Some(LOCAL.into()), "镜像未改 URL → 本地代理仍生效");
    }

    #[test]
    fn custom_reverse_proxy_env_wins() {
        let (url, _) = chain_url(
            "https://github.com/a/b.zip",
            false,
            &HashMap::new(),
            "",
            "https://rp.example.com/prefix/",
            "",
        );
        assert!(
            url.starts_with("https://rp.example.com/prefix/https://github.com/"),
            "{url}"
        );
    }
}
