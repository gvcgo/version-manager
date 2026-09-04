//! System command execution (mirrors Go `vmr-go/internal/utils/exec.go` + goutils `ExecuteSysCommand`).
//!
//! - Windows: `args` is run via `cmd` with `/c` prepended; other platforms execute `args[0]` directly.
//! - `collect_output`: stdout is captured into an internal buffer, otherwise the parent's stdout is
//!   inherited; stderr/stdin are always inherited (mirrors Go).
//! - Environment: fully inherits the current process environment (Rust `Command`'s default behavior,
//!   mirroring Go's `cmd.Env = os.Environ()`).
//! - Go's unix `FlushPathEnvForUnix()` is effectively a no-op (a `source ~/.bashrc` child process
//!   cannot modify the parent's environment, and errors are ignored), so it is omitted on the Rust side.

use std::io;
use std::process::{Child, Command, Stdio};

/// Command runner (mirrors Go `SysCommandRunner`).
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

    /// Spawn the child process (mirrors the spawn phase of Go `Run`; `wait` must follow).
    pub fn spawn(&mut self) -> io::Result<()> {
        if self.child.is_some() {
            return Ok(());
        }
        self.child = Some(self.command().spawn()?);
        Ok(())
    }

    /// Wait for the child to finish and store its stdout in the buffer; returns whether the exit
    /// status was successful.
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

    /// Blocking execution (spawn + wait).
    pub fn run(&mut self) -> io::Result<()> {
        self.spawn()?;
        self.wait()
    }

    /// Terminate the child process (mirrors Go `Cancel`; call it while another thread is waiting).
    pub fn cancel(&mut self) {
        if let Some(child) = self.child.as_mut() {
            let _ = child.kill();
        }
    }

    /// Collected stdout; `None` if not collected or not yet finished.
    pub fn get_output(&self) -> Option<&str> {
        self.output
            .as_deref()
            .and_then(|b| std::str::from_utf8(b).ok())
    }
}

/// One-shot command execution: an empty `work_dir` leaves the working directory unset.
///
/// Mirrors goutils `ExecuteSysCommand(collectOutput, workDir, args...)`:
/// with collect, the collected stdout text is returned; without collect, an empty string is returned.
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
