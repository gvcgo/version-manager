//! Installer main dispatch (mirrors Go `installer.go` +
//! `install/{unarchiver,executable,conda,coursier}.go`).
//!
//! Disk contract: `<versions>/<sdk>_versions/<plugin>-<version>` + `<sdk>` symlink.
//! Modes: Globally (symlink + global env) / Sessionly / ToLock (writes .vmr.lock);
//! the latter two return `Action::RunSession`, letting the caller spawn a child shell
//! (PTY belongs to vmr-pty).

use std::path::{Path, PathBuf};

use vmr_core::paths;
use vmr_lua::types::{InstallerConfig, Item, installer_kind};
use vmr_utils::copy::copy_directory;
use vmr_utils::extract::extract;
use vmr_utils::find_dir::HomeDirFinder;
use vmr_utils::symlink::create_sym_link;

use crate::common::{install_dir, sdk_version_dir, symbol_link_path};
use crate::download::download_to_cache;
use crate::locker::VersionLocker;
use crate::post::run_post_install;

pub const ADD_TO_PATH_TEMPORARILY_ENV: &str = "VMR_ADD_TO_PATH_TEMPORARILY";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InvokeMode {
    Globally,
    Sessionly,
    ToLock,
}

/// Result of an install action.
#[derive(Debug)]
pub enum Action {
    /// Done (global mode).
    Done,
    /// Requires a session child shell (session/lock modes; lock written as needed).
    RunSession,
}

/// A single install request.
pub struct InstallRequest {
    pub sdk_name: String,
    pub plugin_name: String,
    pub version_name: String,
    pub version: Item,
    pub ic: InstallerConfig,
    pub mode: InvokeMode,
    pub no_envs: bool,
}

impl InstallRequest {
    pub fn install_dir(&self) -> PathBuf {
        install_dir(&self.sdk_name, &self.plugin_name, &self.version_name)
    }

    pub fn symbol_path(&self) -> PathBuf {
        symbol_link_path(&self.sdk_name)
    }

    fn is_installed(&self) -> bool {
        std::fs::read_dir(self.install_dir())
            .map(|mut it| it.next().is_some())
            .unwrap_or(false)
    }

    /// Installs when not yet installed; returns whether it was newly installed.
    fn ensure_installed(&self) -> Result<bool, String> {
        if self.is_installed() {
            return Ok(false);
        }
        match self.version.installer.as_str() {
            installer_kind::CONDA | installer_kind::CONDA_FORGE => {
                self.install_via_conda()?;
            }
            installer_kind::COURSIER => {
                self.install_via_coursier()?;
            }
            installer_kind::EXECUTABLE | installer_kind::DPKG | installer_kind::RPM => {
                self.install_via_executable()?;
            }
            _ => self.install_via_unarchiver()?,
        }
        run_post_install(&self.plugin_name, &self.install_dir(), &self.version)?;
        Ok(true)
    }

    fn install_via_unarchiver(&self) -> Result<(), String> {
        let cached = download_to_cache(&self.plugin_name, &self.version_name, &self.version)
            .ok_or("download failed")?;
        let temp = paths::temp_dir();
        // Clean up leftovers from the last run (only vmr-rust-owned subdirectories).
        let work = temp.join(format!("extract-{}", self.plugin_name.replace('.', "_")));
        let _ = std::fs::remove_dir_all(&work);
        std::fs::create_dir_all(&work).map_err(|e| e.to_string())?;

        let result = (|| -> Result<(), String> {
            extract(&cached, &work).map_err(|e| format!("extract failed: {e}"))?;

            let mut finder = HomeDirFinder::new(flag_files_for(&self.ic, current_os()));
            finder.set_flag_dir_excepted(self.ic.flag_dir_excepted);
            finder.find(&work);
            let home = finder.get_dir_name().ok_or("can't find dir to copy")?;

            copy_directory(&home, &self.install_dir()).map_err(|e| format!("copy failed: {e}"))?;
            Ok(())
        })();
        let _ = std::fs::remove_dir_all(&work);
        result
    }

