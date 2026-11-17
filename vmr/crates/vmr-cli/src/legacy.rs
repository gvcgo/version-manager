//! Startup cleanup for an installation created by the Go implementation.
//!
//! The Go version copied itself to `~/.vmr/vmr` (`vmr.exe` on Windows) and wrote
//! `~/.vmr/vmr.sh|vmr.fish` in a pre-`$HOME` form. A binary left there can shadow the Rust
//! one on `PATH`, so it is reported (and deleted after confirmation when stdin is a terminal),
//! and the env files are rewritten to the current form. Commands whose contract is read-only
//! or stream-based skip this (see `may_prompt` in `main.rs`).

use std::io::{IsTerminal, Write};
use std::path::{Path, PathBuf};

use vmr_core::paths;

/// Binary names the Go implementation installed under `~/.vmr`.
const LEGACY_BINARIES: [&str; 2] = ["vmr", "vmr.exe"];

/// Checks `~/.vmr` for a binary and env file left by the Go implementation.
pub fn check() {
    for path in legacy_binaries(&paths::work_dir_path()) {
        offer_removal(&path);
    }
    for path in vmr_shell::migrate_legacy_env_files() {
        eprintln!("Updated legacy vmr lines in {}.", path.display());
    }
}

/// Existing legacy binary files, excluding the executable currently running.
fn legacy_binaries(dir: &Path) -> Vec<PathBuf> {
    LEGACY_BINARIES
        .iter()
        .map(|name| dir.join(name))
        .filter(|path| path.is_file() && !is_current_exe(path))
        .collect()
}

/// Whether `path` and the running executable resolve to the same file, so a vmr installed in
/// `~/.vmr` is not reported as old.
fn is_current_exe(path: &Path) -> bool {
    let Ok(current) = std::env::current_exe() else {
        return false;
    };
    let same_path = match (std::fs::canonicalize(path), std::fs::canonicalize(&current)) {
        (Ok(a), Ok(b)) => a == b,
        _ => false,
    };
    same_path || same_file(path, &current)
}

/// Hardlinks resolve to different canonical paths even though they share one file.
#[cfg(unix)]
fn same_file(a: &Path, b: &Path) -> bool {
    use std::os::unix::fs::MetadataExt;
    match (a.metadata(), b.metadata()) {
        (Ok(a), Ok(b)) => a.dev() == b.dev() && a.ino() == b.ino(),
        _ => false,
    }
}

#[cfg(not(unix))]
fn same_file(_: &Path, _: &Path) -> bool {
    false
}

/// Asks for confirmation on a terminal; otherwise prints the manual removal hint.
fn offer_removal(path: &Path) {
    if !std::io::stdin().is_terminal() {
        eprintln!(
            "An old vmr binary was found at {}; delete it manually to avoid running it.",
            path.display()
        );
        return;
    }
    eprintln!("An old version of vmr was found: {}", path.display());
    eprint!("Delete it now? [y/N] ");
    let _ = std::io::stderr().flush();
    let mut input = String::new();
    if std::io::stdin().read_line(&mut input).unwrap_or(0) == 0 || !crate::is_yes(&input) {
        eprintln!(
            "Kept {}; delete it manually if it is no longer needed.",
            path.display()
        );
        return;
    }
    match std::fs::remove_file(path) {
        Ok(()) => eprintln!("Removed {}.", path.display()),
        Err(error) => eprintln!("failed to remove {}: {error}", path.display()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_files_and_ignores_directories() {
        let dir = std::env::temp_dir().join(format!("vmr-legacy-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("vmr"), b"old").unwrap();
        std::fs::create_dir(dir.join("vmr.exe")).unwrap();

        assert_eq!(legacy_binaries(&dir), vec![dir.join("vmr")]);

        let _ = std::fs::remove_dir_all(&dir);
    }
}
