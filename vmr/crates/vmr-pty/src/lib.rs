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
//! On Windows the shell is taken from the parent process (PowerShell, pwsh,
//! git-bash, cmd), mirroring Go's `terminal.RunTerminal`; it must be spawned in
//! the current console (never via `start`, which opens a detached window and
//! returns before the session ends).

use std::process::Command;

pub const VM_DISABLE_ENV: &str = "VM_DISABLE";

/// Enters an interactive sub-shell; returns the child process's exit code (1 when the shell
/// cannot be found). When already inside a vmr session, returns 0 without spawning another
/// shell unless nested sessions are explicitly allowed (Go `terminal.RunTerminal`).
pub fn run_terminal() -> i32 {
    let in_session = std::env::var(VM_DISABLE_ENV)
        .map(|v| !v.is_empty())
        .unwrap_or(false);
    let allow_nested = std::env::var("VMR_ALLOW_NESTED_SESSIONS")
        .map(|v| matches!(v.to_lowercase().as_str(), "true" | "1" | "t" | "yes" | "on"))
        .unwrap_or(false);
    if in_session && !allow_nested {
        return 0;
    }

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
#[cfg(not(windows))]
fn detect_shell() -> (String, Vec<String>) {
    if let Ok(sh) = std::env::var("SHELL") {
        if !sh.is_empty() {
            return (sh, Vec::new());
        }
    }
    ("/bin/sh".to_string(), Vec::new())
}

/// Windows: reuse the parent shell so the session behaves like the invoking console.
/// `SHELL` is a POSIX path there (git-bash sets `/usr/bin/bash`), so it cannot be used
/// directly; the executable name of the parent process is the reliable source.
#[cfg(windows)]
fn detect_shell() -> (String, Vec<String>) {
    if let Some(name) = parent_process_name() {
        let lower = name.to_ascii_lowercase();
        if lower.contains("pwsh") {
            return ("pwsh.exe".to_string(), Vec::new());
        }
        if lower.contains("powershell") {
            return ("powershell.exe".to_string(), Vec::new());
        }
        if lower.contains("bash") || lower == "sh.exe" {
            return ("bash.exe".to_string(), Vec::new());
        }
        if lower.ends_with("zsh.exe") {
            return ("zsh.exe".to_string(), Vec::new());
        }
        if lower.ends_with("fish.exe") {
            return ("fish.exe".to_string(), Vec::new());
        }
        if lower.ends_with("cmd.exe") {
            return (comspec(), Vec::new());
        }
    }
    (comspec(), Vec::new())
}

#[cfg(windows)]
fn comspec() -> String {
    std::env::var("COMSPEC")
        .ok()
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "cmd.exe".to_string())
}

/// Executable name of the current process's parent, via a Toolhelp snapshot.
#[cfg(windows)]
fn parent_process_name() -> Option<String> {
    use windows_sys::Win32::Foundation::{CloseHandle, INVALID_HANDLE_VALUE};
    use windows_sys::Win32::System::Diagnostics::ToolHelp::{
        CreateToolhelp32Snapshot, PROCESSENTRY32W, Process32FirstW, Process32NextW,
        TH32CS_SNAPPROCESS,
    };

    unsafe {
        let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
        if snapshot == INVALID_HANDLE_VALUE {
            return None;
        }
        let mut entry: PROCESSENTRY32W = std::mem::zeroed();
        entry.dwSize = std::mem::size_of::<PROCESSENTRY32W>() as u32;
        let mut processes = Vec::new();
        if Process32FirstW(snapshot, &mut entry) != 0 {
            loop {
                let end = entry
                    .szExeFile
                    .iter()
                    .position(|&c| c == 0)
                    .unwrap_or(entry.szExeFile.len());
                processes.push((
                    entry.th32ProcessID,
                    entry.th32ParentProcessID,
                    String::from_utf16_lossy(&entry.szExeFile[..end]),
                ));
                if Process32NextW(snapshot, &mut entry) == 0 {
                    break;
                }
            }
        }
        CloseHandle(snapshot);
        let current = std::process::id();
        let parent_id = processes.iter().find(|(pid, _, _)| *pid == current)?.1;
        processes
            .into_iter()
            .find(|(pid, _, _)| *pid == parent_id)
            .map(|(_, _, name)| name)
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

    #[test]
    #[cfg(windows)]
    fn windows_session_shell_stays_in_the_current_console() {
        let (bin, args) = detect_shell();
        assert!(!bin.is_empty());
        assert!(
            !args.iter().any(|a| a.eq_ignore_ascii_case("start")),
            "session shell must not use `start`: {bin} {args:?}"
        );
    }
}
