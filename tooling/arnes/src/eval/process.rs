use rustix::process::{Pid, Signal, kill_process_group};
use std::{
    collections::BTreeMap,
    io::{Read, Write},
    os::unix::process::CommandExt,
    path::Path,
    process::{Child, Command, Stdio},
    sync::mpsc::{self, Receiver, SyncSender},
    thread,
    time::{Duration, Instant},
};

const OUTPUT_LIMIT_BYTES: usize = 4_194_304;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExecutionError {
    AgentFailed,
    Timeout,
    OutputLimit,
    ProtocolInvalid,
}

#[derive(Clone, Copy)]
pub struct CaptureOptions<'a> {
    pub cwd: &'a Path,
    pub env: &'a BTreeMap<String, String>,
    pub stdin: &'a str,
    pub timeout: Duration,
}

#[derive(Debug)]
pub struct CaptureResult {
    pub output: String,
    pub error: Option<ExecutionError>,
    pub failure_detail: Option<String>,
}

enum PipeEvent {
    Output(Vec<u8>),
    OutputClosed,
    InputClosed,
    Failed,
}

#[must_use]
pub fn capture(command: &Path, args: &[String], options: CaptureOptions<'_>) -> CaptureResult {
    let child = Command::new(command)
        .args(args)
        .current_dir(options.cwd)
        .env_clear()
        .envs(options.env)
        .process_group(0)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn();
    let mut child = match child {
        Ok(child) => child,
        Err(error) => {
            return CaptureResult {
                output: String::new(),
                error: Some(ExecutionError::AgentFailed),
                failure_detail: Some(format!("could not start process: {}", error.kind())),
            };
        }
    };
    let (Some(stdin), Some(stdout)) = (child.stdin.take(), child.stdout.take()) else {
        let _ = stop_group(&mut child);
        let _ = child.wait();
        return CaptureResult {
            output: String::new(),
            error: Some(ExecutionError::AgentFailed),
            failure_detail: Some("process pipes unavailable".into()),
        };
    };
    let (sender, receiver) = mpsc::sync_channel(4);
    thread::scope(|scope| {
        let input_sender = sender.clone();
        let input = thread::Builder::new().spawn_scoped(scope, move || {
            write_input(stdin, options.stdin, &input_sender);
        });
        let output =
            thread::Builder::new().spawn_scoped(scope, move || read_output(stdout, &sender));
        if input.is_err() || output.is_err() {
            let _ = stop_group(&mut child);
        }
        let mut result = monitor(&mut child, &receiver, options.timeout);
        for worker in [input, output] {
            if !matches!(worker.map(std::thread::ScopedJoinHandle::join), Ok(Ok(()))) {
                result.error.get_or_insert(ExecutionError::AgentFailed);
                result
                    .failure_detail
                    .get_or_insert_with(|| "process I/O worker failed".into());
            }
        }
        result
    })
}

fn write_input(mut input: impl Write, text: &str, sender: &SyncSender<PipeEvent>) {
    if input.write_all(text.as_bytes()).is_err() {
        let _ = sender.send(PipeEvent::Failed);
    }
    drop(input);
    let _ = sender.send(PipeEvent::InputClosed);
}

fn read_output(mut output: impl Read, sender: &SyncSender<PipeEvent>) {
    let mut bytes = [0_u8; 8192];
    loop {
        match output.read(&mut bytes) {
            Ok(0) => break,
            Ok(count) => {
                let Some(chunk) = bytes.get(..count) else {
                    let _ = sender.send(PipeEvent::Failed);
                    break;
                };
                if sender.send(PipeEvent::Output(chunk.to_vec())).is_err() {
                    return;
                }
            }
            Err(error) if error.kind() == std::io::ErrorKind::Interrupted => {}
            Err(_) => {
                let _ = sender.send(PipeEvent::Failed);
                break;
            }
        }
    }
    let _ = sender.send(PipeEvent::OutputClosed);
}

fn stop_group(child: &mut Child) -> Result<(), ExecutionError> {
    let pid = Pid::from_child(child);
    match kill_process_group(pid, Signal::KILL) {
        Ok(()) | Err(rustix::io::Errno::SRCH) => Ok(()),
        Err(_) => {
            let _ = child.kill();
            Err(ExecutionError::AgentFailed)
        }
    }
}

fn monitor(child: &mut Child, receiver: &Receiver<PipeEvent>, timeout: Duration) -> CaptureResult {
    let started = Instant::now();
    let mut output = Vec::new();
    let mut failure = None;
    let mut closed = 0;
    let mut exited = false;
    let mut status = None;
    let mut stopped = false;
    while !exited || closed < 2 {
        if !exited {
            match child.try_wait() {
                Ok(Some(exit_status)) => {
                    exited = true;
                    status = Some(exit_status);
                }
                Err(_) => {
                    failure.get_or_insert(ExecutionError::AgentFailed);
                    exited = true;
                }
                _ => {}
            }
        }
        if started.elapsed() >= timeout {
            failure.get_or_insert(ExecutionError::Timeout);
        }
        if !stopped && failure.is_some() {
            if let Err(error) = stop_group(child) {
                failure.get_or_insert(error);
            }
            stopped = true;
        }
        match receiver.recv_timeout(Duration::from_millis(5)) {
            Ok(PipeEvent::Output(bytes)) => {
                if output.len() + bytes.len() > OUTPUT_LIMIT_BYTES {
                    failure.get_or_insert(ExecutionError::OutputLimit);
                } else if failure != Some(ExecutionError::OutputLimit) {
                    output.extend(bytes);
                }
            }
            Ok(PipeEvent::OutputClosed | PipeEvent::InputClosed) => closed += 1,
            Ok(PipeEvent::Failed) => {
                failure.get_or_insert(ExecutionError::AgentFailed);
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                closed = 2;
                if !exited {
                    thread::sleep(Duration::from_millis(5));
                }
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {}
        }
    }
    if !stopped && let Err(error) = stop_group(child) {
        failure.get_or_insert(error);
    }
    if child.wait().is_err() {
        failure.get_or_insert(ExecutionError::AgentFailed);
    }
    let output = String::from_utf8(output).unwrap_or_else(|_| {
        failure.get_or_insert(ExecutionError::ProtocolInvalid);
        String::new()
    });
    let error = failure.or_else(|| {
        (!status.is_some_and(|status| status.success())).then_some(ExecutionError::AgentFailed)
    });
    CaptureResult {
        output,
        error,
        failure_detail: error.map(|error| failure_detail(error, status)),
    }
}

fn failure_detail(error: ExecutionError, status: Option<std::process::ExitStatus>) -> String {
    match error {
        ExecutionError::Timeout => "process timed out".into(),
        ExecutionError::OutputLimit => "process output exceeded the limit".into(),
        ExecutionError::ProtocolInvalid => "invalid process protocol".into(),
        ExecutionError::AgentFailed => match status.and_then(|status| status.code()) {
            Some(code) if code != 0 => format!("process exited with code {code}"),
            _ => "process terminated or I/O failed".into(),
        },
    }
}

#[cfg(test)]
mod tests;
