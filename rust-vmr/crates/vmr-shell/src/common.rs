use std::env;
use std::fs;
use std::path::{Path, PathBuf};

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

pub const BASH: &str = "bash";
pub const ZSH: &str = "zsh";
pub const FISH: &str = "fish";
pub const MODE_PERM: u32 = 0o644;
pub const VM_DISABLE_ENV_NAME: &str = "VM_DISABLE";
pub const VM_ENV_FILE_NAME: &str = "vmr";

// ---------------------------------------------------------------------------
// Sheller trait
// ---------------------------------------------------------------------------

pub trait Sheller {
    /// Path to the shell config file (e.g. ~/.bashrc, ~/.zshrc).
    fn conf_path(&self) -> PathBuf;

    /// Path to the VM env file (~/.vmr/vmr).
    fn vm_env_conf_path(&self) -> PathBuf;

    /// Write a source line into the shell config file so that the vmr env
    /// file is loaded on every new shell session.
    fn write_vm_env_to_shell(&self);

    /// Format an export statement that adds `path` to the front of PATH.
    fn pack_path(&self, path: &str) -> String;

    /// Format an export statement for an environment variable.
    fn pack_env(&self, key: &str, value: &str) -> String;
}

// ---------------------------------------------------------------------------
// helpers
// ---------------------------------------------------------------------------

/// Replace the home-directory prefix with `~`.  On non-Unix (Windows) the
/// string is returned unchanged.
pub fn format_path_string(p: &str) -> String {
    #[cfg(unix)]
    {
        if let Ok(home) = env::var("HOME") {
            return p.replace(&home, "~");
        }
    }
    p.to_string()
}

/// Resolve `~` at the beginning of a path to the real home directory.
fn expand_tilde(p: &str) -> PathBuf {
    if p.starts_with('~') {
        if let Ok(home) = env::var("HOME") {
            let rest = p.strip_prefix('~').unwrap_or(p);
            return PathBuf::from(home).join(rest.trim_start_matches('/'));
        }
    }
    PathBuf::from(p)
}

/// Return the VM work directory (`~/.vmr`).
fn vmr_work_dir() -> PathBuf {
    expand_tilde("~/.vmr")
}

// ---------------------------------------------------------------------------
// update_vmr_shell_file
// ---------------------------------------------------------------------------

/// Read the file at `f_path`, find any content between `# cd hook start` and
/// `# cd hook end`, replace it with `new_hook_content`, and remove any
/// occurrence of `vmr_path_env`.  If no existing hook block is found the new
/// content is prepended.
///
/// This is used to maintain the VM env file (~/.vmr/vmr) which contains the
/// cd-hook snippet and an `export PATH=...` line.
pub fn update_vmr_shell_file(f_path: &Path, vmr_path_env: &str, new_hook_content: &str) {
    let old_data = fs::read_to_string(f_path).unwrap_or_default();
    let mut old_content = old_data.trim().to_string();

    if old_content.is_empty() {
        let _ = fs::write(f_path, new_hook_content);
        return;
    }

    // Remove any existing vmr PATH export line.
    if !old_content.contains(vmr_path_env) {
        old_content = old_content.replace(vmr_path_env, "");
    }

    // Find the block between markers.
    if let Some(old_hook) = extract_hook_block(&old_content) {
        old_content = old_content.replace(&old_hook, new_hook_content);
    } else {
        old_content = format!("{}\n{}", new_hook_content, old_content);
    }

    let _ = fs::write(f_path, old_content.trim());
}

/// Extract the first occurrence of text between `# cd hook start` and
/// `# cd hook end` (inclusive of both markers).
fn extract_hook_block(content: &str) -> Option<String> {
    let start_marker = "# cd hook start";
    let end_marker = "# cd hook end";
    let start = content.find(start_marker)?;
    let after_start = &content[start..];
    let end = after_start.find(end_marker)?;
    Some(after_start[..end + end_marker.len()].to_string())
}

// ---------------------------------------------------------------------------
// shell-specific constants (source-line templates)
// ---------------------------------------------------------------------------

