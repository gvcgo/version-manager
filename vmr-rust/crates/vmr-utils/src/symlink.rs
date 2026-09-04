//! 目录/文件符号链接（对齐 Go `vmr-go/internal/utils/shell.go` 的 `CreateSymLink`）。
//!
//! Windows 用 junction（`mklink /j` 子进程，无需管理员权限的目录链接），
//! 其余平台用 `std::os::unix::fs::symlink`（目录/文件均可）。

use std::io;

#[cfg(windows)]
use crate::exec::exec;

/// 为 `oldname` 在 `newname` 处创建目录符号链接。
///
/// Windows：以用户主目录为 cwd 执行 `cmd /c mklink /j <newname> <oldname>`。
/// 其余平台：`symlink(oldname, newname)`。
pub fn create_sym_link(oldname: &str, newname: &str) -> io::Result<()> {
    #[cfg(windows)]
    {
        let home = std::env::var("USERPROFILE").unwrap_or_default();
        let args = [
            "cmd".to_string(),
            "/c".to_string(),
            "mklink".to_string(),
            "/j".to_string(),
            newname.to_string(),
            oldname.to_string(),
        ];
        exec(true, &home, &args)?;
        Ok(())
    }
    #[cfg(not(windows))]
    {
        std::os::unix::fs::symlink(oldname, newname)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    #[cfg(unix)]
    fn creates_symlink_to_directory() {
        let base = std::env::temp_dir().join(format!("vmr-utils-symlink-{}", std::process::id()));
        let _ = fs::remove_dir_all(&base);
        fs::create_dir_all(base.join("target")).unwrap();
        create_sym_link(
            base.join("target").to_str().unwrap(),
            base.join("link").to_str().unwrap(),
        )
        .unwrap();
        let link = fs::read_link(base.join("link")).unwrap();
        assert_eq!(link, base.join("target"));
        // 目录链接可被当作目录访问。
        assert!(base.join("link").is_dir());
        let _ = fs::remove_dir_all(&base);
    }
}
