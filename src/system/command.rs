//! Helpers for interacting with system commands.

use snafu::{ResultExt, Snafu};
use std::{
    convert::Into,
    io,
    os::unix::process::ExitStatusExt as _,
    process::{Command as SystemCommand, ExitStatus, Stdio},
    string::FromUtf8Error,
};

/// Returns a message indicating the cause of a process exit.
fn exit_status_reason(status: ExitStatus) -> String {
    if let Some(code) = status.code() {
        format!("non-zero exit code {}", code)
    } else if let Some(signal) = status.signal() {
        format!("killed by signal {}", signal)
    } else {
        String::from("unknown reason")
    }
}

#[derive(Debug, Snafu)]
/// Indicates how a process failed.
pub enum CommandError {
    /// General errors from `std::io`, usually indicating a failure to start a process.
    #[snafu(display("Failed to run {}: {}", command, source))]
    Io {
        /// The underlying IO error
        source: io::Error,
        /// The command that failed
        command: String,
    },

    /// Indicates that a process exited with a non-zero code. On *nix systems, also
    /// indicates death by signal.
    #[snafu(display("{} exited unsuccessfully: {}", command, exit_status_reason(*exit)))]
    BadExit {
        /// The command that failed
        command: String,
        /// The exit cause
        exit: ExitStatus,
    },

    /// Indicates that process output could not be decoded as valid UTF-8.
    #[snafu(display("UTF-8 error: {}", source))]
    Utf8 {
        /// The underlying UTF-8 error
        source: FromUtf8Error
    },
}

// For some reason, snafu won't generate this automatically
impl From<FromUtf8Error> for CommandError {
    fn from(err: FromUtf8Error) -> Self {
        Self::Utf8 { source: err }
    }
}

/// Helper type for the result of command execution.
pub type Result<T> = std::result::Result<T, CommandError>;

/// Convert an `ExitStatus` into a Result, using `command` for context to the user
fn status_result(status: ExitStatus, command: &str) -> Result<ExitStatus> {
    if status.success() {
        Ok(status)
    } else {
        Err(CommandError::BadExit {
            command: String::from(command),
            exit: status,
        })
    }
}

/// Helper type to construct new commands.
///
/// The program name and arguments are captured in an introspectable way for debugging
/// and dry-running in test form.
#[derive(Debug)]
pub struct Command<'a> {
    /// The program to execute (this is `argv[0]` in C parlance).
    program: &'a str,
    /// The arguments to pass to the program, if any.
    args: Vec<String>,
}

impl<'a> Command<'a> {
    /// Create a new `Command` struct that will execute the given `program`.
    pub fn new(program: &'a str) -> Self {
        Command {
            program,
            args: Vec::new(),
        }
    }

    /// Add an argument to the program's argument list.
    pub fn arg<S>(&mut self, arg: S) -> &mut Self
    where
        S: AsRef<str> + 'a,
    {
        self.args.push(String::from(arg.as_ref()));
        self
    }

    /// Batch add arguments to the program's argument list.
    pub fn args<I, S>(&mut self, args: I) -> &mut Self
    where
        S: AsRef<str> + 'a,
        I: IntoIterator<Item = S>,
    {
        let args = args.into_iter().map(|arg| String::from(arg.as_ref()));

        self.args.extend(args);
        self
    }

    /// Execute the given command and wait for its status, returning `Err` on failed
    /// execution.
    pub fn status(self) -> Result<ExitStatus> {
        let program = self.program;
        let mut command: SystemCommand = self.into();
        let status = command.status().with_context(|| Io {
            command: String::from(program),
        })?;

        let status = status_result(status, program)?;
        Ok(status)
    }

    /// Execute the given command and wait for it to complete, discarding successful
    /// exit information.
    pub fn wait(self) -> Result<()> {
        let _ = self.status()?;

        Ok(())
    }

    /// Execute the given command and capture its standard output.
    ///
    /// This method closes stdin, inherits stderr (allowing the user to see any error messages
    /// before `f1-ext-install` reports an error and exits), and attempts to decode the
    /// captured standard out into UTF-8. Any errors encountered along the way (process exit
    /// or encoding issues) are propagated as `Err` results.
    pub fn stdout(self) -> Result<String> {
        let program = self.program;
        let mut command: SystemCommand = self.into();
        command
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit());

        let output = command.output().with_context(|| Io {
            command: String::from(program),
        })?;

        let _ = status_result(output.status, program)?;

        let buffer = String::from_utf8(output.stdout)?;

        Ok(buffer)
    }
}

impl<'a> Into<SystemCommand> for Command<'a> {
    fn into(self) -> SystemCommand {
        let mut command = SystemCommand::new(self.program);
        command.args(self.args);
        command
    }
}
