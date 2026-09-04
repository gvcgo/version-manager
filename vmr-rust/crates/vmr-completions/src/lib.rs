//! Shell completions install (plan.md §3.10): writes the completion script to `~/.vmr/vmr_completions.*`
//! and appends a source/import block to the shell configuration (the `# VMR Completions` marker).

use std::fs;

use vmr_core::paths;

/// Supported shells (mirrors plan §3.10 and the Go side).
#[derive(Debug, Clone, Copy)]
pub enum ShellKind {
    Bash,
    Zsh,
    Fish,
    PowerShell,
}

pub fn shell_kind_from_str(s: &str) -> Option<ShellKind> {
    match s.to_lowercase().as_str() {
        "bash" => Some(ShellKind::Bash),
        "zsh" => Some(ShellKind::Zsh),
        "fish" => Some(ShellKind::Fish),
        "powershell" | "ps1" | "power-shell" => Some(ShellKind::PowerShell),
        _ => None,
    }
}

fn home() -> String {
    #[cfg(windows)]
    {
        std::env::var("USERPROFILE").unwrap_or_default()
    }
    #[cfg(not(windows))]
    {
        std::env::var("HOME").unwrap_or_default()
    }
}

/// The completion target file and shell configuration paths.
fn targets(kind: ShellKind) -> (PathBuf, PathBuf) {
    let h = PathBuf::from(home());
    match kind {
        ShellKind::Bash => (
            paths::work_dir().join("vmr_completions.sh"),
            h.join(".bashrc"),
        ),
        ShellKind::Zsh => (
            paths::work_dir().join("vmr_completions.sh"),
            h.join(".zshrc"),
        ),
        ShellKind::Fish => (
            paths::work_dir().join("vmr_completions.fish"),
            h.join(".config/fish/config.fish"),
        ),
        ShellKind::PowerShell => (
            paths::work_dir().join("vmr_completions.ps1"),
            h.join("Documents/WindowsPowerShell/profile.ps1"),
        ),
    }
}

/// Appends the completions source/import block to the configuration (idempotent via `# VMR Completions`).
pub fn install_completions(kind: ShellKind, script: &str) -> Result<(), String> {
    let (file, conf) = targets(kind);
    fs::create_dir_all(paths::work_dir()).map_err(|e| e.to_string())?;
    fs::write(&file, script).map_err(|e| e.to_string())?;

    let block = match kind {
        ShellKind::Bash | ShellKind::Zsh => {
            format!("# VMR Completions\n. {}\n# VMR Completions", file.display())
        }
        ShellKind::Fish => format!(
            "# VMR Completions\nsource {}\n# VMR Completions",
            file.display()
        ),
        ShellKind::PowerShell => format!(
            "# VMR Completions\nImport-Module {}\n# VMR Completions",
            file.display()
        ),
    };
    let data = fs::read_to_string(&conf).unwrap_or_default();
    if data.contains("# VMR Completions") {
        return Ok(()); // already installed (idempotent)
    }
    let out = if data.trim().is_empty() {
        block
    } else {
        format!("{}\n{block}", data.trim_end())
    };
    if let Some(parent) = conf.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    fs::write(&conf, out).map_err(|e| e.to_string())
}

use std::path::PathBuf;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kind_parse() {
        assert!(matches!(shell_kind_from_str("zsh"), Some(ShellKind::Zsh)));
        assert!(shell_kind_from_str("powershell").is_some());
        assert!(shell_kind_from_str("tcsh").is_none());
    }
}
