use super::ReaderState;
use std::io::{self, Read};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::thread;
use std::time::Duration;

const MAX_PROCESS_OUTPUT_BYTES: usize = 1024 * 1024;
const READER_POLL_INTERVAL: Duration = Duration::from_millis(5);

pub(super) fn read_bounded(
    mut reader: impl Read,
    state: &ReaderState,
    cancelled: &AtomicBool,
) -> io::Result<Vec<u8>> {
    let mut bytes = Vec::new();
    let mut chunk = [0; 8192];
    loop {
        if cancelled.load(Ordering::Acquire) {
            return Err(reader_cancelled());
        }
        match reader.read(&mut chunk) {
            Ok(0) => break,
            Ok(count) => append_chunk(&mut bytes, &chunk[..count], state),
            Err(error) if error.kind() == io::ErrorKind::WouldBlock => {
                thread::sleep(READER_POLL_INTERVAL);
            }
            Err(error) => return Err(error),
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

fn reader_cancelled() -> io::Error {
    io::Error::new(io::ErrorKind::TimedOut, "process_reader_cancelled")
}

fn append_chunk(bytes: &mut Vec<u8>, chunk: &[u8], state: &ReaderState) {
    if !state.overflow.load(Ordering::Acquire) && reserve_output(&state.captured, chunk.len()) {
        bytes.extend_from_slice(chunk);
    } else {
        state.overflow.store(true, Ordering::Release);
    }
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

    struct AvailableReader(bool);

    impl Read for FailingReader {
        fn read(&mut self, _buffer: &mut [u8]) -> io::Result<usize> {
            Err(io::Error::other("read unavailable"))
        }
    }

    impl Read for AvailableReader {
        fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
            if self.0 {
                return Ok(0);
            }
            self.0 = true;
            buffer[0] = b'x';
            Ok(1)
        }
    }

    #[test]
    fn preserves_pipe_read_failures() {
        let error = read_bounded(
            FailingReader,
            &ReaderState::default(),
            &AtomicBool::new(false),
        )
        .unwrap_err();

        assert_eq!(error.kind(), io::ErrorKind::Other);
    }

    #[test]
    fn stops_before_reading_available_bytes_after_cancellation() {
        let error = read_bounded(
            AvailableReader(false),
            &ReaderState::default(),
            &AtomicBool::new(true),
        )
        .unwrap_err();

        assert_eq!(error.kind(), io::ErrorKind::TimedOut);
    }
}
