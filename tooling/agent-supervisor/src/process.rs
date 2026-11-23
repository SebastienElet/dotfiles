use std::{
    io::Read,
    os::unix::process::CommandExt,
    path::Path,
    process::{Command, Stdio},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc,
    },
    thread,
    time::{Duration, Instant},
};

use crate::{Result, ensure};

pub struct Cancellation {
    pub signal: Arc<AtomicBool>,
}

impl Cancellation {
    pub fn new() -> Result<Self> {
        let signal = Arc::new(AtomicBool::new(false));
        for number in [
            signal_hook::consts::SIGINT,
            signal_hook::consts::SIGTERM,
            signal_hook::consts::SIGHUP,
        ] {
            signal_hook::flag::register(number, Arc::clone(&signal))?;
        }
        Ok(Self { signal })
    }

    pub fn stopped(&self, directory: &Path) -> bool {
        self.signal.load(Ordering::Relaxed) || directory.join("stop").exists()
    }
}

fn kill_group(id: u32) -> Result<()> {
    let group = nix::unistd::Pid::from_raw(i32::try_from(id)?);
    match nix::sys::signal::killpg(group, nix::sys::signal::Signal::SIGKILL) {
        Ok(()) | Err(nix::errno::Errno::ESRCH) => Ok(()),
        Err(_) => Err("owned process group cleanup failed".into()),
    }
}

pub fn run(
    command: &mut Command,
    directory: &Path,
    cancellation: &Cancellation,
    timeout: Duration,
    expires_at: u64,
    mut consume: impl FnMut(&[u8]) -> Result<()>,
    mut submitted: impl FnMut(u32) -> Result<()>,
) -> Result<()> {
    ensure(!cancellation.stopped(directory), "stopped")?;
    ensure(
        !timeout.is_zero() && crate::store::now()? < expires_at,
        "mandate expired before submission",
    )?;
    let mut child = command
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .process_group(0)
        .spawn()?;
    let (sender, receiver) = mpsc::sync_channel(4);
    let mut stdout = child.stdout.take().ok_or("stdout unavailable")?;
    let reader = thread::spawn(move || -> std::io::Result<()> {
        let mut buffer = [0; 8192];
        loop {
            let length = stdout.read(&mut buffer)?;
            if length == 0 || sender.send(buffer[..length].to_vec()).is_err() {
                return Ok(());
            }
        }
    });
    let mut result = supervise(
        &mut child,
        directory,
        cancellation,
        timeout,
        &receiver,
        &mut consume,
        &mut submitted,
    );
    let cleanup = kill_group(child.id());
    let wait_deadline = Instant::now() + Duration::from_secs(2);
    while child.try_wait()?.is_none() {
        ensure(
            Instant::now() < wait_deadline,
            "owned subprocess did not exit after cleanup",
        )?;
        thread::sleep(Duration::from_millis(10));
    }
    if let Err(error) = cleanup {
        return Err(format!(
            "{}; {error}",
            result
                .err()
                .map_or_else(|| "subprocess exited".into(), |error| error.to_string())
        )
        .into());
    }
    let drain_deadline = Instant::now() + Duration::from_secs(1);
    while !reader.is_finished() && Instant::now() < drain_deadline {
        while let Ok(bytes) = receiver.try_recv() {
            if let Err(error) = consume(&bytes)
                && result.is_ok()
            {
                result = Err(error);
            }
        }
        thread::sleep(Duration::from_millis(10));
    }
    if !reader.is_finished() {
        kill_group(child.id())?;
        return Err("subprocess output incomplete; inspect owned descendants".into());
    }
    reader.join().map_err(|_| "output reader failed")??;
    for bytes in receiver.try_iter() {
        if let Err(error) = consume(&bytes)
            && result.is_ok()
        {
            result = Err(error);
        }
    }
    result
}

fn supervise(
    child: &mut std::process::Child,
    directory: &Path,
    cancellation: &Cancellation,
    timeout: Duration,
    receiver: &mpsc::Receiver<Vec<u8>>,
    consume: &mut impl FnMut(&[u8]) -> Result<()>,
    submitted: &mut impl FnMut(u32) -> Result<()>,
) -> Result<()> {
    submitted(child.id())?;
    let deadline = Instant::now() + timeout;
    loop {
        ensure(!cancellation.stopped(directory), "stopped")?;
        ensure(Instant::now() < deadline, "subprocess deadline exceeded")?;
        for bytes in receiver.try_iter() {
            consume(&bytes)?;
        }
        if let Some(status) = child.try_wait()? {
            ensure(status.success(), "subprocess failed")?;
            return Ok(());
        }
        thread::sleep(Duration::from_millis(20));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero_budget_never_submits_a_process() -> Result<()> {
        let directory = tempfile::tempdir()?;
        let cancellation = Cancellation::new()?;
        let mut submitted = false;
        let result = run(
            &mut Command::new("true"),
            directory.path(),
            &cancellation,
            Duration::ZERO,
            crate::store::now()? + 10,
            |_| Ok(()),
            |_| {
                submitted = true;
                Ok(())
            },
        );
        assert!(result.is_err());
        assert!(!submitted);
        Ok(())
    }
}
