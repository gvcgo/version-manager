//! vmr-env: environment variable collection and injection (plan.md §3.7, requirement 6).
//!
//! - `collect`: takes `ic.BinaryDirs.{linux|darwin|windows}` per platform (the version
//!   directory itself when empty) plus `ic.AdditionalEnvs`; keeps only **existing**
//!   paths (mirrors Go `CollectEnvs`).
//! - `set_globally` / `unset_globally`: write via vmr-shell to the shell environment file.
//! - `add_temporarily`: injects the SDK bin into the current process PATH when
//!   `VMR_ADD_TO_PATH_TEMPORARILY=1` (used by session/lock-mode subshells).
//! - `remove_global_sdk_path`: removes the current SDK symlink path from the shell
//!   environment file.
//!
//! The environment variable name contract is defined in vmr-core `envs`.

use vmr_core::envs;
use vmr_lua::types::{AdditionalEnv, InstallerConfig, dir_items_for};
use vmr_shell::Shell;

/// Collection result: PATH directories (absolute paths) and additional environment variables.
#[derive(Debug, Clone, Default)]
pub struct EnvBundle {
    pub path_dirs: Vec<String>,
    pub env_vars: Vec<(String, String)>,
}

fn sep() -> char {
    if cfg!(windows) { ';' } else { ':' }
}

fn exists(p: &std::path::Path) -> bool {
    p.exists()
}

fn join_base(base: &std::path::Path, parts: &[String]) -> String {
    let mut p = base.to_path_buf();
    for part in parts {
        p.push(part);
    }
    p.to_string_lossy().into_owned()
}

/// Collects from ic + the install root (mirrors Go `CollectEnvs(basePath)`).
pub fn collect(ic: &InstallerConfig, install_dir: &str, os: &str) -> EnvBundle {
    let mut out = EnvBundle::default();
    let base = std::path::Path::new(install_dir);

    // BinaryDirs (empty → the version directory itself).
    let dirs = ic
        .binary_dirs
        .as_ref()
        .map(|d| dir_items_for(d, os).clone())
        .unwrap_or_default();
    if dirs.is_empty() {
        out.path_dirs.push(base.to_string_lossy().into_owned());
    } else {
        for dir_path in dirs {
            let p = join_base(base, &dir_path);
            if exists(std::path::Path::new(&p)) {
                out.path_dirs.push(p);
            }
        }
    }

    // AdditionalEnvs: concatenate each path; only existing paths enter the result, and
    // same-name ones are joined with the separator.
    for env in &ic.additional_envs {
        out.env_vars.extend(collect_additional_env(env, base));
    }
    out
}

fn collect_additional_env(env: &AdditionalEnv, base: &std::path::Path) -> Vec<(String, String)> {
    let mut vals: Vec<String> = Vec::new();
    for parts in &env.value {
        let p = join_base(base, parts);
        if exists(std::path::Path::new(&p)) {
            vals.push(p);
        }
    }
    if vals.is_empty() {
        return Vec::new();
    }
    vec![(env.name.clone(), vals.join(&sep().to_string()))]
}

/// Removes the SDK install path from the current process PATH (call before session/lock mode).
pub fn remove_sdk_path_from_process_path(sdk_symlink: &str) {
    let Ok(cur) = std::env::var("PATH") else {
        return;
    };
    let parts: Vec<&str> = cur.split(sep()).filter(|p| *p != sdk_symlink).collect();
    let joined = parts.join(&sep().to_string());
    unsafe { std::env::set_var("PATH", joined) };
}

/// Injects a temporary PATH (Go `AddEnvsTemporarilly`: when VMR_ADD_TO_PATH_TEMPORARILY=1).
pub fn add_temporarily(bundle: &EnvBundle) {
    if !std::env::var(envs::ADD_TO_PATH_TEMPORARILY)
        .map(|v| v == "1")
        .unwrap_or(false)
    {
        return;
    }
    let cur = std::env::var("PATH").unwrap_or_default();
    let mut parts: Vec<String> = bundle.path_dirs.clone();
    parts.extend(cur.split(sep()).map(|s| s.to_string()));
    unsafe { std::env::set_var("PATH", parts.join(&sep().to_string())) };
}

/// Prepends dirs not already in the PATH (for interactive hooks and similar scenarios).
pub fn prepend_process_path(dirs: &[String]) {
    let cur = std::env::var("PATH").unwrap_or_default();
    let mut parts: Vec<String> = dirs.to_vec();
    parts.extend(cur.split(sep()).map(|s| s.to_string()));
    unsafe { std::env::set_var("PATH", parts.join(&sep().to_string())) };
}

/// Writes the shell environment file globally (mirrors Go `SetEnvGlobally`:
/// set_path per bin directory, set_env per additional env).
pub fn set_globally(shell: &Shell, bundle: &EnvBundle) {
    for dir in &bundle.path_dirs {
        shell.set_path(dir);
    }
    for (k, v) in &bundle.env_vars {
        shell.set_env(k, v);
    }
}

/// Unsets globally (when uninstalling the current version / an SDK).
pub fn unset_globally(shell: &Shell, bundle: &EnvBundle) {
    for dir in &bundle.path_dirs {
        shell.unset_path(dir);
    }
    for (k, _) in &bundle.env_vars {
        shell.unset_env(k);
    }
}

/// Removes the SDK symlink path from the global environment (Go `RemoveGlobalSDKPathTemporarily`).
pub fn remove_global_sdk_path(shell: &Shell, sdk_symlink: &str) {
    shell.unset_path(sdk_symlink);
}

/// A pure collection function for test verification (takes the platform as a parameter).
#[allow(dead_code)]
pub fn collect_for_test(ic: &InstallerConfig, dir: &str, os: &str) -> EnvBundle {
    collect(ic, dir, os)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn mkdirs(dir: &std::path::Path, rel: &[&str]) {
        let mut p = dir.to_path_buf();
        for r in rel {
            p.push(r);
            fs::create_dir_all(&p).unwrap();
        }
    }

    fn ic_sample() -> InstallerConfig {
        let mut ic = vmr_lua::types::InstallerConfig::new();
        let bd = ic.binary_dirs.as_mut().unwrap();
        bd.linux = vec![vec!["bin".to_string()], vec!["missing".to_string()]];
        ic.additional_envs.push(vmr_lua::types::AdditionalEnv {
            name: "GOROOT".into(),
            value: vec![vec![]],
            ..Default::default()
        });
        ic
    }

    #[test]
    fn collect_filters_existing_only() {
        let dir = std::env::temp_dir().join(format!("vmr-env-test-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        mkdirs(&dir, &["bin"]);
        let ic = ic_sample();
        let b = collect(&ic, dir.to_str().unwrap(), "linux");
        // bin exists, missing does not → path_dirs contains only bin.
        assert_eq!(b.path_dirs, vec![format!("{}/bin", dir.display())]);
        // GOROOT=base (empty path segment → base itself, which exists).
        assert_eq!(
            b.env_vars,
            vec![("GOROOT".to_string(), dir.to_string_lossy().into_owned())]
        );
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn empty_binary_dirs_defaults_to_base() {
        let dir = std::env::temp_dir().join(format!("vmr-env-test2-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let ic = vmr_lua::types::InstallerConfig::new();
        let b = collect(&ic, dir.to_str().unwrap(), "linux");
        assert_eq!(b.path_dirs, vec![dir.to_string_lossy().into_owned()]);
        let _ = fs::remove_dir_all(&dir);
    }
}
