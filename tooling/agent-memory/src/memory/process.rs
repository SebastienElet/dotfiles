use std::ffi::{OsStr, OsString};
use std::io;
use std::path::Path;
use std::process::Command;
use std::time::Duration;

mod deadline;

pub use deadline::DeadlineProcessRunner;

#[derive(Debug)]
pub struct ProcessOutput {
    success: bool,
    code: Option<i32>,
    stdout: Vec<u8>,
    stderr: Vec<u8>,
}

impl ProcessOutput {
    #[must_use]
    pub const fn new(success: bool, code: Option<i32>, stdout: Vec<u8>, stderr: Vec<u8>) -> Self {
        Self {
            success,
            code,
            stdout,
            stderr,
        }
    }

    #[must_use]
    pub const fn success(&self) -> bool {
        self.success
    }

    #[must_use]
    pub const fn code(&self) -> Option<i32> {
        self.code
    }

    #[must_use]
    pub fn stdout(&self) -> &[u8] {
        &self.stdout
    }

    #[must_use]
    pub fn stderr(&self) -> &[u8] {
        &self.stderr
    }
}

pub trait ProcessRunner {
    /// # Errors
    ///
    /// Returns an I/O error if spawning or supervising the process, capturing output, or deadline cleanup fails.
    fn run(
        &self,
        program: &OsStr,
        arguments: &[OsString],
        current_directory: Option<&Path>,
    ) -> io::Result<ProcessOutput>;

    fn remaining_time(&self) -> Option<Duration> {
        None
    }
}

pub struct SystemProcessRunner;

impl ProcessRunner for SystemProcessRunner {
    fn run(
        &self,
        program: &OsStr,
        arguments: &[OsString],
        current_directory: Option<&Path>,
    ) -> io::Result<ProcessOutput> {
        let mut command = Command::new(program);
        command.args(arguments);
        if let Some(directory) = current_directory {
            command.current_dir(directory);
        }
        let output = command.output()?;
        Ok(ProcessOutput::new(
            output.status.success(),
            output.status.code(),
            output.stdout,
            output.stderr,
        ))
    }
}
