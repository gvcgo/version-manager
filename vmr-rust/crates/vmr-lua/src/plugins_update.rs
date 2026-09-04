//! 插件目录更新（对齐 Go `internal/luapi/plugin/download.go`，要求 1）。
//!
//! `gvcgo/vmr_plugins` main.zip 下载 → 解压到 temp → 以
//! flag(go.lua, LICENSE)+except_dir 找仓库目录 → 复制 `*.lua` 到插件目录 →
//! 用 GitHub contents API 列文件写 `plugins.json` → 清理 temp。

use std::fs;
use std::path::Path;

use vmr_core::paths;
use vmr_net::fetcher::Fetcher;
use vmr_utils::copy::copy_a_file;
use vmr_utils::extract::extract;
use vmr_utils::find_dir::HomeDirFinder;

pub const PLUGINS_DOWNLOAD_URL: &str =
    "https://github.com/gvcgo/vmr_plugins/archive/refs/heads/main.zip";
pub const PLUGIN_REPO: &str = "gvcgo/vmr_plugins";
pub const PLUGIN_INFO_FILE_NAME: &str = "plugins.json";

fn download_zip(dest: &Path) -> Result<(), String> {
    let client = Fetcher::for_url(PLUGINS_DOWNLOAD_URL).map_err(|e| e.to_string())?;
    vmr_net::download_file(
        client.client(),
        PLUGINS_DOWNLOAD_URL,
        dest,
        1,
        None,
        None,
        None,
    )
    .map_err(|e| format!("download plugins failed: {e}"))
}

fn copy_lua_files(repo_dir: &Path) -> Result<(), String> {
    let plugin_dir = paths::plugin_dir();
    fs::create_dir_all(&plugin_dir).map_err(|e| e.to_string())?;
    let entries = fs::read_dir(repo_dir).map_err(|e| e.to_string())?;
    for e in entries.flatten() {
        let name = e.file_name().to_string_lossy().into_owned();
        if e.file_type().map(|t| t.is_dir()).unwrap_or(false) || !name.ends_with(".lua") {
            continue;
        }
        let dst = plugin_dir.join(&name);
        let _ = fs::remove_file(&dst);
        copy_a_file(&e.path(), &dst).map_err(|e| format!("copy {name} failed: {e}"))?;
    }
    Ok(())
}

fn update_info() {
    if let Ok(gh) = vmr_net::github::Gh::new() {
        if let Ok(files) = gh.file_list(PLUGIN_REPO, "") {
            if let Ok(json) = serde_json::to_string(&files) {
                let _ = fs::write(paths::plugin_dir().join(PLUGIN_INFO_FILE_NAME), json);
            }
        }
    }
}

/// 更新插件（幂等；失败返回错误串）。
pub fn update_plugins() -> Result<(), String> {
    let temp = paths::temp_dir();
    fs::create_dir_all(&temp).map_err(|e| e.to_string())?;
    let zip_path = temp.join("vmr_plugins_main.zip");
    let extract_dir = temp.join("vmr_plugins_src");
    let _ = fs::remove_dir_all(&extract_dir);
    fs::create_dir_all(&extract_dir).map_err(|e| e.to_string())?;

    let result = (|| -> Result<(), String> {
        download_zip(&zip_path)?;
        extract(&zip_path, &extract_dir).map_err(|e| e.to_string())?;
        let mut finder = HomeDirFinder::new(vec!["go.lua".to_string(), "LICENSE".to_string()]);
        finder.set_flag_dir_excepted(true);
        finder.find(&extract_dir);
        let repo_dir = finder
            .get_dir_name()
            .ok_or("cannot locate vmr_plugins repo dir")?;
        copy_lua_files(&repo_dir)?;
        update_info();
        Ok(())
    })();
    let _ = fs::remove_file(&zip_path);
    let _ = fs::remove_dir_all(&extract_dir);
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constants_present() {
        assert!(PLUGINS_DOWNLOAD_URL.ends_with(".zip"));
        assert_eq!(PLUGIN_REPO, "gvcgo/vmr_plugins");
    }
}
