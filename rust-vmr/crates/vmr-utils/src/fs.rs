use std::fs;
use std::io;
use std::path::Path;

/// Standard file copy using std::io::copy.
pub fn copy_file(src: &Path, dst: &Path) -> io::Result<u64> {
    let mut src_file = fs::File::open(src)?;
    let mut dst_file = fs::File::create(dst)?;
    io::copy(&mut src_file, &mut dst_file)
}

/// Handles regular files AND symlinks.
/// - Regular file: copy contents + replicate file mode using set_permissions.
/// - Symlink: read link target and create new symlink.
/// - Otherwise return an error.
pub fn copy_a_file(source: &Path, destination: &Path) -> io::Result<()> {
    if source.as_os_str().is_empty() {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "no source file path provided"));
    }
    if destination.as_os_str().is_empty() {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "no destination file path provided"));
    }

    let metadata = fs::symlink_metadata(source)?;
    let file_type = metadata.file_type();

    if file_type.is_symlink() {
        let link_target = fs::read_link(source)?;
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(&link_target, destination)?;
        }
        #[cfg(windows)]
        {
            if link_target.is_dir() {
                std::os::windows::fs::symlink_dir(&link_target, destination)?;
            } else {
                std::os::windows::fs::symlink_file(&link_target, destination)?;
            }
        }
    } else if file_type.is_file() {
        let mut src_file = fs::File::open(source)?;
        let mut dst_file = fs::File::create(destination)?;
        io::copy(&mut src_file, &mut dst_file)?;

        // Replicate the source file mode for the destination file
        let permissions = metadata.permissions();
        fs::set_permissions(destination, permissions)?;
    } else {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!("unable to copy file with mode: {:?}", file_type),
        ));
    }

    Ok(())
}

/// Recursively copies a directory. Skips `.Trashes` and `.DS_Store` entries.
pub fn copy_directory(source: &Path, destination: &Path) -> io::Result<()> {
    if source.as_os_str().is_empty() || destination.as_os_str().is_empty() {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "file paths must not be empty"));
    }

    let source_info = fs::metadata(source)?;
    fs::create_dir_all(destination)?;
    fs::set_permissions(destination, source_info.permissions())?;

    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let file_name = entry.file_name();
        let name = file_name.to_string_lossy();

        if name == ".Trashes" || name == ".DS_Store" {
            continue;
        }

        let src_path = source.join(&file_name);
        let dst_path = destination.join(&file_name);

        if entry.file_type()?.is_dir() {
            copy_directory(&src_path, &dst_path)?;
        } else {
            copy_a_file(&src_path, &dst_path)?;
        }
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// HomeDirFinder — finds the "home" directory of an extracted SDK
// ---------------------------------------------------------------------------

/// Finds the "home" directory of an extracted SDK by looking for flag files.
pub struct HomeDirFinder {
    home: Option<std::path::PathBuf>,
    flag_files: Vec<String>,
    except_dir: bool,
}

impl HomeDirFinder {
    pub fn new(flag_files: Vec<String>) -> Self {
        HomeDirFinder {
            home: None,
            flag_files,
            except_dir: false,
        }
    }

    /// Depth-first search: check if all flag_files exist in current dir
    /// (by name match), if not, recurse into subdirs (skip `__MACOSX`).
    pub fn find(&mut self, start_dir: &Path) {
        if self.home.is_some() {
            return;
        }

        let entries = match fs::read_dir(start_dir) {
            Ok(d) => d,
            Err(_) => return,
        };

        // Collect all entry names in current dir
        let mut file_names = String::new();
        for entry in entries.flatten() {
            let ft = match entry.file_type() {
                Ok(t) => t,
                Err(_) => continue,
            };

            if self.except_dir && ft.is_dir() {
                continue;
            }

            file_names.push_str(&entry.file_name().to_string_lossy());
        }

        // Check if all flag files exist in current dir
        let ok = self.flag_files.iter().all(|ff| file_names.contains(ff.as_str()));

        if ok {
            self.home = Some(start_dir.to_path_buf());
        } else {
            // Recurse into subdirs
            let entries: Vec<_> = match fs::read_dir(start_dir) {
                Ok(d) => d.flatten().collect(),
                Err(_) => return,
            };

            for entry in entries {
                let ft = match entry.file_type() {
                    Ok(t) => t,
                    Err(_) => continue,
                };
                if ft.is_dir() && entry.file_name() != "__MACOSX" {
                    self.find(&start_dir.join(entry.file_name()));
                }
            }
        }
    }

    pub fn set_flags(&mut self, flag_files: Vec<String>) {
        self.flag_files = flag_files;
    }

    pub fn set_flag_dir_excepted(&mut self, ok: bool) {
        self.except_dir = ok;
    }

    pub fn clear(&mut self) {
        self.flag_files.clear();
        self.home = None;
        self.except_dir = false;
    }

    pub fn get_dir_name(&self) -> Option<&std::path::PathBuf> {
        self.home.as_ref()
    }
}

// ---------------------------------------------------------------------------
// Platform-specific file modified time
// ---------------------------------------------------------------------------

/// Returns the file's last modified time as unix seconds.
/// Uses `std::fs::metadata` and `modified()` which works cross-platform.
#[cfg(any(target_os = "linux", target_os = "macos", target_os = "freebsd", target_os = "dragonfly"))]
pub fn get_file_last_modified_time(path: &Path) -> Option<i64> {
    let metadata = fs::metadata(path).ok()?;
    let modified = metadata.modified().ok()?;
    let duration = modified.duration_since(std::time::UNIX_EPOCH).ok()?;
    Some(duration.as_secs() as i64)
}

#[cfg(windows)]
pub fn get_file_last_modified_time(path: &Path) -> Option<i64> {
    let metadata = fs::metadata(path).ok()?;
    let modified = metadata.modified().ok()?;
    let duration = modified.duration_since(std::time::UNIX_EPOCH).ok()?;
    Some(duration.as_secs() as i64)
}
