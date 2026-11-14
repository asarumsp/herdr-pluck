use super::error::ClipboardError;
use super::runner::ClipboardCommandRunner;
use super::tool::ClipboardTool;
use std::process::{Command, Stdio};

/// `std::process`-backed clipboard command runner for normal operation.
#[derive(Debug, Default, Clone, Copy)]
pub(crate) struct SystemCommandRunner;

impl ClipboardCommandRunner for SystemCommandRunner {
    fn command_exists(&self, command: &str) -> bool {
        which::which(command).is_ok()
    }

    fn run_with_stdin(&self, tool: ClipboardTool, stdin: &str) -> Result<(), ClipboardError> {
        let mut command = Command::new(tool.name);
        command.args(tool.args).stdin(Stdio::piped());
        detach_from_session(&mut command);
        let mut child = command.spawn().map_err(|err| ClipboardError::SpawnFailed {
            tool: tool.name.to_string(),
            message: err.to_string(),
        })?;

        if let Some(mut child_stdin) = child.stdin.take() {
            use std::io::Write;
            child_stdin
                .write_all(stdin.as_bytes())
                .map_err(|err| ClipboardError::WriteFailed {
                    tool: tool.name.to_string(),
                    message: err.to_string(),
                })?;
        }

        let status = child.wait().map_err(|err| ClipboardError::WaitFailed {
            tool: tool.name.to_string(),
            message: err.to_string(),
        })?;

        if status.success() {
            Ok(())
        } else {
            Err(ClipboardError::CommandFailed {
                tool: tool.name.to_string(),
                status: status.to_string(),
            })
        }
    }
}

/// Runs the clipboard tool in its own session so tools that keep serving the
/// selection after exit (e.g. `wl-copy`) survive the picker tab closing.
#[cfg(unix)]
fn detach_from_session(command: &mut Command) {
    use std::os::unix::process::CommandExt;
    // SAFETY: `setsid` is async-signal-safe and touches no parent state.
    unsafe {
        command.pre_exec(|| {
            if libc::setsid() == -1 {
                return Err(std::io::Error::last_os_error());
            }
            Ok(())
        });
    }
}

#[cfg(not(unix))]
fn detach_from_session(_command: &mut Command) {}
