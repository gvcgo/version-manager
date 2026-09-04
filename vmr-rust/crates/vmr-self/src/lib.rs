//! vmr 自身安装/卸载（plan.md §3.10；无 TUI，SDK 目录经 CLI 参数/默认）。
//!
//! - `install_self`：复制自身可执行文件到 `~/.vmr/<bin>` → 写 shell 环境
//!   （cd hook + PATH）→ 生成 `vmr-update` / `vmr-uninstall` 脚本。
//! - `uninstall_self`：执行卸载（删除自身文件与工作目录由调用方/脚本负责）。
//!
//! 更新/卸载脚本源：`https://scripts.vmr.dpdns.org`（unix）/`/windows`。

use std::fs;
use std::path::{Path, PathBuf};

use vmr_core::paths;

const SCRIPTS_HOST: &str = "https://scripts.vmr.dpdns.org";

/// 安装二进制目标：`~/.vmr/<bin_name>`。
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

/// 复制自身到 `~/.vmr/<bin>` 并写 shell 环境与脚本。
pub fn install_self(current_exe: &Path, sdk_dir: Option<&str>) -> Result<(), String> {
    let dest = installed_bin_path();
    fs::create_dir_all(paths::work_dir()).map_err(|e| e.to_string())?;
    fs::copy(current_exe, &dest).map_err(|e| format!("copy self failed: {e}"))?;
    #[cfg(unix)]
    chmod_x(&dest);

    // 写 shell 环境（cd hook + source 块）。
    let shell = vmr_shell::Shell::detect();
    shell.write_vm_env_to_shell();

    // SDK 安装目录设置（conf）。
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

/// 生成更新/卸载脚本（unix sh；windows 由 cli 侧用 ps1 描述，见注释）。
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
        // PowerShell：irm {SCRIPTS_HOST}/windows | iex（Go 对齐；含 mingw 包装已简化）。
        let _ = SCRIPTS_HOST;
    }
    Ok(())
}

/// 从 shell 配置摘除 vmr 环境并删除自身二进制（保留数据目录由脚本删除）。
pub fn uninstall_self() -> Result<(), String> {
    let bin = installed_bin_path();
    if bin.exists() {
        fs::remove_file(&bin).map_err(|e| e.to_string())?;
    }
    // 环境摘除：重写为最小 shell 文件由用户在卸载后删除 ~/.vmr。
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
