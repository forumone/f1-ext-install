//! Helpers for interacting with system commands.

use quick_error::{quick_error, ResultExt as _};
use std::{
    borrow::Cow,
    convert::Into,
    io,
    os::unix::process::ExitStatusExt as _,
    process::{Command as SystemCommand, ExitStatus, Stdio},
    str::Utf8Error,
    string::FromUtf8Error,
};

/// Returns a message indicating the cause of a process exit.
fn exit_status_reason(status: ExitStatus) -> Cow<'static, str> {
    if let Some(code) = status.code() {
        format!("non-zero exit code {}", code).into()
    } else if let Some(signal) = status.signal() {
        format!("killed by signal {}", signal).into()
    } else {
        "unknown reason".into()
    }
}

quick_error! {
    #[derive(Debug)]
    /// Indicates how a process failed.
    pub enum CommandError {
        /// General errors from `std::io`, usually indicating a failure to start a process.
        Io(command: String, err: io::Error) {
            context(command: &'a str, err: io::Error) -> (command.to_owned(), err)
            cause(err)
            display("I/O Error: {}", err)
        }

        /// Indicates that a process exited with a non-zero code. On *nix systems, also
        /// indicates death by signal.
        BadExit(exit: ExitStatus) {
            from()
            display("Process exited unsuccessfully: {}", exit_status_reason(*exit))
        }

        /// Indicates that process output could not be decoded as valid UTF-8.
        Utf8(err: Utf8Error) {
            cause(err)
            from()
            from(err: FromUtf8Error) -> (err.utf8_error())
            display("UTF-8 error: {}", err)
        }
    }
}

/// Helper type for the result of command execution.
pub type Result<T> = std::result::Result<T, CommandError>;

/// Extension trait for converting process exit codes into `Result`s.
///
/// This trait is the glue between a normal `ExitStatus` and `CommandError::BadExit`.
pub trait ExitStatusExt {
    /// Convert this object into a `Result<Self, CommandError>`.
    ///
    /// If the status is anything other than succesful, this method is expected to return
    /// an `Err(CommandError::BadExit(exit))`. Otherwise, `Ok(self)` is returned. This
    /// makes handling simple cases of failure more ergonomic.
    fn into_result(self) -> Result<Self>
    where
        Self: Sized;
}

impl ExitStatusExt for ExitStatus {
    fn into_result(self) -> Result<Self> {
        if self.success() {
            Ok(self)
        } else {
            Err(self.into())
        }
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
        let status = command.status().context(program)?;

        let status = status.into_result()?;
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

        let output = command.output().context(program)?;

        let _ = output.status.into_result()?;

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
