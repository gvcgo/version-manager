//! GitHub REST API 客户端（plan.md §3.3 要求 3；对齐 Go `luapi/gh/gh.go`）。
//!
//! - `releases(repo)`：分页拉 `/repos/{repo}/releases?per_page=100&page=N`，
//!   直到空页或不足一页。
//! - `file_list(repo, path)`：contents API 列目录文件。
//! - token：配置 `GithubToken` 优先；未配置则不带头（Go 内置只读 token 属
//!   私有凭据，不随源码分发；匿名访问限流更低但可工作）。
//! - API 域 `https://api.github.com` 直连，不经镜像/反代。

use reqwest::blocking::Client;
use reqwest::header::{ACCEPT, AUTHORIZATION};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use vmr_core::conf::get_github_token;

const API_BASE: &str = "https://api.github.com";
const PER_PAGE: usize = 100;

/// GitHub release 资产。
#[derive(Debug, Clone, Deserialize)]
pub struct Asset {
    pub name: String,
    #[serde(rename = "browser_download_url")]
    pub url: String,
    pub size: Option<i64>,
}

/// 单个 release。
#[derive(Debug, Clone, Deserialize)]
pub struct ReleaseItem {
    #[serde(rename = "tag_name")]
    pub tag: String,
    pub assets: Vec<Asset>,
}

/// contents API 条目（文件/目录）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoFile {
    pub name: String,
    #[serde(rename = "type")]
    pub kind: String,
    #[serde(default)]
    pub path: String,
    #[serde(rename = "download_url")]
    pub download_url: Option<String>,
}

/// GitHub API 客户端。
pub struct Gh {
    client: Client,
}

impl Gh {
    /// 构造直连客户端（无自定义超时/重试；UA 固定为稳定标识，GitHub 强制要求）。
    pub fn new() -> Result<Self, String> {
        let client = reqwest::blocking::Client::builder()
            .user_agent("vmr-rust")
            .build()
            .map_err(|e| e.to_string())?;
        Ok(Gh { client })
    }

    /// 分页拉取 repo 全部 releases（对齐 Go `Gh.GetReleases`）。
    pub fn releases(&self, repo: &str) -> Result<Vec<ReleaseItem>, String> {
        let mut page = 1usize;
        let mut all = Vec::new();
        loop {
            let url = format!("{API_BASE}/repos/{repo}/releases?per_page={PER_PAGE}&page={page}");
            let batch: Vec<ReleaseItem> = self.get_json(&url)?;
            let n = batch.len();
            all.extend(batch);
            if n < PER_PAGE {
                break;
            }
            page += 1;
        }
        Ok(all)
    }

    /// contents API 列出目录条目（对齐 Go `Gh.GetFileList`）。
    pub fn file_list(&self, repo: &str, path: &str) -> Result<Vec<RepoFile>, String> {
        let path = path.trim_matches('/');
        let url = format!("{API_BASE}/repos/{repo}/contents/{path}");
        self.get_json(&url)
    }

    fn get_json<T: DeserializeOwned>(&self, url: &str) -> Result<T, String> {
        let mut req = self
            .client
            .get(url)
            .header(ACCEPT, "application/vnd.github+json");
        let token = get_github_token();
        if !token.is_empty() {
            req = req.header(AUTHORIZATION, format!("Bearer {token}"));
        }
        let resp = req
            .send()
            .map_err(|e| e.to_string())?
            .error_for_status()
            .map_err(|e| e.to_string())?;
        let bytes = resp
            .bytes()
            .map(|b| b.to_vec())
            .map_err(|e| e.to_string())?;
        serde_json::from_slice(&bytes).map_err(|e| e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_release_json() {
        let json = r#"[{"tag_name":"v1.2.3","assets":[{"name":"x.zip","browser_download_url":"https://github.com/a/b/releases/download/v1.2.3/x.zip","size":123}]}]"#;
        let items: Vec<ReleaseItem> = serde_json::from_str(json).unwrap();
        assert_eq!(items[0].tag, "v1.2.3");
        assert_eq!(items[0].assets[0].name, "x.zip");
        assert_eq!(items[0].assets[0].size, Some(123));
    }

    #[test]
    fn parse_file_list_json() {
        let json =
            r#"[{"name":"go.lua","type":"file","path":"go.lua","download_url":"https://raw..."}]"#;
        let files: Vec<RepoFile> = serde_json::from_str(json).unwrap();
        assert_eq!(files[0].name, "go.lua");
        assert_eq!(files[0].kind, "file");
        assert_eq!(files[0].download_url.as_deref(), Some("https://raw..."));
    }
}
