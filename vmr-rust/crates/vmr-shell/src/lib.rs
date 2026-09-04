//! vmr-shell: shell hook environment injection and automatic cd-hook switching (plan.md §3.8, requirement 7).
//!
//! Mirrors Go `internal/shell/`: a shared scheme for bash/zsh/fish —
//! - Shell environment file `~/.vmr/vmr.sh` (bash/zsh) or `~/.vmr/vmr.fish` (fish);
//! - `update_vmr_shell_file`: idempotent replacement between `# cd hook start … # cd hook end` markers;
//! - rc files (`.bashrc`/`.zshrc`/`config.fish`) get a source block appended (`VM_DISABLE` guard);
//! - `PackPath`/`PackEnv` line injection / prefix-based removal (Set/Unset).
//!
//! Session contract: skip injection when `VM_DISABLE=111`; the cd hook calls `vmr use -E` for automatic switching.
//! On Windows, the generated PowerShell config includes the cd hook (registry broadcast
//! is limited on that platform; the Rust side only provides profile write-back — documented).

use std::fs;
use std::path::{Path, PathBuf};

use vmr_core::paths;

pub const VM_DISABLE_ENV_NAME: &str = "VM_DISABLE";
pub const VM_CD_INIT_ENV_NAME: &str = "VMR_CD_INIT";
pub const MODE_PERM: u32 = 0o644;

/// Supported shell types.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Bash,
    Zsh,
    Fish,
    /// Windows PowerShell (not a primary unix path).
    PowerShell,
}

impl Kind {
    fn env_file_name(self) -> &'static str {
        match self {
            Kind::Fish => "vmr.fish",
            _ => "vmr.sh",
        }
    }

    /// Shared bash/zsh hook template (Go `vmEnvZsh`).
    fn hook_content(self, install_dir: &str, home_dir: &str) -> String {
        let inst = format_path_string(install_dir, home_dir);
        match self {
            Kind::Fish => format!(
                "# cd hook start\nfish_add_path --global {inst}\n\n\
function _vmr_cdhook --on-variable=\"PWD\" --description \"version manager cd hook\"\n\
\tif type -q vmr\n\
        vmr use -E\n\
\tend\n\
end\n\n\
if set -q \"$VMR_CD_INIT\"\n\
\tset VMR_CD_INIT \"vmr_cd_init\"\n\
    cd \"$(pwd)\"\n\
end\n\
# cd hook end"
            ),
            _ => format!(
                "# cd hook start\nexport PATH={inst}:\"${{PATH}}\"\n\n\
if [ -z \"$(alias|grep cdhook)\" ]; then\n\
\tcdhook() {{\n\
\t\tif [ $# -eq 0 ]; then\n\
\t\t\tcd\n\
\t\telse\n\
\t\t\tcd \"$@\" && vmr use -E\n\
\t\tfi\n\
\t}}\n\
\talias cd='cdhook'\n\
fi\n\n\
if [ -z \"${{VMR_CD_INIT}}\" ]; then\n\
        VMR_CD_INIT=\"vmr_cd_init\"\n\
        cd \"$(pwd)\"\n\
fi\n\
# cd hook end"
            ),
        }
    }

    /// Source block for the rc file (VM_DISABLE guard).
    fn source_block(self, env_file: &str, home_dir: &str) -> String {
        let f = format_path_string(env_file, home_dir);
        match self {
            Kind::Fish => format!(
                "# vm_envs start\nif not test ${VM_DISABLE_ENV_NAME} \n    . {f}\nend\n# vm_envs end"
            ),
            _ => format!(
                "# vm_envs start\nif [ -z \"${VM_DISABLE_ENV_NAME}\" ]; then\n    . {f}\nfi\n# vm_envs end"
            ),
        }
    }
}

/// Path display: `$HOME/...` → `~/...` (mirrors Go `FormatPathString`).
pub fn format_path_string(p: &str, home_dir: &str) -> String {
    if let Some(rest) = p.strip_prefix(home_dir) {
        return format!("~{rest}");
    }
    p.to_string()
}

/// Detect the current shell (SHELL env; docker without SHELL → bash).
pub fn detect_kind() -> Kind {
    let shell = std::env::var("SHELL").unwrap_or_default();
    if shell.ends_with("zsh") {
        Kind::Zsh
    } else if shell.ends_with("fish") {
        Kind::Fish
    } else {
        Kind::Bash
    }
}

