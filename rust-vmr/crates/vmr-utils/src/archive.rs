use std::fs;
use std::io;
use std::path::Path;
use std::process::Command;

use flate2::read::GzDecoder;

/// Try system `unzip` command, fallback to zip crate.
pub fn unzip(src_path: &Path, dst_dir: &Path) -> io::Result<()> {
    // --- Try system unzip first ---
    #[cfg(windows)]
    let use_powershell = !crate::shell::is_mingw_bash();

    #[cfg(not(windows))]
    let use_powershell = false;

    if use_powershell {
        // powershell expand -r srcPath dstDir
        let path_var = std::env::var("PATH").unwrap_or_default();
        let new_path = format!(r"C:\Windows\System32;{}", path_var);
        std::env::set_var("PATH", &new_path);

        let status = Command::new("powershell")
            .args(["expand", "-r"])
            .arg(src_path)
            .arg(dst_dir)
            .status()?;
        if status.success() {
            return Ok(());
        }
    } else {
        let status = Command::new("unzip")
            .arg(src_path)
            .args(["-d"])
            .arg(dst_dir)
            .status();
        if let Ok(s) = status {
            if s.success() {
                return Ok(());
            }
        }
    }

    // --- Fallback to zip crate ---
    let file = fs::File::open(src_path)?;
    let mut archive = zip::ZipArchive::new(file)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    for i in 0..archive.len() {
        let mut entry = archive.by_index(i)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        let entry_name = entry.name().to_string();
        let out_path = dst_dir.join(&entry_name);

        if entry.is_dir() {
            fs::create_dir_all(&out_path)?;
        } else {
            if let Some(parent) = out_path.parent() {
                fs::create_dir_all(parent)?;
            }
            let mut out_file = fs::File::create(&out_path)?;
            io::copy(&mut entry, &mut out_file)?;
        }

        // Set permissions on Unix
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if let Some(mode) = entry.unix_mode() {
                fs::set_permissions(&out_path, fs::Permissions::from_mode(mode)).ok();
            }
        }
    }

    Ok(())
}

/// Try system `tar` command, fallback to tar/flate2 crate.
pub fn untar(src_path: &Path, dst_dir: &Path) -> io::Result<()> {
    // --- Try system tar first ---
    let status = Command::new("tar")
        .args(["-xf"])
        .arg(src_path)
        .args(["-C"])
        .arg(dst_dir)
        .status();
    if let Ok(s) = status {
        if s.success() {
            return Ok(());
        }
    }

    // --- Fallback to tar/flate2 crate ---
    let file = fs::File::open(src_path)?;

    let file_name = src_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .to_lowercase();

    if file_name.ends_with(".tar.gz") || file_name.ends_with(".tgz") {
        let decoder = GzDecoder::new(file);
        let mut archive = tar::Archive::new(decoder);
        archive.unpack(dst_dir)?;
    } else if file_name.ends_with(".tar") {
        let mut archive = tar::Archive::new(file);
        archive.unpack(dst_dir)?;
    } else {
        // Try gz decode anyway
        let decoder = GzDecoder::new(file);
        let mut archive = tar::Archive::new(decoder);
        archive.unpack(dst_dir)?;
    }

    Ok(())
}

/// Decompress by system command based on file extension.
fn decompress_by_system_command(src_path: &Path, dst_dir: &Path) -> io::Result<()> {
    let file_name = src_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .to_lowercase();

    if file_name.ends_with(".zip") {
        unzip(src_path, dst_dir)
    } else if file_name.ends_with(".tar") || file_name.contains(".tar.") {
        untar(src_path, dst_dir)
    } else {
        Err(io::Error::new(
            io::ErrorKind::Other,
            "unsupported by system command",
        ))
    }
}

/// Handle nested .zip files inside the destination directory.
fn handle_multi_compress(dest_dir: &Path) -> io::Result<()> {
    let entries = fs::read_dir(dest_dir)?;
    for entry in entries.flatten() {
        let path = entry.path();
        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            if !entry.file_type().map(|t| t.is_dir()).unwrap_or(true)
                && name.to_lowercase().ends_with(".zip")
            {
                let _ = unzip(&path, dest_dir);
            }
        }
    }
    Ok(())
}

/// Main entry point: extract an archive file to a destination directory.
/// Tries system commands first, then falls back to Rust library compression.
/// Also handles nested archives after extraction.
pub fn extract(src_file: &Path, dest_dir: &Path) -> io::Result<()> {
    // Create dest dir if not exists
    if !dest_dir.exists() {
        fs::create_dir_all(dest_dir)?;
    }

    // Try system commands first
    if decompress_by_system_command(src_file, dest_dir).is_ok() {
        handle_multi_compress(dest_dir)?;
        return Ok(());
    }

    // Fallback to Rust lib compression
    if extract_with_rust(src_file, dest_dir).is_ok() {
        handle_multi_compress(dest_dir)?;
        return Ok(());
    }

    Err(io::Error::new(
        io::ErrorKind::Other,
        "could not decompress file with system commands or rust libraries",
    ))
}

/// Extract using Rust library compression (fallback).
fn extract_with_rust(src_file: &Path, dest_dir: &Path) -> io::Result<()> {
    let file_name = src_file
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .to_lowercase();

    if file_name.ends_with(".zip") {
        unzip(src_file, dest_dir)
    } else if file_name.ends_with(".tar.gz")
        || file_name.ends_with(".tgz")
        || file_name.ends_with(".tar")
        || file_name.contains(".tar.")
    {
        untar(src_file, dest_dir)
    } else {
        Err(io::Error::new(
            io::ErrorKind::Other,
            "unsupported archive format",
        ))
    }
}
