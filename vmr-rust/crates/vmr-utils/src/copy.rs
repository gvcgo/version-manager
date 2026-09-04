//! 文件/目录复制（对齐 Go `vmr-go/internal/utils/copy.go`）。
//!
//! - 顺序复制、无并发无进度回调。
//! - 符号链接按链接重建（不复制目标内容）。
//! - 递归复制跳过 `.Trashes` 与 `.DS_Store`（macOS 噪音）。
//! - 目录模式按源目录；文件保留源模式。

use std::fs;
use std::io;
use std::path::Path;

fn is_trash_or_ds_store(name: &str) -> bool {
    name == ".Trashes" || name == ".DS_Store"
}

/// 打开并整体复制单个文件内容（对齐 Go `CopyFile`；目标以 0o777 创建）。
pub fn copy_file(src: &Path, dst: &Path) -> io::Result<u64> {
    fs::copy(src, dst)
}

/// 复制单个文件/符号链接（对齐 Go `CopyAFile`）：
/// 普通文件 → 复制内容 + 保留模式；符号链接 → 重建链接；其它类型报错。
pub fn copy_a_file(source: &Path, destination: &Path) -> io::Result<()> {
    let meta = fs::symlink_metadata(source)?;
    if meta.file_type().is_symlink() {
        let target = fs::read_link(source)?;
        return symlink_rebuild(&target, destination);
    }
    if meta.is_file() {
        fs::copy(source, destination)?;
        fs::set_permissions(destination, meta.permissions())?;
        return Ok(());
    }
    Err(io::Error::new(
        io::ErrorKind::InvalidInput,
        format!("unsupported file kind for copy: {}", source.display()),
    ))
}

#[cfg(unix)]
fn symlink_rebuild(target: &Path, destination: &Path) -> io::Result<()> {
    std::os::unix::fs::symlink(target, destination)
}

#[cfg(windows)]
fn symlink_rebuild(target: &Path, destination: &Path) -> io::Result<()> {
    // 复刻 Go：Windows 下重建符号链接（需权限）；失败透传。
    std::os::windows::fs::symlink_dir(target, destination)
        .or_else(|_| std::os::windows::fs::symlink_file(target, destination))
}

/// 递归复制目录（对齐 Go `CopyDirectory`）。
pub fn copy_directory(source: &Path, destination: &Path) -> io::Result<()> {
    if source.as_os_str().is_empty() || destination.as_os_str().is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "paths must not be empty",
        ));
    }
    let src_meta = fs::metadata(source)?;
    fs::create_dir_all(destination)?;
    let _ = fs::set_permissions(destination, src_meta.permissions());

    let mut entries: Vec<_> = fs::read_dir(source)?
        .filter_map(|e| e.ok())
        .map(|e| {
            // Go `DirEntry.IsDir`：不跟随符号链接（链接目录按文件处理 → 走链接重建）。
            let is_dir = e.file_type().map(|t| t.is_dir()).unwrap_or(false);
            (e.file_name(), is_dir)
        })
        .collect();
    // Go os.File.Readdir 不保证排序；这里排序保证确定性。
    entries.sort_by(|a, b| a.0.cmp(&b.0));

    for (name, is_dir) in entries {
        let name = name.to_string_lossy();
        if is_trash_or_ds_store(&name) {
            continue;
        }
        let s = source.join(name.as_ref());
        let d = destination.join(name.as_ref());
        if is_dir {
            copy_directory(&s, &d)?;
        } else {
            copy_a_file(&s, &d)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    struct TempDir(PathBuf);
    impl TempDir {
        fn new(name: &str) -> Self {
            let p =
                std::env::temp_dir().join(format!("vmr-utils-copy-{}-{name}", std::process::id()));
            let _ = fs::remove_dir_all(&p);
            fs::create_dir_all(&p).unwrap();
            TempDir(p)
        }
        fn path(&self) -> &Path {
            &self.0
        }
    }
    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn touch(p: &Path, content: &str) {
        fs::create_dir_all(p.parent().unwrap()).unwrap();
        fs::write(p, content).unwrap();
    }

    #[test]
    fn copies_tree_with_nested_dirs_and_modes() {
        let t = TempDir::new("tree");
        let src = t.path().join("src");
        touch(&src.join("a/b.txt"), "b");
        touch(&src.join("run.sh"), "#!/bin/sh");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(src.join("run.sh"), fs::Permissions::from_mode(0o755)).unwrap();
        }
        // 噪音文件必须被跳过。
        touch(&src.join(".DS_Store"), "junk");
        touch(&src.join(".Trashes/x"), "junk");
        touch(&src.join("a/.DS_Store"), "junk");

        let dst = t.path().join("dst");
        copy_directory(&src, &dst).unwrap();

        assert_eq!(fs::read_to_string(dst.join("a/b.txt")).unwrap(), "b");
        assert!(dst.join("run.sh").is_file());
        assert!(!dst.join(".DS_Store").exists());
        assert!(!dst.join(".Trashes").exists());
        assert!(!dst.join("a/.DS_Store").exists());
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = fs::metadata(dst.join("run.sh"))
                .unwrap()
                .permissions()
                .mode();
            assert_eq!(mode & 0o777, 0o755, "mode preserved");
        }
    }

    #[test]
    #[cfg(unix)]
    fn symlink_is_recreated_not_followed() {
        let t = TempDir::new("symlink-copy");
        let src = t.path().join("src");
        touch(&src.join("target/file.txt"), "data");
        std::os::unix::fs::symlink("target", src.join("ln")).unwrap();

        let dst = t.path().join("dst");
        copy_directory(&src, &dst).unwrap();

        // 链接存在且指向同名目标（relink 语义），不是复制的内容。
        let target = fs::read_link(dst.join("ln")).unwrap();
        assert_eq!(target, PathBuf::from("target"));
        assert!(dst.join("target/file.txt").is_file());
    }

    #[test]
    fn copy_a_file_preserves_content() {
        let t = TempDir::new("file");
        let src = t.path().join("src.bin");
        fs::write(&src, b"payload").unwrap();
        let dst = t.path().join("dst.bin");
        copy_a_file(&src, &dst).unwrap();
        assert_eq!(fs::read(&dst).unwrap(), b"payload");
    }
}