/// Idempotently replace the hook section in the shell environment file (mirrors Go `UpdateVMRShellFile`).
pub fn update_vmr_shell_file(path: &Path, vmr_path_env: &str, new_hook: &str) {
    let old = fs::read_to_string(path).unwrap_or_default();
    if old.is_empty() {
        let _ = fs::write(path, new_hook);
        return;
    }
    let content = old.clone();
    let start = content.find("# cd hook start");
    let end = content.find("# cd hook end");
    let old_hook = match (start, end) {
        (Some(s), Some(e)) if e > s => Some(content[s..e + "# cd hook end".len()].to_string()),
        _ => None,
    };
    let content = if let Some(hook) = &old_hook {
        // Go semantics: the install-path line is stripped only when the old hook section
        // lacks it; the whole section is then replaced.
        let base = if hook.contains(vmr_path_env) {
            content
        } else {
            content.replace(vmr_path_env, "")
        };
        base.replace(hook, new_hook)
    } else {
        // No old hook: strip leftover install-path lines, then prepend the new hook.
        format!("{new_hook}\n{}", content.replace(vmr_path_env, ""))
    };
    let content = content.trim().to_string();
    let _ = fs::write(path, content);
}

/// Append the source block to the rc file (idempotent; mirrors the tail of Go `WriteVMEnvToShell`).
fn ensure_source_block(conf_path: &Path, block: &str) {
    let data = fs::read_to_string(conf_path).unwrap_or_default();
    let trimmed = block.trim().to_string();
    if data.contains(&trimmed) {
        return;
    }
    let new_data = if data.trim().is_empty() {
        block.to_string()
    } else {
        format!("{}\n{block}", data.trim_end())
    };
    let _ = fs::write(conf_path, new_data);
}

/// Shell environment operator (non-Windows).
#[derive(Debug, Clone)]
pub struct Shell {
    pub kind: Kind,
    home: String,
}

impl Shell {
    pub fn new(kind: Kind) -> Self {
        let home = dirs_home();
        Shell { kind, home }
    }

    pub fn detect() -> Self {
        let mut sh = Shell::new(detect_kind());
        if cfg!(windows) {
            sh.kind = Kind::PowerShell;
        }
        sh
    }

    /// Path of the rc file.
    pub fn conf_path(&self) -> PathBuf {
        match self.kind {
            Kind::Fish => PathBuf::from(&self.home).join(".config/fish/config.fish"),
            Kind::Zsh => PathBuf::from(&self.home).join(".zshrc"),
            Kind::PowerShell => powershell_profile_path(&self.home),
            _ => PathBuf::from(&self.home).join(".bashrc"),
        }
    }

    /// Path of the vmr shell environment file (`~/.vmr/vmr.sh|vmr.fish`).
    pub fn vm_env_conf_path(&self) -> PathBuf {
        paths::work_dir().join(self.kind.env_file_name())
    }

    fn env_file_text(&self) -> String {
        fs::read_to_string(self.vm_env_conf_path()).unwrap_or_default()
    }

    fn write_env_file(&self, data: &str) {
        let _ = fs::write(self.vm_env_conf_path(), data.trim_start_matches('\n'));
    }

    pub fn pack_path(&self, path: &str) -> String {
        match self.kind {
            Kind::Fish => format!("fish_add_path --global {path}"),
            _ => format!("export PATH={path}:\"${{PATH}}\""),
        }
    }

    pub fn pack_env(&self, key: &str, value: &str) -> String {
        match self.kind {
            Kind::Fish => {
                if value.is_empty() {
                    format!("set --global {key} ")
                } else {
                    format!("set --global {key} {value}")
                }
            }
            _ => {
                if value.is_empty() {
                    format!("export {key}=")
                } else {
                    format!("export {key}={value}")
                }
            }
        }
    }

    pub fn set_path(&self, path: &str) {
        let data = self.env_file_text();
        let line = self.pack_path(&format_path_string(path, &self.home));
        if !data.contains(&line) {
            let out = format!("{}\n{line}", data.trim_end());
            self.write_env_file(&out);
        }
    }

