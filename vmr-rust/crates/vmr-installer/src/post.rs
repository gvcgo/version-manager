//! Per-SDK post-processing (mirrors the Go `internal/installer/post/` registry;
//! a lean implementation).
//!
//! Actual behavior per entry (linux/macOS):
//! - zig / upx / moonbit / bun: shallow search under the install root for an executable
//!   of the same name → chmod +x; bun additionally adds a `bunx` link in the bin directory.
//! - rustup: rustup-init copies/finds and chmods (Go's Library/bin copy is rustup's own
//!   runtime behavior; here only the executable bit is ensured).
//!
//! Plugins not listed are a no-op (Ok).

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
    // bun's install root contains bin/bun; add a bunx symlink (unix).
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

/// Runs post-processing (install root = version directory).
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
