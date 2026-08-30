use std::io::{self, Read};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::thread::{self, JoinHandle};

const MAX_PROCESS_OUTPUT_BYTES: usize = 1024 * 1024;

#[derive(Default)]
pub(super) struct ReaderState {
    captured: AtomicUsize,
    pub(super) completed: AtomicUsize,
    pub(super) failed: AtomicBool,
    pub(super) overflow: AtomicBool,
}

pub(super) fn spawn_reader<R>(reader: R, state: Arc<ReaderState>) -> JoinHandle<io::Result<Vec<u8>>>
where
    R: Read + Send + 'static,
{
    thread::spawn(move || {
        let result = read_bounded(reader, &state);
        if result.is_err() {
            state.failed.store(true, Ordering::Release);
        }
        state.completed.fetch_add(1, Ordering::Release);
        result
    })
}

fn read_bounded(mut reader: impl Read, state: &ReaderState) -> io::Result<Vec<u8>> {
    let mut bytes = Vec::new();
    let mut chunk = [0; 8192];
    loop {
        let count = reader.read(&mut chunk)?;
        if count == 0 {
            break;
        }
        if !state.overflow.load(Ordering::Acquire) && reserve_output(&state.captured, count) {
            bytes.extend_from_slice(&chunk[..count]);
        } else {
            state.overflow.store(true, Ordering::Release);
        }
    }
    if state.overflow.load(Ordering::Acquire) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "process_output_limit",
        ));
    }
    Ok(bytes)
}

fn reserve_output(captured: &AtomicUsize, count: usize) -> bool {
    let mut current = captured.load(Ordering::Acquire);
    loop {
        let Some(next) = current.checked_add(count) else {
            return false;
        };
        if next > MAX_PROCESS_OUTPUT_BYTES {
            return false;
        }
        match captured.compare_exchange_weak(current, next, Ordering::AcqRel, Ordering::Acquire) {
            Ok(_) => return true,
            Err(observed) => current = observed,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct FailingReader;

    impl Read for FailingReader {
        fn read(&mut self, _buffer: &mut [u8]) -> io::Result<usize> {
            Err(io::Error::other("read unavailable"))
        }
    }

    #[test]
    fn preserves_pipe_read_failures() {
        let error = read_bounded(FailingReader, &ReaderState::default()).unwrap_err();

        assert_eq!(error.kind(), io::ErrorKind::Other);
    }
}
