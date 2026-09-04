//! 网络策略：反代判定、镜像替换、下载线程数。
//!
//! 优先级链（plan.md §3.3）：镜像替换先于反代；仅镜像未改 URL 才叠加反代；
//! gitee 不反代也不用本地代理。vmr-core 只提供策略判定，实际请求在 vmr-net。

use std::cmp::Reverse;
use std::collections::HashMap;
use std::env;
use std::fs;

use crate::default_reverse_proxy;
use crate::envs;
use crate::paths;

/// 反代判定（对齐 Go `GetReverseProxyUri`）：
/// 本地代理非空或 URL 含 `gitee.com` → 不反代；env 未设反代且 URL 含 `github`
/// 子串 → 默认反代；结果统一补尾部 `/`。
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

/// 下载线程数（对齐 Go `GetDownloadThreadNum`）：env 解析失败或 <1 时取 1。
pub fn get_download_thread_num() -> i32 {
    let num = env::var(envs::DOWNLOAD_THREADS)
        .ok()
        .and_then(|s| s.parse::<i32>().ok())
        .unwrap_or(0);
    if num < 1 { 1 } else { num }
}

/// 读取镜像表；文件缺失或解析失败 → 空表（下载补全由 vmr-net 负责）。
pub fn load_customed_mirror() -> HashMap<String, String> {
    match fs::read_to_string(paths::customed_mirrors_file_path()) {
        Ok(content) => toml::from_str(&content).unwrap_or_default(),
        Err(_) => HashMap::new(),
    }
}

/// 镜像替换核心（纯函数）。键按长度降序匹配，保证具体域名先于通用域名；
/// Go 侧 map 迭代序随机，这里做确定性化。gradle 分支对齐 Go：
/// URL 以 `https://gradle.org/releases` 开头且镜像值含 `%s` 时，
/// 用 query 参数 `version` 填充；缺失则原样返回。
pub fn apply_customed_mirror(d_url: &str, mirrors: &HashMap<String, String>) -> String {
    let mut keys: Vec<&String> = mirrors.keys().collect();
    keys.sort_by_key(|k| Reverse(k.len()));
    let mut result = d_url.to_string();
    for k in keys {
        let v = &mirrors[k];
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

/// 提取 gradle releases URL 的 query 参数 `version`（不做百分号解码，版本号不含特殊字符）。
fn gradle_version(url: &str) -> Option<String> {
    let query = url.split_once('?')?.1;
    let query = query.split('#').next().unwrap_or(query);
    query.split('&').find_map(|pair| {
        let (k, v) = pair.split_once('=')?;
        (k == "version" && !v.is_empty()).then(|| v.to_string())
    })
}

/// 镜像替换入口（对齐 Go `UseCustomedMirrorUrl`）：env 开关未开则原样返回。
pub fn use_customed_mirror_url(d_url: &str) -> String {
    if !env_bool(envs::USE_CUSTOMED_MIRRORS) {
        return d_url.to_string();
    }
    let mirrors = load_customed_mirror();
    apply_customed_mirror(d_url, &mirrors)
}

/// 宽松布尔 env 解析（对齐 Go gconv.Bool 的常见取值）。
fn env_bool(key: &str) -> bool {
    match env::var(key) {
        Ok(v) => matches!(v.to_lowercase().as_str(), "true" | "1" | "t" | "yes" | "on"),
        Err(_) => false,
    }
}
