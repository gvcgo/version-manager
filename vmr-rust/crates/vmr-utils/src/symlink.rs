//! Directory/file symlinks (mirrors `CreateSymLink` in Go `vmr-go/internal/utils/shell.go`).
//!
//! Windows uses a junction (a `mklink /j` subprocess — a directory link requiring no admin
//! privileges); other platforms use `std::os::unix::fs::symlink` (works for both dirs and files).

use std::io;

#[cfg(windows)]
use crate::exec::exec;

/// Create a directory symlink for `oldname` at `newname`.
///
/// Windows: run `cmd /c mklink /j <newname> <oldname>` with the user's home directory as cwd.
/// Other platforms: `symlink(oldname, newname)`.
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
        // A directory link can be accessed as a directory.
        assert!(base.join("link").is_dir());
        let _ = fs::remove_dir_all(&base);
    }
}
