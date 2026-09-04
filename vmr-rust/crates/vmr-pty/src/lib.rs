//! vmr-pty: interactive sub-shell in session mode (plan.md §3.9).
//!
//! Mirrors Go `internal/terminal`: sets `VM_DISABLE=111` before entering the
//! interactive shell (preventing the shell hook from re-injecting the SDK
//! environment), while the child process inherits the current process
//! environment (the caller has already injected the temporary PATH/env or
//! removed the global SDK paths).
//! Returns the child process's exit code after the session exits.
//!
//! Note: the Rust side implements interaction with a direct child process
//! (inheriting stdio) rather than introducing portable-pty's
//! master/slave terminal layer; the behavior is equivalent for CLI sessions
//! (foreground interaction) — the sub-shell occupies the terminal directly.
//! The Windows default shell goes through `cmd`.

use std::process::Command;

pub const VM_DISABLE_ENV: &str = "VM_DISABLE";

/// Enters an interactive sub-shell; returns the child process's exit code (1 when the shell cannot be found).
pub fn run_terminal() -> i32 {
    let shell = detect_shell();
    let mut cmd = Command::new(&shell.0);
    cmd.args(&shell.1);
    cmd.env(VM_DISABLE_ENV, "111");
    cmd.stdin(std::process::Stdio::inherit());
    cmd.stdout(std::process::Stdio::inherit());
    cmd.stderr(std::process::Stdio::inherit());
    match cmd.status() {
        Ok(st) => st.code().unwrap_or(1),
        Err(e) => {
            eprintln!("failed to start shell {shell:?}: {e}");
            1
        }
    }
}

/// Detects the login shell (the SHELL env or the platform default).
fn detect_shell() -> (String, Vec<String>) {
    if let Ok(sh) = std::env::var("SHELL") {
        if !sh.is_empty() {
            return (sh, Vec::new());
        }
    }
    if cfg!(windows) {
        (
            "cmd".to_string(),
            vec!["/c".to_string(), "start".to_string(), "cmd".to_string()],
        )
    } else {
        ("/bin/sh".to_string(), Vec::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shell_detect_returns_something() {
        let (bin, _) = detect_shell();
        assert!(!bin.is_empty());
    }
}
