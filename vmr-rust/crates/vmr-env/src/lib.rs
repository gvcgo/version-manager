//! vmr-env：环境变量收集与注入（plan.md §3.7，要求 6）。
//!
//! - `collect`：按平台取 `ic.BinaryDirs.{linux|darwin|windows}`（空则版本目录本身）
//!   + `ic.AdditionalEnvs`；只收**存在**的路径（对齐 Go `CollectEnvs`）。
//! - `set_globally` / `unset_globally`：经 vmr-shell 写入 shell 环境文件。
//! - `add_temporarily`：`VMR_ADD_TO_PATH_TEMPORARILY=1` 时把 SDK bin 注入当前
//!   进程 PATH（会话/锁模式子 shell 用）。
//! - `remove_global_sdk_path`：把当前 SDK 符号链接路径从环境文件摘除。
//!
//! 环境变量名契约见 vmr-core `envs`。

use vmr_core::envs;
use vmr_lua::types::{AdditionalEnv, InstallerConfig, dir_items_for};
use vmr_shell::Shell;

/// 收集结果：PATH 目录（绝对路径）与附加环境变量。
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

/// 从 ic + 安装根收集（对齐 Go `CollectEnvs(basePath)`）。
pub fn collect(ic: &InstallerConfig, install_dir: &str, os: &str) -> EnvBundle {
    let mut out = EnvBundle::default();
    let base = std::path::Path::new(install_dir);

    // BinaryDirs（空 → 版本目录本身）。
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

    // AdditionalEnvs：逐条拼路径，仅存在路径入结果；同 name 的用分隔符合并。
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

/// 把 SDK 安装路径从当前进程 PATH 摘除（会话/锁模式前调用）。
pub fn remove_sdk_path_from_process_path(sdk_symlink: &str) {
    let Ok(cur) = std::env::var("PATH") else {
        return;
    };
    let parts: Vec<&str> = cur.split(sep()).filter(|p| *p != sdk_symlink).collect();
    let joined = parts.join(&sep().to_string());
    unsafe { std::env::set_var("PATH", joined) };
}

/// 注入临时 PATH（Go `AddEnvsTemporarilly`：VMR_ADD_TO_PATH_TEMPORARILY=1 时）。
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

/// 把 PATH 中不存在的 dir 前置（供交互 hook 等场景）。
pub fn prepend_process_path(dirs: &[String]) {
    let cur = std::env::var("PATH").unwrap_or_default();
    let mut parts: Vec<String> = dirs.to_vec();
    parts.extend(cur.split(sep()).map(|s| s.to_string()));
    unsafe { std::env::set_var("PATH", parts.join(&sep().to_string())) };
}

/// 全局写 shell 环境文件（对齐 Go `SetEnvGlobally`：逐 bin 目录 set_path、
/// 逐附加 env set_env）。
pub fn set_globally(shell: &Shell, bundle: &EnvBundle) {
    for dir in &bundle.path_dirs {
        shell.set_path(dir);
    }
    for (k, v) in &bundle.env_vars {
        shell.set_env(k, v);
    }
}

/// 全局摘除（卸载当前版本 / 卸载 SDK 时）。
pub fn unset_globally(shell: &Shell, bundle: &EnvBundle) {
    for dir in &bundle.path_dirs {
        shell.unset_path(dir);
    }
    for (k, _) in &bundle.env_vars {
        shell.unset_env(k);
    }
}

/// 把 SDK 符号链接路径从全局环境摘除（Go `RemoveGlobalSDKPathTemporarily`）。
pub fn remove_global_sdk_path(shell: &Shell, sdk_symlink: &str) {
    shell.unset_path(sdk_symlink);
}

/// 供测试验证的纯收集函数（platform 取参）。
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
        // bin 存在、missing 不存在 → path_dirs 只有 bin。
        assert_eq!(b.path_dirs, vec![format!("{}/bin", dir.display())]);
        // GOROOT=base（空路径段→ base 本身，存在）。
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
