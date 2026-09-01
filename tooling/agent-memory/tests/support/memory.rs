use agent_memory::{ProcessOutput, ProcessRunner};
use std::cell::RefCell;
use std::collections::VecDeque;
use std::ffi::{OsStr, OsString};
use std::fs;
use std::io;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProcessCall {
    pub program: OsString,
    pub arguments: Vec<OsString>,
    pub current_directory: Option<PathBuf>,
}

pub struct FakeResponse {
    pub success: bool,
    pub code: Option<i32>,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
    pub body: Option<Vec<u8>>,
    pub error: Option<io::ErrorKind>,
}

impl FakeResponse {
    pub fn success(stdout: impl Into<Vec<u8>>) -> Self {
        Self {
            success: true,
            code: Some(0),
            stdout: stdout.into(),
            stderr: Vec::new(),
            body: None,
            error: None,
        }
    }

    pub fn failure(code: i32, stdout: impl Into<Vec<u8>>) -> Self {
        Self {
            success: false,
            code: Some(code),
            stdout: stdout.into(),
            stderr: b"redacted process failure".to_vec(),
            body: None,
            error: None,
        }
    }

    pub fn missing() -> Self {
        Self {
            success: false,
            code: None,
            stdout: Vec::new(),
            stderr: Vec::new(),
            body: None,
            error: Some(io::ErrorKind::NotFound),
        }
    }

    #[allow(dead_code)]
    pub fn with_body(mut self, body: impl Into<Vec<u8>>) -> Self {
        self.body = Some(body.into());
        self
    }
}

#[derive(Default)]
pub struct FakeProcessRunner {
    responses: RefCell<VecDeque<FakeResponse>>,
    calls: RefCell<Vec<ProcessCall>>,
    output_files: RefCell<Vec<(PathBuf, u32)>>,
}

impl FakeProcessRunner {
    pub fn with_responses(responses: impl IntoIterator<Item = FakeResponse>) -> Self {
        Self {
            responses: RefCell::new(responses.into_iter().collect()),
            calls: RefCell::new(Vec::new()),
            output_files: RefCell::new(Vec::new()),
        }
    }

    pub fn calls(&self) -> Vec<ProcessCall> {
        self.calls.borrow().clone()
    }

    #[allow(dead_code)]
    pub fn output_files(&self) -> Vec<(PathBuf, u32)> {
        self.output_files.borrow().clone()
    }
}

impl ProcessRunner for FakeProcessRunner {
    fn run(
        &self,
        program: &OsStr,
        arguments: &[OsString],
        current_directory: Option<&Path>,
    ) -> io::Result<ProcessOutput> {
        self.calls.borrow_mut().push(ProcessCall {
            program: program.to_owned(),
            arguments: arguments.to_vec(),
            current_directory: current_directory.map(Path::to_owned),
        });
        let response = self.responses.borrow_mut().pop_front().unwrap();
        if let Some(error) = response.error {
            return Err(io::Error::from(error));
        }
        if let Some(body) = response.body {
            let output = output_path(arguments);
            let mode = fs::metadata(&output).unwrap().permissions().mode() & 0o777;
            self.output_files.borrow_mut().push((output.clone(), mode));
            fs::write(output, body).unwrap();
        }
        Ok(ProcessOutput::new(
            response.success,
            response.code,
            response.stdout,
            response.stderr,
        ))
    }
}

fn output_path(arguments: &[OsString]) -> PathBuf {
    let index = arguments
        .iter()
        .position(|argument| argument == "--output")
        .unwrap();
    PathBuf::from(&arguments[index + 1])
}

pub fn git(directory: &Path, arguments: &[&str]) {
    let status = std::process::Command::new("git")
        .arg("-C")
        .arg(directory)
        .args(arguments)
        .status()
        .unwrap();
    assert!(status.success(), "git invocation failed: {arguments:?}");
}
