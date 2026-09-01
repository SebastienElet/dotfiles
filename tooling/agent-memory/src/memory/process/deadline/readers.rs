use rustix::fs::{OFlags, fcntl_getfl, fcntl_setfl};
use std::io::{self, Read};
use std::os::fd::AsFd;
use std::process::{ChildStderr, ChildStdout};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::thread::{self, JoinHandle};

mod capture;

#[derive(Default)]
pub(super) struct ReaderState {
    captured: AtomicUsize,
    failed: AtomicBool,
    overflow: AtomicBool,
}

pub(super) enum ReaderPipe {
    Stdout(ChildStdout),
    Stderr(ChildStderr),
}

impl Read for ReaderPipe {
    fn read(&mut self, bytes: &mut [u8]) -> io::Result<usize> {
        match self {
            Self::Stdout(reader) => reader.read(bytes),
            Self::Stderr(reader) => reader.read(bytes),
        }
    }
}

pub(super) trait ReaderSpawner {
    fn spawn(
        &self,
        reader: ReaderPipe,
        state: Arc<ReaderState>,
        cancelled: Arc<AtomicBool>,
    ) -> io::Result<JoinHandle<io::Result<Vec<u8>>>>;
}

pub(super) struct SystemReaderSpawner;

impl ReaderSpawner for SystemReaderSpawner {
    fn spawn(
        &self,
        reader: ReaderPipe,
        state: Arc<ReaderState>,
        cancelled: Arc<AtomicBool>,
    ) -> io::Result<JoinHandle<io::Result<Vec<u8>>>> {
        thread::Builder::new()
            .name("agent-memory-reader".to_owned())
            .spawn(move || {
                let result = capture::read_bounded(reader, &state, &cancelled);
                if result.is_err() {
                    state.failed.store(true, Ordering::Release);
                }
                result
            })
    }
}

pub(super) struct Readers {
    state: Arc<ReaderState>,
    cancelled: Arc<AtomicBool>,
    stdout: Option<JoinHandle<io::Result<Vec<u8>>>>,
    stderr: Option<JoinHandle<io::Result<Vec<u8>>>>,
}

impl Readers {
    pub(super) fn new() -> Self {
        Self {
            state: Arc::new(ReaderState::default()),
            cancelled: Arc::new(AtomicBool::new(false)),
            stdout: None,
            stderr: None,
        }
    }

    pub(super) fn start_stdout(
        &mut self,
        reader: ChildStdout,
        spawner: &dyn ReaderSpawner,
    ) -> io::Result<()> {
        self.stdout = Some(spawner.spawn(
            ReaderPipe::Stdout(reader),
            Arc::clone(&self.state),
            Arc::clone(&self.cancelled),
        )?);
        Ok(())
    }

    pub(super) fn start_stderr(
        &mut self,
        reader: ChildStderr,
        spawner: &dyn ReaderSpawner,
    ) -> io::Result<()> {
        self.stderr = Some(spawner.spawn(
            ReaderPipe::Stderr(reader),
            Arc::clone(&self.state),
            Arc::clone(&self.cancelled),
        )?);
        Ok(())
    }

    pub(super) fn failure(&self) -> Option<io::Error> {
        if self.state.overflow.load(Ordering::Acquire) {
            return Some(io::Error::new(
                io::ErrorKind::InvalidData,
                "process_output_limit",
            ));
        }
        self.state
            .failed
            .load(Ordering::Acquire)
            .then(|| io::Error::other("process_reader_unavailable"))
    }

    pub(super) fn finished(&self) -> bool {
        self.stdout.as_ref().is_none_or(JoinHandle::is_finished)
            && self.stderr.as_ref().is_none_or(JoinHandle::is_finished)
    }

    pub(super) fn cancel(&self) {
        self.cancelled.store(true, Ordering::Release);
    }

    pub(super) fn join_outputs(&mut self) -> io::Result<(Vec<u8>, Vec<u8>)> {
        let stdout = self.stdout.take().ok_or_else(reader_unavailable)?;
        let stderr = self.stderr.take().ok_or_else(reader_unavailable)?;
        Ok((join_reader(stdout)?, join_reader(stderr)?))
    }

    pub(super) fn join_finished(&mut self) -> io::Result<()> {
        if let Some(reader) = self.stdout.take_if(|reader| reader.is_finished()) {
            join_discard(reader)?;
        }
        if let Some(reader) = self.stderr.take_if(|reader| reader.is_finished()) {
            join_discard(reader)?;
        }
        Ok(())
    }
}

pub(super) fn set_nonblocking(reader: &impl AsFd) -> io::Result<()> {
    let flags = fcntl_getfl(reader).map_err(from_rustix)?;
    fcntl_setfl(reader, flags | OFlags::NONBLOCK).map_err(from_rustix)
}

fn join_reader(reader: JoinHandle<io::Result<Vec<u8>>>) -> io::Result<Vec<u8>> {
    reader
        .join()
        .map_err(|_| io::Error::other("process_reader_panicked"))?
}

fn join_discard(reader: JoinHandle<io::Result<Vec<u8>>>) -> io::Result<()> {
    let _ = reader
        .join()
        .map_err(|_| io::Error::other("process_reader_panicked"))?;
    Ok(())
}

fn reader_unavailable() -> io::Error {
    io::Error::other("process_reader_unavailable")
}

fn from_rustix(error: rustix::io::Errno) -> io::Error {
    io::Error::from_raw_os_error(error.raw_os_error())
}
