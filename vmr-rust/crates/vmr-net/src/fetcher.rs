//! reqwest 封装与镜像/反代/代理优先级链（对齐 Go `cnf/common.go GetFetcher`）。
//!
//! 优先级链（plan.md §3.3、description.md §3.4）：
//! 1. 镜像替换（仅 `VMR_USE_CUSTOMED_MIRRORS` 开启时，走 vmr-core）。
//! 2. 本地代理 `VMR_LOCAL_PROXY` → 反代判定：本地代理非空或 gitee → 不反代；
//!    `VMR_REVERSE_PROXY` 非空用之，否则 URL 含 `github` → 默认反代
//!    `https://proxy.vmr.dpdns.org/proxy/`。
//! 3. 仅当反代非空**且镜像未改 URL** 时：`url = 反代前缀 + url`
//!    （前缀已补尾 `/`，实际样例为 `proxy/https://…`，不加额外 `/`）。
//! 4. 代理：仅当**非 gitee 且镜像未改 URL** 时使用本地代理；
//!    本地代理为空回退 `GVC_DEFAULT_PROXY`（goutils 行为）。
//!
//! 客户端默认行为对齐 Go/goutils：无自定义 UA、无重试、无超时；支持
//! http/https/socks5 代理。

use std::collections::HashMap;

use vmr_core::default_reverse_proxy;
use vmr_core::envs;
use vmr_core::{apply_customed_mirror, load_customed_mirror};

/// 反代判定（对齐 Go `GetReverseProxyUri`，纯参数版）：
/// 本地代理非空或 URL 含 `gitee.com` → 空；`reverse_proxy_env` 非空用之；
/// 否则 URL 含 `github` 子串 → 默认反代；结果补尾部 `/`。
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

/// 代理链解析的纯函数核心（env 与 conf 已读入参数，便于单测）。
///
/// 返回 `(请求 URL, 应设置的代理)`。`mirror_on` 表示镜像开关开启；
/// `mirrors` 为镜像表（空表等价未替换）。
pub fn chain_url(
    d_url: &str,
    mirror_on: bool,
    mirrors: &HashMap<String, String>,
    local_proxy: &str,
    reverse_proxy_env: &str,
    gvc_proxy: &str,
) -> (String, Option<String>) {
    // 1. 镜像替换。
    let mut url = d_url.to_string();
    if mirror_on {
        url = apply_customed_mirror(&url, mirrors);
    }
    let mirror_changed = url != d_url;

    // 2. 反代判定（对镜像后 URL 判定）。
    let rp = reverse_proxy_for(&url, local_proxy, reverse_proxy_env);

    // 3. 反代叠加（仅镜像未改 URL 时）。
    let mut proxy: Option<String> = None;
    if !rp.is_empty() && !mirror_changed {
        url = format!("{rp}{url}");
    } else if !rp.is_empty() {
        // 镜像已改 URL：反代前缀保留本地策略同 Go——Go 只在镜像未改时叠加，
        // 这里镜像已改则不再叠反代。
    }

    // 4. 代理（仅非 gitee 且镜像未改）。
    if !url.contains("gitee.com") && !mirror_changed {
        if !local_proxy.is_empty() {
            proxy = Some(local_proxy.to_string());
        } else if !gvc_proxy.is_empty() {
            proxy = Some(gvc_proxy.to_string());
        }
    }
    (url, proxy)
}

/// 从环境与配置读取当前值后执行优先级链。
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

/// 请求/下载封装。每个实例绑定解析好的代理（运行时权威 env 决定）。
pub struct Fetcher {
    client: reqwest::blocking::Client,
}

impl Fetcher {
    /// 构建客户端：应用代理（无代理则直连）。无 UA/无超时/无重试（Go 默认行为）。
    pub fn new(proxy: Option<String>) -> reqwest::Result<Self> {
        let mut builder = reqwest::blocking::Client::builder()
            .user_agent("")
            .no_proxy();
        if let Some(p) = proxy {
            // reqwest Proxy::all 支持 http/https/socks5（socks feature）。
            builder = builder.proxy(reqwest::Proxy::all(p)?);
        }
        Ok(Fetcher {
            client: builder.build()?,
        })
    }

    /// 依目标 URL 解析链后构造客户端（代理与 URL 相关时使用）。
    pub fn for_url(d_url: &str) -> reqwest::Result<Self> {
        let (_url, proxy) = resolve_chain(d_url);
        Self::new(proxy)
    }

    pub fn client(&self) -> &reqwest::blocking::Client {
        &self.client
    }

    /// GET 文本（默认 UA 为空串；服务器不接受空 UA 时调用方可覆盖 header）。
    pub fn get(&self, url: &str) -> reqwest::Result<String> {
        self.client.get(url).send()?.error_for_status()?.text()
    }

    /// GET 字节。
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
    const RP: &str = ""; // env 空 → 默认反代

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
        // 非 gitee、无本地代理 → GVC 回退生效；URL 含 github 子串判定用默认反代。
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
        // 镜像开启但表内无匹配 → URL 未变 → 常规规则：本地代理生效、反代被本地代理禁用。
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
