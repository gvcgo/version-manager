//! reqwest wrapper with the reverse-proxy / proxy priority chain,
//! mirroring Go `cnf/common.go GetFetcher`.
//!
//! Priority chain (plan.md §3.3, description.md §3.4):
//! 1. Local proxy `VMR_LOCAL_PROXY` → reverse-proxy decision: local proxy non-empty or
//!    gitee → no reverse proxy; non-empty `VMR_REVERSE_PROXY` → use it; otherwise a URL
//!    containing `github` → default reverse proxy `https://proxy.vmr.dpdns.org/proxy/`.
//! 2. Only when the reverse proxy is non-empty: `url = reverse-proxy prefix + url` (the
//!    prefix already ends in `/`; the real sample is `proxy/https://…`, with no extra `/`).
//! 3. Proxy: the local proxy is used only when the URL is **not gitee**; when the local
//!    proxy is empty, it falls back to `GVC_DEFAULT_PROXY` (goutils behavior).
//!
//! Client defaults mirror Go/goutils: no custom UA, no retry, no timeout; supports
//! http/https/socks5 proxies.

use vmr_core::default_reverse_proxy;
use vmr_core::envs;

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
/// Returns `(request URL, proxy to set)`.
pub fn chain_url(
    d_url: &str,
    local_proxy: &str,
    reverse_proxy_env: &str,
    gvc_proxy: &str,
) -> (String, Option<String>) {
    // 1. Reverse-proxy decision.
    let rp = reverse_proxy_for(d_url, local_proxy, reverse_proxy_env);

    // 2. Prepend the reverse proxy.
    let mut url = d_url.to_string();
    if !rp.is_empty() {
        url = format!("{rp}{url}");
    }

    // 3. Proxy (only when not gitee).
    let mut proxy: Option<String> = None;
    if !url.contains("gitee.com") {
        if !local_proxy.is_empty() {
            proxy = Some(local_proxy.to_string());
        } else if !gvc_proxy.is_empty() {
            proxy = Some(gvc_proxy.to_string());
        }
    }
    (url, proxy)
}

/// Reads the current values from the env, then runs the priority chain.
pub fn resolve_chain(d_url: &str) -> (String, Option<String>) {
    let local_proxy = std::env::var(envs::LOCAL_PROXY).unwrap_or_default();
    let rp_env = std::env::var(envs::REVERSE_PROXY).unwrap_or_default();
    let gvc_proxy = std::env::var(envs::GVC_DEFAULT_PROXY).unwrap_or_default();
    chain_url(d_url, &local_proxy, &rp_env, &gvc_proxy)
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
        // Keep reqwest's default environment-proxy support (`HTTP_PROXY`/`HTTPS_PROXY`);
        // an explicitly configured proxy still takes precedence below.
        let mut builder = reqwest::blocking::Client::builder().user_agent("");
        if let Some(p) = proxy {
            // reqwest Proxy::all supports http/https/socks5 (socks feature).
            builder = builder.proxy(reqwest::Proxy::all(p)?);
        }
        Ok(Fetcher {
            client: builder.build()?,
        })
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

    const LOCAL: &str = "http://127.0.0.1:7890";
    const GVC: &str = "http://gvc:8080";
    const RP: &str = ""; // empty env → default reverse proxy

    #[test]
    fn github_gets_default_reverse_proxy_when_no_local_proxy() {
        let (url, proxy) = chain_url(
            "https://github.com/a/b/releases/download/1.0/x.zip",
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
            LOCAL,
            RP,
            GVC,
        );
        assert_eq!(url, "https://gitee.com/a/b/releases/download/1.0/x.zip");
        assert_eq!(proxy, None, "gitee 不用本地代理（含 GVC 回退）");
    }

    #[test]
    fn gvc_proxy_fallback_when_no_local() {
        let (url, proxy) = chain_url("https://objects.githubusercontent.com/x/y", "", RP, GVC);
        // Not gitee and no local proxy → the GVC fallback applies; since the URL contains
        // the github substring, the default reverse proxy is used for the decision.
        assert_eq!(proxy.as_deref(), Some(GVC));
        assert!(
            url.contains("proxy.vmr.dpdns.org"),
            "含 github 子串 → 默认反代: {url}"
        );
    }

    #[test]
    fn custom_reverse_proxy_env_wins() {
        let (url, _) = chain_url(
            "https://github.com/a/b.zip",
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
