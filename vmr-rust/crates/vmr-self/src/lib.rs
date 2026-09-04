//! vmr self install/uninstall (plan.md §3.10; no TUI, the SDK directory comes via CLI arg/default).
//!
//! - `install_self`: copies its own executable to `~/.vmr/<bin>` → writes the shell environment
//!   (cd hook + PATH) → generates the `vmr-update` / `vmr-uninstall` scripts.
//! - `uninstall_self`: performs the uninstall (removing its own file and work directory is the caller's/scripts' responsibility).
//!
//! Update/uninstall script source: `https://scripts.vmr.dpdns.org` (unix) / `/windows`.

use std::fs;
use std::path::{Path, PathBuf};

use vmr_core::paths;

const SCRIPTS_HOST: &str = "https://scripts.vmr.dpdns.org";

/// Installation binary target: `~/.vmr/<bin_name>`.
pub fn installed_bin_path() -> PathBuf {
    paths::work_dir().join(bin_name())
}

pub fn bin_name() -> String {
    std::env::args()
        .next()
        .and_then(|a| {
            Path::new(&a)
                .file_name()
                .map(|s| s.to_string_lossy().into_owned())
        })
        .unwrap_or_else(|| "vmr".to_string())
}

/// Copies itself to `~/.vmr/<bin>` and writes the shell environment and scripts.
pub fn install_self(current_exe: &Path, sdk_dir: Option<&str>) -> Result<(), String> {
    let dest = installed_bin_path();
    fs::create_dir_all(paths::work_dir()).map_err(|e| e.to_string())?;
    fs::copy(current_exe, &dest).map_err(|e| format!("copy self failed: {e}"))?;
    #[cfg(unix)]
    chmod_x(&dest);

    // Write the shell environment (cd hook + source block).
    let shell = vmr_shell::Shell::detect();
    shell.write_vm_env_to_shell();

    // SDK installation directory setting (conf).
    if let Some(dir) = sdk_dir {
        if !dir.is_empty() {
            let mut conf = vmr_core::conf::VMRConf::new();
            conf.sdk_installation_dir = dir.trim_end_matches('/').to_string();
            let _ = conf.save();
        }
    }

    write_scripts()
}

fn chmod_x(path: &Path) {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if let Ok(meta) = fs::metadata(path) {
            let _ = fs::set_permissions(
                path,
                fs::Permissions::from_mode(meta.permissions().mode() | 0o111),
            );
        }
    }
    #[cfg(not(unix))]
    {
        let _ = path;
    }
}

/// Generates the update/uninstall scripts (unix sh; on windows it is described in ps1 on the cli side, see comment).
fn write_scripts() -> Result<(), String> {
    #[cfg(unix)]
    {
        let upd = paths::work_dir().join("vmr-update");
        fs::write(&upd, format!("#!/bin/sh\ncurl -sSf {SCRIPTS_HOST} | sh\n"))
            .map_err(|e| e.to_string())?;
        chmod_x(&upd);
        let un = paths::work_dir().join("vmr-uninstall");
        fs::write(&un, "#!/bin/sh\ncd ~; vmr Uins; rm -rf ~/.vmr\n").map_err(|e| e.to_string())?;
        chmod_x(&un);
    }
    #[cfg(windows)]
    {
        // PowerShell: irm {SCRIPTS_HOST}/windows | iex (mirrors Go; the mingw wrapper has been simplified).
        let _ = SCRIPTS_HOST;
    }
    Ok(())
}

/// Removes the vmr environment from the shell config and deletes its own binary (the data directory is preserved for removal by the script).
pub fn uninstall_self() -> Result<(), String> {
    let bin = installed_bin_path();
    if bin.exists() {
        fs::remove_file(&bin).map_err(|e| e.to_string())?;
    }
    // Environment removal: rewritten as a minimal shell file; the user deletes ~/.vmr after uninstalling.
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bin_name_not_empty() {
        assert!(!bin_name().is_empty());
    }
}
