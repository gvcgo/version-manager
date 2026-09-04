//! SDK 差异化后处理（对齐 Go `internal/installer/post/` 注册表；精简实现）。
//!
//! 每项真实行为（linux/macOS）：
//! - zig / upx / moonbit / bun：安装根内浅层找同名可执行 → chmod +x；
//!   bun 额外在 bin 目录补 `bunx` 链接。
//! - rustup：rustup-init 复制/查找并 chmod（Go 的 Library/bin 复制为
//!   rustup 自身运行期行为，此处只保证可执行位）。
//!
//! 未列出的插件 no-op（Ok）。

use std::fs;
use std::path::{Path, PathBuf};

use vmr_lua::types::Item;

const EXEC_PLUGINS: &[&str] = &["zig", "upx", "moonbit", "bun", "rustup", "clojure"];

fn shallow_find(root: &Path, name: &str, depth: usize) -> Option<PathBuf> {
    if depth == 0 {
        return None;
    }
    let entries = fs::read_dir(root).ok()?;
    let mut dirs = Vec::new();
    for e in entries.flatten() {
        let p = e.path();
        let fname = e.file_name().to_string_lossy().into_owned();
        let is_dir = e.file_type().map(|t| t.is_dir()).unwrap_or(false);
        if !is_dir && fname == name {
            return Some(p);
        }
        if is_dir {
            dirs.push(p);
        }
    }
    for d in dirs {
        if let Some(found) = shallow_find(&d, name, depth - 1) {
            return Some(found);
        }
    }
    None
}

#[cfg(unix)]
fn chmod_x(path: &Path) {
    use std::os::unix::fs::PermissionsExt;
    if let Ok(meta) = fs::metadata(path) {
        let mode = meta.permissions().mode();
        let _ = fs::set_permissions(path, fs::Permissions::from_mode(mode | 0o111));
    }
}

#[cfg(not(unix))]
fn chmod_x(_path: &Path) {}

fn add_bunx(bin_dir: &Path) {
    // bun 安装根含 bin/bun；补 bunx 符号链接（unix）。
    let bunx = bin_dir.join("bunx");
    if bunx.exists() {
        return;
    }
    let bun = bin_dir.join("bun");
    if bun.exists() {
        #[cfg(unix)]
        let _ = std::os::unix::fs::symlink("bun", &bunx);
    }
}

/// 执行后处理（安装根 = 版本目录）。
pub fn run_post_install(
    plugin_name: &str,
    install_root: &Path,
    _item: &Item,
) -> Result<(), String> {
    if EXEC_PLUGINS.contains(&plugin_name) {
        if let Some(found) = shallow_find(install_root, plugin_name, 3) {
            chmod_x(&found);
        }
    }
    if plugin_name == "bun" {
        if let Some(bin) = shallow_find(install_root, "bin", 3) {
            add_bunx(&bin);
        }
    }
    if plugin_name == "rustup" {
        if let Some(init) = shallow_find(install_root, "rustup-init", 3) {
            chmod_x(&init);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn shallow_find_works() {
        let dir = std::env::temp_dir().join(format!("vmr-post-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join("bin")).unwrap();
        fs::write(dir.join("bin/tool"), "x").unwrap();
        assert_eq!(shallow_find(&dir, "tool", 3), Some(dir.join("bin/tool")));
        let _ = fs::remove_dir_all(&dir);
    }
}
