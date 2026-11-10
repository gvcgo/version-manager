//! Conda/pixi prefix environment conventions.
//!
//! A pixi/conda-forge package is installed into a conda *prefix* (the version directory).
//! Activation never scans the prefix: it adds a fixed set of prefix-relative directories to
//! `PATH`. The list below mirrors, in the same order,
//!
//! - `rattler_shell::activation::prefix_path_entries` (rattler_shell 0.29.6), which is what
//!   `rattler_shell::activation::Activator` uses for activation scripts, and
//! - pixi's `pixi_utils::executable_utils::is_binary_folder`.
//!
//! Using the same list means a package behaves the same whether it is activated by
//! conda/pixi or exposed by vmr: binaries that conda would put on `PATH` are found, and no
//! prefix-wide scan (which would also pick up `lib/`, `include/`, etc.) happens.
//!
//! `vmr_env::collect` keeps only directories that exist, so the Windows list can safely
//! contain entries a particular package does not create (e.g. `Library/bin`).
//!
//! Out of scope here: variables set by `etc/conda/activate.d/*` or
//! `etc/conda/env_vars.d/*.json`. Those are activation-time metadata, not the prefix
//! layout, and are not written to the global shell environment.

use std::path::Path;

use vmr_lua::types::{DirItems, InstallerConfig};

/// Marker file written into every pixi/conda prefix created by vmr.
///
/// A bare `conda-meta/` directory cannot identify the backend: a plugin-installed Miniconda
/// prefix also contains one, and it has its own `InstallerConfig`. The marker makes the
/// distinction explicit, so `vmr use`, `vmr uninstall` and the `cd` hook all resolve the
/// environment with the same rules.
pub const PIXI_MARKER: &str = ".vmr-pixi";

/// `InstallerConfig` describing a conda prefix's executables.
///
/// Only `binary_dirs` is filled: pixi/conda activation contributes `PATH` entries, while
/// extra variables (`CONDA_PREFIX`, ...) are intentionally not written to vmr's global shell
/// file. They would overwrite the values of a user-managed conda installation.
pub fn installer_config() -> InstallerConfig {
    InstallerConfig {
        binary_dirs: Some(path_entries()),
        ..Default::default()
    }
}

/// Prefix-relative `PATH` directories, exactly as conda/pixi define them.
pub fn path_entries() -> DirItems {
    // The empty Windows entry is the prefix root: DLLs and a few launchers live directly
    // there. `vmr_env` treats an empty `DirPath` as the base directory itself.
    DirItems {
        linux: vec![dir(&["bin"])],
        darwin: vec![dir(&["bin"])],
        windows: vec![
            Vec::new(),
            dir(&["Library", "mingw-w64", "bin"]),
            dir(&["Library", "usr", "bin"]),
            dir(&["Library", "bin"]),
            dir(&["Scripts"]),
            dir(&["bin"]),
        ],
    }
}

fn dir(parts: &[&str]) -> Vec<String> {
    parts.iter().map(|part| part.to_string()).collect()
}

/// Records that `prefix` was installed through the pixi/rattler backend.
///
/// Called for conda installs whether the directory was just extracted or already existed,
/// so prefixes installed before the marker was introduced are upgraded on the next
/// `vmr use` instead of requiring a reinstall.
pub fn mark_prefix(prefix: &Path) -> Result<(), String> {
    std::fs::write(prefix.join(PIXI_MARKER), b"pixi\n")
        .map_err(|e| format!("write {} failed: {e}", prefix.join(PIXI_MARKER).display()))
}

/// Whether `prefix` was installed through the pixi/rattler backend.
pub fn is_pixi_prefix(prefix: &Path) -> bool {
    prefix.join(PIXI_MARKER).is_file()
}

/// Resolves the environment bundle for an installed version.
///
/// Order matters:
/// 1. a pixi marker always means the conda layout (even when a plugin with the same SDK
///    name exists, e.g. `vmr use -p go@...`);
/// 2. otherwise the plugin's `InstallerConfig` describes the archive layout;
/// 3. with neither, there is nothing to add to `PATH`.
pub fn collect_bundle(prefix: &Path, plugin_ic: Option<&InstallerConfig>) -> vmr_env::EnvBundle {
    let base = prefix.to_string_lossy();
    if is_pixi_prefix(prefix) {
        vmr_env::collect(&installer_config(), base.as_ref(), os_name())
    } else if let Some(ic) = plugin_ic {
        vmr_env::collect(ic, base.as_ref(), os_name())
    } else {
        vmr_env::EnvBundle::default()
    }
}

/// Current platform name as used by `vmr_env`/`InstallerConfig`.
fn os_name() -> &'static str {
    if cfg!(windows) {
        "windows"
    } else if cfg!(target_os = "macos") {
        "darwin"
    } else {
        "linux"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use vmr_lua::types::dir_items_for;

    /// Compile-time base: keeps filesystem calls out of runtime temp paths.
    fn test_dir(name: &str) -> PathBuf {
        let p = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("target")
            .join("pixi-env-tests")
            .join(format!("{}-{name}", std::process::id()));
        let _ = std::fs::remove_dir_all(&p);
        std::fs::create_dir_all(&p).unwrap();
        p
    }

    #[test]
    fn path_entries_match_conda_activation() {
        let ic = installer_config();
        let dirs = ic.binary_dirs.as_ref().unwrap();
        assert_eq!(dir_items_for(dirs, "linux"), &vec![vec!["bin".to_string()]]);
        assert_eq!(
            dir_items_for(dirs, "darwin"),
            &vec![vec!["bin".to_string()]]
        );
        assert_eq!(
            dir_items_for(dirs, "windows"),
            &vec![
                Vec::new(),
                vec![
                    "Library".to_string(),
                    "mingw-w64".to_string(),
                    "bin".to_string()
                ],
                vec!["Library".to_string(), "usr".to_string(), "bin".to_string()],
                vec!["Library".to_string(), "bin".to_string()],
                vec!["Scripts".to_string()],
                vec!["bin".to_string()],
            ]
        );
        assert!(ic.additional_envs.is_empty());
    }

    #[test]
    fn marker_identifies_pixi_prefix() {
        let dir = test_dir("marker");
        assert!(!is_pixi_prefix(&dir));
        mark_prefix(&dir).unwrap();
        assert!(is_pixi_prefix(&dir));
        let _ = std::fs::remove_dir_all(&dir);
    }
}
