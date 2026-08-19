use std::io::{ErrorKind, Read};
use std::os::fd::AsFd;
use std::os::unix::process::CommandExt;
use std::path::Path;
use std::process::{Child, ChildStderr, ChildStdout, Command, ExitStatus, Stdio};
use std::thread;
use std::time::{Duration, Instant};

const OUTPUT_LIMIT: usize = 1024 * 1024;
const TIMEOUT: Duration = Duration::from_secs(5);

#[derive(Clone, Copy)]
enum StreamFailure {
    Limit,
    Read,
}

struct BoundedStream<R> {
    stream: R,
    output: Vec<u8>,
    complete: bool,
}

struct OutputStreams {
    stdout: BoundedStream<ChildStdout>,
    stderr: BoundedStream<ChildStderr>,
}

pub(super) fn run(home: &Path, args: &[&str], subject: &str) -> Result<Vec<u8>, String> {
    let mut command = Command::new("codex");
    command
        .args(args)
        .current_dir(home)
        .env("HOME", home)
        .env("CODEX_HOME", home.join(".codex"))
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .process_group(0);
    let mut child = command
        .spawn()
        .map_err(|_| format!("Codex {subject} resolver could not be started"))?;
    let mut streams = match OutputStreams::new(&mut child) {
        Ok(streams) => streams,
        Err(()) => {
            terminate(&mut child);
            return Err(format!("Codex {subject} resolver output could not be read"));
        }
    };
    let status = wait_for_output(&mut child, &mut streams, subject)?;
    if !status.success() {
        return Err(exit_detail(subject, status));
    }
    Ok(streams.stdout.output)
}

fn wait_for_output(
    child: &mut Child,
    streams: &mut OutputStreams,
    subject: &str,
) -> Result<ExitStatus, String> {
    let started = Instant::now();
    let mut status = None;
    loop {
        if let Err(failure) = streams.drain() {
            terminate(child);
            return Err(stream_detail(subject, failure));
        }
        if status.is_none() {
            status = match child.try_wait() {
                Ok(status) => status,
                Err(_) => {
                    terminate(child);
                    return Err(format!("Codex {subject} resolver status could not be read"));
                }
            };
        }
        if let Some(status) = status.filter(|_| streams.complete()) {
            return Ok(status);
        }
        if started.elapsed() >= TIMEOUT {
            terminate(child);
            return Err(format!("Codex {subject} resolver timed out"));
        }
        thread::sleep(Duration::from_millis(10));
    }
}

impl OutputStreams {
    fn new(child: &mut Child) -> Result<Self, ()> {
        let stdout = child.stdout.take().ok_or(())?;
        let stderr = child.stderr.take().ok_or(())?;
        set_nonblocking(&stdout)?;
        set_nonblocking(&stderr)?;
        Ok(Self {
            stdout: BoundedStream::new(stdout),
            stderr: BoundedStream::new(stderr),
        })
    }

    fn drain(&mut self) -> Result<(), StreamFailure> {
        self.stdout.drain()?;
        self.stderr.drain()
    }

    fn complete(&self) -> bool {
        self.stdout.complete && self.stderr.complete
    }
}

impl<R: Read> BoundedStream<R> {
    fn new(stream: R) -> Self {
        Self {
            stream,
            output: Vec::new(),
            complete: false,
        }
    }

    fn drain(&mut self) -> Result<(), StreamFailure> {
        let mut buffer = [0; 8192];
        while !self.complete {
            match self.stream.read(&mut buffer) {
                Ok(0) => self.complete = true,
                Ok(read) if self.output.len() + read > OUTPUT_LIMIT => {
                    return Err(StreamFailure::Limit);
                }
                Ok(read) => self.output.extend_from_slice(&buffer[..read]),
                Err(error) if error.kind() == ErrorKind::WouldBlock => break,
                Err(error) if error.kind() == ErrorKind::Interrupted => {}
                Err(_) => return Err(StreamFailure::Read),
            }
        }
        Ok(())
    }
}

fn set_nonblocking(stream: &impl AsFd) -> Result<(), ()> {
    let flags = rustix::fs::fcntl_getfl(stream).map_err(|_| ())?;
    rustix::fs::fcntl_setfl(stream, flags | rustix::fs::OFlags::NONBLOCK).map_err(|_| ())
}

fn terminate(child: &mut Child) {
    let pid = rustix::process::Pid::from_child(child);
    let _ = rustix::process::kill_process_group(pid, rustix::process::Signal::KILL);
    let _ = child.wait();
}

fn exit_detail(subject: &str, status: ExitStatus) -> String {
    let status = status.code().map_or_else(
        || "from a signal".to_owned(),
        |code| format!("with status {code}"),
    );
    format!("Codex {subject} resolver exited {status}")
}

fn stream_detail(subject: &str, failure: StreamFailure) -> String {
    match failure {
        StreamFailure::Limit => format!("Codex {subject} resolver exceeded its output limit"),
        StreamFailure::Read => format!("Codex {subject} resolver output could not be read"),
    }
}