/// Used for bash/zsh – writes a guarded `source` line into the shell config.
fn bash_zsh_source_line(vm_env_path: &str) -> String {
    format!(
        "# vm_envs start\nif [ -z \"${}\" ]; then\n    . {}\nfi\n# vm_envs end",
        VM_DISABLE_ENV_NAME,
        vm_env_path
    )
}

/// Used for fish – writes a guarded `source` line into the shell config.
fn fish_source_line(vm_env_path: &str) -> String {
    format!(
        "# vm_envs start\nif not test ${}\n    source {}\nend\n# vm_envs end",
        VM_DISABLE_ENV_NAME,
        vm_env_path
    )
}

/// Append `source_line` to `conf_path` unless it is already present.
fn append_source_line_to_config(conf_path: &Path, source_line: &str) {
    let data = fs::read_to_string(conf_path).unwrap_or_default();
    if data.contains(source_line.trim()) {
        return;
    }
    let new_data = if data.trim().is_empty() {
        source_line.to_string()
    } else {
        format!("{}\n{}", data.trim_end(), source_line)
    };
    let _ = fs::write(conf_path, new_data);
}

// ---------------------------------------------------------------------------
// BashShell
// ---------------------------------------------------------------------------

pub struct BashShell;

impl Sheller for BashShell {
    fn conf_path(&self) -> PathBuf {
        expand_tilde("~/.bashrc")
    }

    fn vm_env_conf_path(&self) -> PathBuf {
        vmr_work_dir().join(VM_ENV_FILE_NAME)
    }

    fn write_vm_env_to_shell(&self) {
        let vm_env_path = format_path_string(
            self.vm_env_conf_path().to_str().unwrap_or("~/.vmr/vmr"),
        );
        let source_line = bash_zsh_source_line(&vm_env_path);
        append_source_line_to_config(&self.conf_path(), &source_line);
    }

    fn pack_path(&self, path: &str) -> String {
        format!("export PATH=\"{}:$PATH\"", path)
    }

    fn pack_env(&self, key: &str, value: &str) -> String {
        if value.is_empty() {
            format!("export {}=", key)
        } else {
            format!("export {}=\"{}\"", key, value)
        }
    }
}

// ---------------------------------------------------------------------------
// ZshShell
// ---------------------------------------------------------------------------

pub struct ZshShell;

impl Sheller for ZshShell {
    fn conf_path(&self) -> PathBuf {
        expand_tilde("~/.zshrc")
    }

    fn vm_env_conf_path(&self) -> PathBuf {
        vmr_work_dir().join(VM_ENV_FILE_NAME)
    }

    fn write_vm_env_to_shell(&self) {
        let vm_env_path = format_path_string(
            self.vm_env_conf_path().to_str().unwrap_or("~/.vmr/vmr"),
        );
        let source_line = bash_zsh_source_line(&vm_env_path);
        append_source_line_to_config(&self.conf_path(), &source_line);
    }

    fn pack_path(&self, path: &str) -> String {
        format!("export PATH=\"{}:$PATH\"", path)
    }

    fn pack_env(&self, key: &str, value: &str) -> String {
        if value.is_empty() {
            format!("export {}=", key)
        } else {
            format!("export {}=\"{}\"", key, value)
        }
    }
}

// ---------------------------------------------------------------------------
// FishShell
// ---------------------------------------------------------------------------

pub struct FishShell;

impl Sheller for FishShell {
    fn conf_path(&self) -> PathBuf {
        expand_tilde("~/.config/fish/config.fish")
    }

    fn vm_env_conf_path(&self) -> PathBuf {
        vmr_work_dir().join(VM_ENV_FILE_NAME)
    }

    fn write_vm_env_to_shell(&self) {
        let vm_env_path = format_path_string(
            self.vm_env_conf_path().to_str().unwrap_or("~/.vmr/vmr"),
        );
        let source_line = fish_source_line(&vm_env_path);
        append_source_line_to_config(&self.conf_path(), &source_line);
    }

    fn pack_path(&self, path: &str) -> String {
        format!("fish_add_path \"{}\"", path)
    }

    fn pack_env(&self, key: &str, value: &str) -> String {
        if value.is_empty() {
            format!("set -gx {} ", key)
        } else {
            format!("set -gx {} \"{}\"", key, value)
        }
    }
}
