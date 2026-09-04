//! 系统命令执行（对齐 Go `vmr-go/internal/utils/exec.go` + goutils `ExecuteSysCommand`）。
//!
//! - Windows：`args` 前插 `/c` 后经 `cmd` 执行；其余平台直接执行 `args[0]`。
//! - `collect_output`：stdout 收集到内部缓冲，否则继承父进程 stdout；
//!   stderr/stdin 恒继承（对齐 Go）。
//! - 环境：完整继承当前进程环境（Rust `Command` 默认行为，对齐 Go `cmd.Env = os.Environ()`）。
//! - Go 的 unix `FlushPathEnvForUnix()` 实为无效调用（`source ~/.bashrc` 子进程
//!   无法修改父进程环境、错误被忽略），Rust 侧省略。

use std::io;
use std::process::{Child, Command, Stdio};

/// 命令执行器（对齐 Go `SysCommandRunner`）。
pub struct SysCommandRunner {
    args: Vec<String>,
    work_dir: Option<String>,
    collect_output: bool,
    child: Option<Child>,
    output: Option<Vec<u8>>,
}

impl SysCommandRunner {
    pub fn new(collect_output: bool, work_dir: &str, args: Vec<String>) -> Self {
        SysCommandRunner {
            args,
            work_dir: if work_dir.is_empty() {
                None
            } else {
                Some(work_dir.to_string())
            },
            collect_output,
            child: None,
            output: None,
        }
    }

    fn command(&self) -> Command {
        let mut cmd = if cfg!(windows) {
            let mut c = Command::new("cmd");
            c.arg("/c").args(&self.args);
            c
        } else {
            let mut it = self.args.iter();
            let program = it.next().cloned().unwrap_or_default();
            let mut c = Command::new(program);
            c.args(it);
            c
        };
        if let Some(dir) = &self.work_dir {
            cmd.current_dir(dir);
        }
        cmd.stdin(Stdio::inherit()).stderr(Stdio::inherit());
        cmd.stdout(if self.collect_output {
            Stdio::piped()
        } else {
            Stdio::inherit()
        });
        cmd
    }

    /// 启动子进程（对齐 Go `Run` 的 spawn 阶段；需随后 `wait`）。
    pub fn spawn(&mut self) -> io::Result<()> {
        if self.child.is_some() {
            return Ok(());
        }
        self.child = Some(self.command().spawn()?);
        Ok(())
    }

    /// 等待子进程结束并把 stdout 存入缓冲；返回退出状态是否成功。
    pub fn wait(&mut self) -> io::Result<()> {
        let mut child = self
            .child
            .take()
            .ok_or_else(|| io::Error::other("command not started"))?;
        if self.collect_output {
            let out = child.wait_with_output()?;
            self.output = Some(out.stdout);
            if !out.status.success() {
                return Err(io::Error::other(format!(
                    "command exited with {}",
                    out.status
                )));
            }
        } else {
            let status = child.wait()?;
            if !status.success() {
                return Err(io::Error::other(format!("command exited with {status}")));
            }
        }
        Ok(())
    }

    /// 阻塞执行（spawn + wait）。
    pub fn run(&mut self) -> io::Result<()> {
        self.spawn()?;
        self.wait()
    }

    /// 终止子进程（对齐 Go `Cancel`；须在另一线程等待时调用）。
    pub fn cancel(&mut self) {
        if let Some(child) = self.child.as_mut() {
            let _ = child.kill();
        }
    }

    /// 收集到的 stdout；未收集或尚未结束返回 `None`。
    pub fn get_output(&self) -> Option<&str> {
        self.output
            .as_deref()
            .and_then(|b| std::str::from_utf8(b).ok())
    }
}

/// 一次性命令执行：`work_dir` 为空则不设置工作目录。
///
/// 对齐 goutils `ExecuteSysCommand(collectOutput, workDir, args...)`：
/// collect 时返回收集到的 stdout 文本；非 collect 时返回空串。
pub fn exec(collect_output: bool, work_dir: &str, args: &[String]) -> io::Result<String> {
    let mut runner = SysCommandRunner::new(collect_output, work_dir, args.to_vec());
    runner.run()?;
    Ok(runner.get_output().unwrap_or_default().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn collects_stdout() {
        let out = exec(true, "", &["echo".into(), "hello-vmr".into()]).unwrap();
        assert_eq!(out.trim(), "hello-vmr");
    }

    #[test]
    fn non_collect_returns_empty() {
        let out = exec(false, "", &["echo".into(), "hi".into()]).unwrap();
        assert_eq!(out, "");
    }

    #[test]
    fn non_zero_exit_is_error() {
        let err = exec(true, "", &["sh".into(), "-c".into(), "exit 3".into()]).unwrap_err();
        assert!(err.to_string().contains("exit"), "{err}");
    }

    #[test]
    fn missing_program_is_error() {
        assert!(exec(true, "", &["definitely-not-a-cmd-xyz".into()]).is_err());
    }

    #[test]
    fn work_dir_is_honored() {
        let out = exec(true, "/", &["pwd".into()]).unwrap();
        assert_eq!(out.trim(), "/");
    }
}
