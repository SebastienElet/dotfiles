use super::{ProcessOutput, ProcessRunner};
use readers::{ReaderState, spawn_reader};
use std::ffi::{OsStr, OsString};
use std::io;
use std::os::unix::process::CommandExt;
use std::path::Path;
use std::process::{Child, ChildStderr, ChildStdout, Command, Stdio};
use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

const POLL_INTERVAL: Duration = Duration::from_millis(5);

mod readers;

pub struct DeadlineProcessRunner {
    deadline: Instant,
}

impl DeadlineProcessRunner {
    pub fn new(deadline: Instant) -> Self {
        Self { deadline }
    }
}

impl ProcessRunner for DeadlineProcessRunner {
    fn run(
        &self,
        program: &OsStr,
        arguments: &[OsString],
        current_directory: Option<&Path>,
    ) -> io::Result<ProcessOutput> {
        let mut command = Command::new(program);
        command
            .args(arguments)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .process_group(0);
        if let Some(directory) = current_directory {
            command.current_dir(directory);
        }
        let mut child = command.spawn()?;
        let (stdout, stderr) = take_pipes(&mut child)?;
        let readers = Arc::new(ReaderState::default());
        let stdout_reader = spawn_reader(stdout, Arc::clone(&readers));
        let stderr_reader = spawn_reader(stderr, Arc::clone(&readers));
        wait_for_process(
            &mut child,
            self.deadline,
            readers,
            stdout_reader,
            stderr_reader,
        )
    }

    fn remaining_time(&self) -> Option<Duration> {
        Some(self.deadline.saturating_duration_since(Instant::now()))
    }
}

fn wait_for_process(
    child: &mut Child,
    deadline: Instant,
    readers: Arc<ReaderState>,
    stdout_reader: JoinHandle<io::Result<Vec<u8>>>,
    stderr_reader: JoinHandle<io::Result<Vec<u8>>>,
) -> io::Result<ProcessOutput> {
    let mut status = None;
    loop {
        if status.is_none() {
            match child.try_wait() {
                Ok(Some(completed)) => status = Some(completed),
                Ok(None) => {}
                Err(error) => {
                    return fail_after_cleanup(child, stdout_reader, stderr_reader, error);
                }
            }
        }
        if let Some(error) = reader_failure(&readers) {
            return fail_after_cleanup(child, stdout_reader, stderr_reader, error);
        }
        if let Some(status) = status
            && readers.completed.load(Ordering::Acquire) == 2
        {
            let stdout = join_reader(stdout_reader);
            let stderr = join_reader(stderr_reader);
            return completed_output(status, stdout, stderr);
        }
        if Instant::now() >= deadline {
            return fail_after_cleanup(
                child,
                stdout_reader,
                stderr_reader,
                io::Error::new(io::ErrorKind::TimedOut, "process_deadline"),
            );
        }
        thread::sleep(POLL_INTERVAL.min(deadline.saturating_duration_since(Instant::now())));
    }
}

fn reader_failure(readers: &ReaderState) -> Option<io::Error> {
    if readers.overflow.load(Ordering::Acquire) {
        return Some(io::Error::new(
            io::ErrorKind::InvalidData,
            "process_output_limit",
        ));
    }
    readers
        .failed
        .load(Ordering::Acquire)
        .then(|| io::Error::other("process_reader_unavailable"))
}

fn completed_output(
    status: std::process::ExitStatus,
    stdout: io::Result<Vec<u8>>,
    stderr: io::Result<Vec<u8>>,
) -> io::Result<ProcessOutput> {
    Ok(ProcessOutput::new(
        status.success(),
        status.code(),
        stdout?,
        stderr?,
    ))
}

fn terminate(child: &mut Child) -> io::Result<()> {
    let pid = rustix::process::Pid::from_raw(child.id() as i32).ok_or_else(pipe_unavailable)?;
    match rustix::process::kill_process_group(pid, rustix::process::Signal::KILL) {
        Ok(()) | Err(rustix::io::Errno::SRCH) => Ok(()),
        Err(error) => Err(io::Error::from_raw_os_error(error.raw_os_error())),
    }
}

fn take_pipes(child: &mut Child) -> io::Result<(ChildStdout, ChildStderr)> {
    match (child.stdout.take(), child.stderr.take()) {
        (Some(stdout), Some(stderr)) => Ok((stdout, stderr)),
        _ => terminate_and_wait(child).and(Err(pipe_unavailable())),
    }
}

fn terminate_and_wait(child: &mut Child) -> io::Result<()> {
    let terminate_result = terminate(child);
    let wait_result = child.wait();
    terminate_result?;
    wait_result?;
    Ok(())
}

fn fail_after_cleanup(
    child: &mut Child,
    stdout_reader: JoinHandle<io::Result<Vec<u8>>>,
    stderr_reader: JoinHandle<io::Result<Vec<u8>>>,
    error: io::Error,
) -> io::Result<ProcessOutput> {
    cleanup(child, stdout_reader, stderr_reader).and(Err(error))
}

fn cleanup(
    child: &mut Child,
    stdout_reader: JoinHandle<io::Result<Vec<u8>>>,
    stderr_reader: JoinHandle<io::Result<Vec<u8>>>,
) -> io::Result<()> {
    let process_result = terminate_and_wait(child);
    let stdout_result = join_reader(stdout_reader);
    let stderr_result = join_reader(stderr_reader);
    process_result?;
    stdout_result?;
    stderr_result?;
    Ok(())
}

fn join_reader(reader: JoinHandle<io::Result<Vec<u8>>>) -> io::Result<Vec<u8>> {
    reader
        .join()
        .map_err(|_| io::Error::other("process_reader_panicked"))?
}

fn pipe_unavailable() -> io::Error {
    io::Error::other("process_pipe_unavailable")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reaps_a_child_when_a_configured_pipe_is_unavailable_after_spawn() {
        let mut command = Command::new("sleep");
        command
            .arg("2")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .process_group(0);
        let mut child = command.spawn().unwrap();
        let _ = child.stdout.take();

        let error = take_pipes(&mut child).unwrap_err();

        assert_eq!(error.kind(), io::ErrorKind::Other);
        assert!(child.try_wait().unwrap().is_some());
    }
}
