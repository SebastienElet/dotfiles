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
}

enum PipeEvent {
    Output(Vec<u8>),
    OutputClosed,
    InputClosed,
    Failed,
}

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
    let Ok(mut child) = child else {
        return CaptureResult {
            output: String::new(),
            error: Some(ExecutionError::AgentFailed),
        };
    };
    let stdin = child.stdin.take().expect("piped stdin");
    let stdout = child.stdout.take().expect("piped stdout");
    let (sender, receiver) = mpsc::sync_channel(4);
    thread::scope(|scope| {
        let input_sender = sender.clone();
        scope.spawn(move || write_input(stdin, options.stdin, input_sender));
        scope.spawn(move || read_output(stdout, sender));
        monitor(&mut child, receiver, options.timeout)
    })
}

fn write_input(mut input: impl Write, text: &str, sender: SyncSender<PipeEvent>) {
    if input.write_all(text.as_bytes()).is_err() {
        let _ = sender.send(PipeEvent::Failed);
    }
    drop(input);
    let _ = sender.send(PipeEvent::InputClosed);
}

fn read_output(mut output: impl Read, sender: SyncSender<PipeEvent>) {
    let mut bytes = [0_u8; 8192];
    loop {
        match output.read(&mut bytes) {
            Ok(0) => break,
            Ok(count) => {
                if sender
                    .send(PipeEvent::Output(bytes[..count].to_vec()))
                    .is_err()
                {
                    return;
                }
            }
            Err(error) if error.kind() == std::io::ErrorKind::Interrupted => continue,
            Err(_) => {
                let _ = sender.send(PipeEvent::Failed);
                break;
            }
        }
    }
    let _ = sender.send(PipeEvent::OutputClosed);
}

fn stop_group(child: &mut Child) -> Result<(), ExecutionError> {
    let pid = Pid::from_raw(child.id() as i32).ok_or(ExecutionError::AgentFailed)?;
    match kill_process_group(pid, Signal::KILL) {
        Ok(()) | Err(rustix::io::Errno::SRCH) => Ok(()),
        Err(_) => {
            let _ = child.kill();
            Err(ExecutionError::AgentFailed)
        }
    }
}

fn monitor(child: &mut Child, receiver: Receiver<PipeEvent>, timeout: Duration) -> CaptureResult {
    let started = Instant::now();
    let mut output = Vec::new();
    let mut failure = None;
    let mut closed = 0;
    let mut exited = false;
    let mut successful = false;
    let mut stopped = false;
    while !exited || closed < 2 {
        if !exited {
            match child.try_wait() {
                Ok(Some(status)) => {
                    exited = true;
                    successful = status.success();
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
    let _ = child.wait();
    CaptureResult {
        output: String::from_utf8_lossy(&output).into_owned(),
        error: failure.or_else(|| (!successful).then_some(ExecutionError::AgentFailed)),
    }
}

#[cfg(test)]
mod tests;