    fn install_via_executable(&self) -> Result<(), String> {
        let cached = download_to_cache(&self.plugin_name, &self.version_name, &self.version)
            .ok_or("download failed")?;
        let dir = self.install_dir();
        std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
        let name = cached.file_name().ok_or("bad file")?;
        let dest = dir.join(name);
        vmr_utils::copy::copy_a_file(&cached, &dest).map_err(|e| e.to_string())?;
        if !cfg!(windows) {
            chmod_x(&dest);
        }
        // BinaryRename: rename the executable file.
        if let Some(br) = &self.ic.binary_rename {
            if !br.name_flag.is_empty() {
                if let Ok(entries) = std::fs::read_dir(&dir) {
                    for e in entries.flatten() {
                        let p = e.path();
                        if e.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                            continue;
                        }
                        let fname = e.file_name().to_string_lossy().into_owned();
                        if fname.contains(&br.name_flag) {
                            let mut new_name = br.rename_to.clone();
                            if cfg!(windows) {
                                new_name.push_str(".exe");
                            }
                            let _ = std::fs::rename(p, dir.join(new_name));
                        }
                    }
                }
            }
        }
        Ok(())
    }

    fn install_via_coursier(&self) -> Result<(), String> {
        let cs = std::env::var("VMR_COURSIER_PATH").unwrap_or_else(|_| "cs".to_string());
        let version = self
            .version_name
            .trim_end_matches("-LTS")
            .trim_end_matches("-lts");
        let args: Vec<String> = vec![
            cs,
            "install".to_string(),
            "-q".to_string(),
            format!("--install-dir={}", self.install_dir().display()),
            format!("{}:{version}", self.plugin_name),
        ];
        run_cmd(&args, &paths::work_dir()).map_err(|e| format!("coursier failed: {e}"))
    }

    fn install_via_conda(&self) -> Result<(), String> {
        // Requirement 2/4: does not depend on a local conda; install straight from the
        // source into the version directory via vmr-conda.
        let pkg = vmr_conda::select_package(&self.plugin_name, &self.version_name)?
            .ok_or("package not found in conda source")?;
        vmr_conda::install_package(&pkg, &self.install_dir())
    }

    /// OS name of the current platform.
    fn collect_bundle(&self, base: &Path) -> vmr_env::EnvBundle {
        if self.no_envs {
            return vmr_env::EnvBundle::default();
        }
        vmr_env::collect(&self.ic, base.to_str().unwrap_or(""), &current_os())
    }

    fn set_env_globally(&self) {
        if self.no_envs {
            return;
        }
        let shell = vmr_shell::Shell::detect();
        let bundle = self.collect_bundle(&self.symbol_path());
        vmr_env::set_globally(&shell, &bundle);
    }

    fn unset_env_globally(&self) {
        if self.no_envs {
            return;
        }
        let shell = vmr_shell::Shell::detect();
        let bundle = self.collect_bundle(&self.symbol_path());
        vmr_env::unset_globally(&shell, &bundle);
    }

    fn add_envs_temporarily(&self) {
        if self.no_envs {
            return;
        }
        let install = self.install_dir();
        let bundle = self.collect_bundle(&install);
        vmr_env::add_temporarily(&bundle);
    }
}

/// Session/lock modes: remove the SDK symlink prefix from the process PATH and prepare
/// temporary env (mirrors the non-global branch of Go `Install()`).
fn prepare_session_envs(req: &InstallRequest, to_lock: bool) {
    if to_lock {
        let mut locker = VersionLocker::default();
        locker.save(None, &req.plugin_name, &req.version_name);
    }
    remove_sdk_path_from_process(&req.sdk_name);
    unsafe { std::env::set_var(ADD_TO_PATH_TEMPORARILY_ENV, "1") };
    req.add_envs_temporarily();
}

/// Removes the SDK symlink path prefix from the process PATH
/// (mirrors Go `RemoveGlobalSDKPathTemporarily`).
pub fn remove_sdk_path_from_process(sdk_name: &str) {
    let sep = if cfg!(windows) { ';' } else { ':' };
    let symbolic = sdk_version_dir(sdk_name).join(sdk_name);
    let prefix = symbolic.to_string_lossy().into_owned();
    let cur = std::env::var("PATH").unwrap_or_default();
    let parts: Vec<&str> = cur.split(sep).filter(|p| !p.starts_with(&prefix)).collect();
    unsafe { std::env::set_var("PATH", parts.join(&sep.to_string())) };
}