    pub fn unset_path(&self, path: &str) {
        let line = self.pack_path(&format_path_string(path, &self.home));
        let data = self.env_file_text();
        let out = data.replace(&line, "").replace("\n\n", "\n");
        self.write_env_file(&out);
    }

    pub fn set_env(&self, key: &str, value: &str) {
        let data = self.env_file_text();
        let env_line = self.pack_env(key, &format_path_string(value, &self.home));
        if !data.contains(&env_line) {
            let out = format!("{}\n{env_line}", data.trim_end());
            self.write_env_file(&out);
        }
    }

    pub fn unset_env(&self, key: &str) {
        let prefix = self.pack_env(key, "");
        let data = self.env_file_text();
        let mut out = String::new();
        for line in data.lines() {
            if line.trim_start().starts_with(prefix.trim_start()) {
                continue;
            }
            out.push_str(line);
            out.push('\n');
        }
        self.write_env_file(&out.replace("\n\n", "\n"));
    }

    /// Write the cd hook (shell environment file) and ensure the rc source block (mirrors `WriteVMEnvToShell`).
    pub fn write_vm_env_to_shell(&self) {
        let install = paths::work_dir().to_string_lossy().into_owned();
        let env_file = self.vm_env_conf_path();
        let hook = self.kind.hook_content(&install, &self.home);
        let path_line = match self.kind {
            Kind::Fish => format!(
                "fish_add_path --global {}",
                format_path_string(&install, &self.home)
            ),
            _ => format!(
                "export PATH={}:\"${{PATH}}\"",
                format_path_string(&install, &self.home)
            ),
        };
        update_vmr_shell_file(&env_file, &path_line, &hook);
        let block = self
            .kind
            .source_block(env_file.to_str().unwrap_or(""), &self.home);
        ensure_source_block(&self.conf_path(), &block);
    }
}

fn dirs_home() -> String {
    std::env::var("HOME")
        .ok()
        .or_else(dirs_sys_home)
        .unwrap_or_else(|| ".".to_string())
}

fn dirs_sys_home() -> Option<String> {
    #[cfg(windows)]
    {
        std::env::var("USERPROFILE").ok()
    }
    #[cfg(not(windows))]
    {
        None
    }
}

/// Windows PowerShell profile path (mirrors Go `win.go`).
fn powershell_profile_path(home: &str) -> PathBuf {
    PathBuf::from(home).join("Documents/WindowsPowerShell/profile.ps1")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn format_path_replaces_home() {
        assert_eq!(format_path_string("/home/u/.vmr", "/home/u"), "~/.vmr");
        assert_eq!(format_path_string("/opt/x", "/home/u"), "/opt/x");
    }

    #[test]
    fn update_shell_file_replaces_hook_idempotently() {
        let dir = std::env::temp_dir().join(format!("vmr-shell-test-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let f = dir.join("vmr.sh");
        let hook_a = "# cd hook start\nexport PATH=~/x:\"${PATH}\"\n# cd hook end";
        let hook_b = "# cd hook start\nexport PATH=~/y:\"${PATH}\"\n# cd hook end";
        update_vmr_shell_file(&f, "export PATH=~/x:\"${PATH}\"", hook_a);
        update_vmr_shell_file(&f, "export PATH=~/x:\"${PATH}\"", hook_b);
        let content = fs::read_to_string(&f).unwrap();
        assert!(content.contains("~/y"));
        assert!(!content.contains("~/x"));
        assert_eq!(content.matches("# cd hook start").count(), 1);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn pack_lines_shape() {
        let sh = Shell::new(Kind::Bash);
        assert_eq!(sh.pack_path("/a/b"), "export PATH=/a/b:\"${PATH}\"");
        assert_eq!(sh.pack_env("GOROOT", "/go"), "export GOROOT=/go");
        let fsh = Shell::new(Kind::Fish);
        assert_eq!(fsh.pack_env("X", ""), "set --global X ");
    }

    #[test]
    fn source_block_guards_vm_disable() {
        let sh = Shell::new(Kind::Bash);
        let b = sh.kind.source_block("~/.vmr/vmr.sh", &sh.home);
        assert!(b.contains("VM_DISABLE"));
        assert!(b.contains(". ~/.vmr/vmr.sh"));
    }
}
