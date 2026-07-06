use std::io;
use std::path::Path;
use std::process::Command;

/// Join paths using `;` on Windows, `:` on Unix.
pub fn join_path(paths: &[&str]) -> String {
    #[cfg(windows)]
    let sep = ";";
    #[cfg(not(windows))]
    let sep = ":";

    paths.join(sep)
}

/// Create a symbolic link. On Unix uses `std::os::unix::fs::symlink`,
/// on Windows uses `cmd /c mklink /j`.
pub fn create_symlink(oldname: &Path, newname: &Path) -> io::Result<()> {
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(oldname, newname)
    }
    #[cfg(windows)]
    {
        let output = Command::new("cmd")
            .args(["/c", "mklink", "/j"])
            .arg(newname)
            .arg(oldname)
            .output()?;

        if output.status.success() {
            Ok(())
        } else {
            Err(io::Error::new(
                io::ErrorKind::Other,
                String::from_utf8_lossy(&output.stderr).to_string(),
            ))
        }
    }
}

/// Check if running in MinGW bash (Windows only).
#[cfg(windows)]
pub fn is_mingw_bash() -> bool {
    std::env::var("SHELL")
        .map(|s| s.contains("bash"))
        .unwrap_or(false)
}

#[cfg(not(windows))]
pub fn is_mingw_bash() -> bool {
    false
}

/// Convert a Windows path (e.g. `C:\foo\bar`) to MinGW format (e.g. `/c/foo/bar`).
pub fn convert_windows_path_to_mingw_path(original: &str) -> String {
    if original.is_empty() {
        return String::new();
    }
    let new_path = original.replace('\\', "/").replace(':', "");
    let parts: Vec<&str> = new_path.split('/').collect();
    if parts.is_empty() {
        return new_path;
    }
    let disk_name = parts[0].to_lowercase();
    let mut path_parts = vec![disk_name.as_str()];
    path_parts.extend(&parts[1..]);
    format!("/{}", path_parts.join("/"))
}

/// Open a URL in the default browser.
/// Windows: `cmd /c start`, Linux: `xdg-open`, macOS: `open`.
pub fn open_url(url: &str) -> io::Result<()> {
    #[cfg(target_os = "windows")]
    {
        Command::new("cmd").args(["/c", "start", url]).spawn()?;
    }
    #[cfg(target_os = "macos")]
    {
        Command::new("open").arg(url).spawn()?;
    }
    #[cfg(target_os = "linux")]
    {
        Command::new("xdg-open").arg(url).spawn()?;
    }
    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    {
        // Fallback: try xdg-open
        Command::new("xdg-open").arg(url).spawn()?;
    }
    Ok(())
}

/// Move a file using `sudo mv`.
pub fn move_file_on_unix_sudo(from: &Path, to: &Path) -> io::Result<()> {
    let status = Command::new("sudo")
        .arg("mv")
        .arg(from)
        .arg(to)
        .status()?;
    if status.success() {
        Ok(())
    } else {
        Err(io::Error::new(
            io::ErrorKind::Other,
            format!("sudo mv failed with exit code: {:?}", status.code()),
        ))
    }
}
