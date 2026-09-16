use crate::Result;
#[cfg(unix)]
use std::os::unix::process::CommandExt;
use std::{
    io::{self, Read},
    process::{Child, Command, Output, Stdio},
    sync::mpsc::{self, Receiver},
    thread,
    time::{Duration, Instant},
};

pub fn capture(command: &mut Command, timeout: Duration) -> Result<Output> {
    command
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    #[cfg(unix)]
    command.process_group(0);
    let mut child = command.spawn()?;
    let result = collect(&mut child, timeout);
    if result.is_err() {
        #[cfg(unix)]
        let _ = rustix::process::kill_process_group(
            rustix::process::Pid::from_child(&child),
            rustix::process::Signal::KILL,
        );
        let _ = child.kill();
        let _ = child.wait();
    }
    result
}
fn collect(child: &mut Child, timeout: Duration) -> Result<Output> {
    let start = Instant::now();
    let stdout = pipe(child.stdout.take().ok_or("missing process stdout")?);
    let stderr = pipe(child.stderr.take().ok_or("missing process stderr")?);
    let status = loop {
        if start.elapsed() >= timeout {
            return Err("Git process exceeded deadline".into());
        }
        if let Some(status) = child.try_wait()? {
            break status;
        }
        thread::sleep(Duration::from_millis(10));
    };
    let stdout = stdout.recv_timeout(timeout.saturating_sub(start.elapsed()))??;
    let stderr = stderr.recv_timeout(timeout.saturating_sub(start.elapsed()))??;
    Ok(Output {
        status,
        stdout,
        stderr,
    })
}
fn pipe(reader: impl Read + Send + 'static) -> Receiver<io::Result<Vec<u8>>> {
    let (sender, receiver) = mpsc::channel();
    thread::spawn(move || {
        let mut output = Vec::new();
        let result = reader
            .take(64 * 1024 * 1024 + 1)
            .read_to_end(&mut output)
            .and_then(|length| {
                if length > 64 * 1024 * 1024 {
                    Err(io::Error::other("Git output exceeds 64MiB"))
                } else {
                    Ok(output)
                }
            });
        let _ = sender.send(result);
    });
    receiver
}

#[cfg(test)]
mod tests {
    use super::capture;
    use std::{
        process::Command,
        time::{Duration, Instant},
    };

    #[test]
    fn refuses_a_process_that_exceeds_its_deadline() {
        let start = Instant::now();
        let result = capture(
            Command::new("/bin/sleep").arg("1"),
            Duration::from_millis(20),
        );
        assert!(result.is_err());
        assert!(start.elapsed() < Duration::from_millis(500));
    }
}