/// Main entry: install + mode handling.
pub fn install(req: &InstallRequest) -> Result<Action, String> {
    if req.version.installer == installer_kind::COURSIER {
        // Prerequisite: the coursier binary must exist (Go behavior intentionally kept).
        ensure_command("cs").map_err(|_| "coursier is not installed".to_string())?;
    }
    let newly = req.ensure_installed()?;
    match req.mode {
        InvokeMode::Globally => {
            if !newly {
                println!(
                    "{} {} is already installed.",
                    req.plugin_name, req.version_name
                );
            }
            // Rebuild the symlink + global env.
            let sym = req.symbol_path();
            if sym.exists() {
                let _ = std::fs::remove_dir_all(&sym);
            }
            if req.install_dir().exists() {
                create_sym_link(req.install_dir().to_str().unwrap(), sym.to_str().unwrap())
                    .map_err(|e| format!("create symlink failed: {e}"))?;
            }
            req.set_env_globally();
            req.add_envs_temporarily();
            Ok(Action::Done)
        }
        InvokeMode::Sessionly => {
            prepare_session_envs(req, false);
            Ok(Action::RunSession)
        }
        InvokeMode::ToLock => {
            prepare_session_envs(req, true);
            Ok(Action::RunSession)
        }
    }
}

/// Uninstall: removes the version directory; for the current version also
/// removes the symlink + global env.
pub fn uninstall(req: &InstallRequest) -> Result<(), String> {
    let dir = req.install_dir();
    let _ = std::fs::remove_dir_all(&dir);
    let sym = req.symbol_path();
    if let Ok(target) = std::fs::read_link(&sym) {
        if target == dir {
            let _ = std::fs::remove_dir_all(&sym);
            req.unset_env_globally();
        }
    }
    Ok(())
}

/// `vmr use -E`: injects env per the lock and returns RunSession (mirrors Go HookForCdCommand).
pub fn hook_for_cd_command() -> Result<Action, String> {
    let locker = VersionLocker::load_from(None);
    if locker.versions.is_empty() {
        return Ok(Action::Done);
    }
    unsafe { std::env::set_var(ADD_TO_PATH_TEMPORARILY_ENV, "1") };
    for (sdk, version) in &locker.versions {
        remove_sdk_path_from_process(sdk);
        // Reuse plugin data to load ic (SDKName == plugin_name is common).
        let mut plugins = vmr_lua::Plugins::new();
        let plugin = plugins
            .get_by_sdk_name(sdk)
            .or_else(|| plugins.get_by_plugin_name(sdk))
            .ok_or_else(|| format!("no plugin for locked sdk: {sdk}"))?;
        let mut plugin = plugin;
        let item = plugin
            .get_version(version)
            .ok_or_else(|| format!("locked version not found: {sdk}@{version}"))?;
        let ic = plugin.get_installer_config()?;
        let req = InstallRequest {
            sdk_name: plugin.sdk_name.clone(),
            plugin_name: plugin.plugin_name.clone(),
            version_name: version.clone(),
            version: item,
            ic,
            mode: InvokeMode::Sessionly,
            no_envs: false,
        };
        let install = req.install_dir();
        let bundle = req.collect_bundle(&install);
        vmr_env::add_temporarily(&bundle);
    }
    Ok(Action::RunSession)
}

// ---- Internal utilities ----

fn current_os() -> String {
    if cfg!(windows) {
        "windows".to_string()
    } else if cfg!(target_os = "macos") {
        "darwin".to_string()
    } else {
        "linux".to_string()
    }
}

fn flag_files_for(ic: &InstallerConfig, os: String) -> Vec<String> {
    let Some(ff) = &ic.flag_files else {
        return Vec::new();
    };
    let v = match os.as_str() {
        "windows" => &ff.windows,
        "darwin" => &ff.darwin,
        _ => &ff.linux,
    };
    v.clone()
}

#[cfg(unix)]
fn chmod_x(path: &std::path::Path) {
    use std::os::unix::fs::PermissionsExt;
    if let Ok(meta) = std::fs::metadata(path) {
        let mode = meta.permissions().mode();
        let _ = std::fs::set_permissions(path, std::fs::Permissions::from_mode(mode | 0o111));
    }
}

#[cfg(not(unix))]
fn chmod_x(_: &std::path::Path) {}

fn run_cmd(args: &[String], cwd: &std::path::Path) -> Result<(), String> {
    let mut cmd = std::process::Command::new(&args[0]);
    cmd.args(&args[1..]);
    cmd.current_dir(cwd);
    cmd.stdin(std::process::Stdio::inherit());
    cmd.stdout(std::process::Stdio::inherit());
    cmd.stderr(std::process::Stdio::inherit());
    let status = cmd.status().map_err(|e| e.to_string())?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("command failed: {status}"))
    }
}

fn ensure_command(name: &str) -> Result<(), String> {
    let probe = std::process::Command::new(name)
        .arg("--help")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status();
    match probe {
        Ok(_) => Ok(()),
        Err(e) => Err(e.to_string()),
    }
}
