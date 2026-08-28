use std::ffi::{OsStr, OsString};
use std::io;
use std::path::Path;
use std::process::Command;

#[derive(Debug)]
pub struct ProcessOutput {
    success: bool,
    code: Option<i32>,
    stdout: Vec<u8>,
    stderr: Vec<u8>,
}

impl ProcessOutput {
    pub fn new(success: bool, code: Option<i32>, stdout: Vec<u8>, stderr: Vec<u8>) -> Self {
        Self {
            success,
            code,
            stdout,
            stderr,
        }
    }

    pub fn success(&self) -> bool {
        self.success
    }

    pub fn code(&self) -> Option<i32> {
        self.code
    }

    pub fn stdout(&self) -> &[u8] {
        &self.stdout
    }

    pub fn stderr(&self) -> &[u8] {
        &self.stderr
    }
}

pub trait ProcessRunner {
    fn run(
        &self,
        program: &OsStr,
        arguments: &[OsString],
        current_directory: Option<&Path>,
    ) -> io::Result<ProcessOutput>;
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
