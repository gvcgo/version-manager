//! vmr-pty：会话模式交互子 shell（plan.md §3.9）。
//!
//! 对齐 Go `internal/terminal`：进入交互 shell 前设 `VM_DISABLE=111`
//! （阻止 shell hook 重复注入 SDK 环境），子进程继承当前进程环境
//! （调用方已注入临时 PATH/env 或摘除全局 SDK 路径）。
//! 会话退出后返回子进程退出码。
//!
//! 说明：Rust 侧以直接子进程（继承 stdio）实现交互，未引入 portable-pty
//! 主从终端层；对 CLI 会话（前台交互）行为等价——子 shell 直接占用终端。
//! Windows 默认 shell 走 `cmd`。

use std::process::Command;

pub const VM_DISABLE_ENV: &str = "VM_DISABLE";

/// 进入交互子 shell；返回子进程退出码（找不到 shell 时 1）。
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

/// 探测登录 shell（SHELL env 或平台默认）。
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
