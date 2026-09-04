//! GitHub REST API client (plan.md §3.3 requirement 3; mirrors Go `luapi/gh/gh.go`).
//!
//! - `releases(repo)`: paginates `/repos/{repo}/releases?per_page=100&page=N` until an
//!   empty page or a page with fewer than `per_page` entries.
//! - `file_list(repo, path)`: lists the directory files via the contents API.
//! - token: a configured `GithubToken` wins; otherwise no header is sent (Go's built-in
//!   read-only token is a private credential not shipped with the source; anonymous
//!   access gets a lower rate limit but still works).
//! - The API domain `https://api.github.com` is contacted directly, not via the mirror /
//!   reverse proxy.

use reqwest::blocking::Client;
use reqwest::header::{ACCEPT, AUTHORIZATION};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use vmr_core::conf::get_github_token;

const API_BASE: &str = "https://api.github.com";
const PER_PAGE: usize = 100;

/// A GitHub release asset.
#[derive(Debug, Clone, Deserialize)]
pub struct Asset {
    pub name: String,
    #[serde(rename = "browser_download_url")]
    pub url: String,
    pub size: Option<i64>,
}

/// A single release.
#[derive(Debug, Clone, Deserialize)]
pub struct ReleaseItem {
    #[serde(rename = "tag_name")]
    pub tag: String,
    pub assets: Vec<Asset>,
}

/// A contents API entry (file or directory).
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

/// GitHub API client.
pub struct Gh {
    client: Client,
}

impl Gh {
    /// Builds a direct-connection client (no custom timeout/retry; the UA is fixed to a
    /// stable identifier, which GitHub mandates).
    pub fn new() -> Result<Self, String> {
        let client = reqwest::blocking::Client::builder()
            .user_agent("vmr-rust")
            .build()
            .map_err(|e| e.to_string())?;
        Ok(Gh { client })
    }

    /// Paginated fetch of all releases of a repo (mirrors Go `Gh.GetReleases`).
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

    /// Lists the directory entries via the contents API (mirrors Go `Gh.GetFileList`).
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
