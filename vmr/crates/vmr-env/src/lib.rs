//! vmr-env: environment variable collection and injection (plan.md §3.7, requirement 6).
//!
//! - `collect`: takes `ic.BinaryDirs.{linux|darwin|windows}` per platform (the version
//!   directory itself when empty) plus `ic.AdditionalEnvs`; keeps only **existing**
//!   paths (mirrors Go `CollectEnvs`).
//! - `set_globally` / `unset_globally`: write via vmr-shell to the shell environment file;
//!   on Windows the `PATH` side is a single shim directory (`shims`), while other variables
//!   still go to `HKCU\Environment`.
//! - `add_temporarily`: injects the SDK bin into the current process PATH when
//!   `VMR_ADD_TO_PATH_TEMPORARILY=1` (used by session/lock-mode subshells).
//! - `remove_global_sdk_path`: removes the current SDK symlink path from the shell
//!   environment file.
//!
//! The environment variable name contract is defined in vmr-core `envs`.

use vmr_core::envs;
use vmr_lua::types::{AdditionalEnv, InstallerConfig, dir_items_for};
use vmr_shell::Shell;

#[cfg(windows)]
pub mod shims;

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
            // A `PATH` entry must be a directory; `exists` alone would also accept a file
            // that happens to have the same name. Existence is checked here, before any
            // shell file is written, so a missing conda directory is never recorded.
            if std::path::Path::new(&p).is_dir() {
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
    // Go appends an empty DirPath when the plugin supplied no value, making the base directory
    // itself the value (e.g. `GOROOT`).
    let empty = vec![Vec::<String>::new()];
    let paths = if env.value.is_empty() {
        &empty
    } else {
        &env.value
    };
    let mut vals: Vec<String> = Vec::new();
    for parts in paths {
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
    prepend_process_path(&bundle.path_dirs);
}

/// Prepends dirs to the current process PATH.
///
/// Windows keeps the first existing entry (usually `C:\Windows\system32`) ahead of the SDK
/// directories, matching the Go Windows fix (#204); Unix prepends as before.
pub fn prepend_process_path(dirs: &[String]) {
    if dirs.is_empty() {
        return;
    }
    let cur = std::env::var("PATH").unwrap_or_default();
    let joined = dirs.join(&sep().to_string());
    let new = if cur.is_empty() {
        joined
    } else {
        #[cfg(windows)]
        {
            match cur.split_once(';') {
                Some((first, rest)) => format!("{first};{joined};{rest}"),
                None => format!("{cur};{joined}"),
            }
        }
        #[cfg(not(windows))]
        {
            format!("{joined}{}{cur}", sep())
        }
    };
    unsafe { std::env::set_var("PATH", new) };
}

/// Writes the environment globally (mirrors Go `SetEnvGlobally`: `set_path` per bin
/// directory, `set_env` per additional env).
///
/// Windows keeps a single shim directory on `PATH` and writes one command shim per
/// executable of the SDK ([`shims::sync`]); the registry value therefore stays constant no
/// matter how many SDKs or pixi packages are installed. Other platforms prepend one
/// `export PATH=...` line per directory.
pub fn set_globally(shell: &Shell, sdk: &str, bundle: &EnvBundle) -> Result<(), String> {
    #[cfg(windows)]
    {
        shims::sync(sdk, bundle)?;
        let dir = shims::shim_dir();
        vmr_shell::use_shim_path(&dir.to_string_lossy());
    }
    #[cfg(not(windows))]
    {
        let _ = sdk;
        // Iterate in reverse so the final PATH order matches the declared order.
        for dir in bundle.path_dirs.iter().rev() {
            shell.set_path(dir);
        }
    }
    for (k, v) in &bundle.env_vars {
        shell.set_env(k, v);
    }
    Ok(())
}

/// Unsets globally (when uninstalling the current version / an SDK).
pub fn unset_globally(shell: &Shell, sdk: &str, bundle: &EnvBundle) {
    #[cfg(windows)]
    {
        let _ = bundle;
        shims::remove(sdk);
    }
    #[cfg(not(windows))]
    {
        let _ = sdk;
        for dir in &bundle.path_dirs {
            shell.unset_path(dir);
        }
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
        assert_eq!(
            b.path_dirs,
            vec![dir.join("bin").to_string_lossy().into_owned()]
        );
        // GOROOT=base (empty path segment → base itself, which exists).
        assert_eq!(
            b.env_vars,
            vec![("GOROOT".to_string(), dir.to_string_lossy().into_owned())]
        );
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn empty_additional_env_value_uses_base() {
        let dir = std::env::temp_dir().join(format!("vmr-env-test3-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let mut ic = vmr_lua::types::InstallerConfig::new();
        ic.additional_envs.push(vmr_lua::types::AdditionalEnv {
            name: "GOROOT".into(),
            value: Vec::new(),
            version: String::new(),
        });
        let b = collect(&ic, dir.to_str().unwrap(), "linux");
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

    #[test]
    #[cfg(windows)]
    fn temporary_path_keeps_first_windows_entry() {
        let old = std::env::var_os("PATH");
        unsafe { std::env::set_var("PATH", r"C:\Windows\system32;C:\Other") };
        prepend_process_path(&[r"C:\vmr\go\bin".to_string()]);
        let got = std::env::var("PATH").unwrap();
        match old {
            Some(v) => unsafe { std::env::set_var("PATH", v) },
            None => unsafe { std::env::remove_var("PATH") },
        }
        assert_eq!(got, r"C:\Windows\system32;C:\vmr\go\bin;C:\Other");
    }
}
