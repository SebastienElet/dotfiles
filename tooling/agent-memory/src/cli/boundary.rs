use super::CliFailure;
use serde::Serialize;
use std::io::{Read, Write};

const MAX_STDIN_BYTES: u64 = 1024 * 1024;

pub(super) fn read_required(input: &mut dyn Read) -> Result<Vec<u8>, CliFailure> {
    let mut bytes = Vec::new();
    input
        .take(MAX_STDIN_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| failure(4, "stdin_unavailable", "stdin"))?;
    if bytes.len() as u64 > MAX_STDIN_BYTES {
        return Err(failure(2, "input_too_large", "stdin"));
    }
    if bytes.is_empty() {
        return Err(failure(2, "empty_stdin", "stdin"));
    }
    Ok(bytes)
}

pub(super) fn write_json(output: &mut dyn Write, value: &impl Serialize) -> Result<(), CliFailure> {
    serde_json::to_writer(&mut *output, value)
        .map_err(|_| failure(4, "output_unavailable", "stdout"))?;
    output
        .write_all(b"\n")
        .map_err(|_| failure(4, "output_unavailable", "stdout"))
}

fn failure(exit: u8, code: &'static str, field: &'static str) -> CliFailure {
    CliFailure { exit, code, field }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;

    struct FailingIo;

    impl Read for FailingIo {
        fn read(&mut self, _buffer: &mut [u8]) -> io::Result<usize> {
            Err(io::Error::other("read unavailable"))
        }
    }

    impl Write for FailingIo {
        fn write(&mut self, _buffer: &[u8]) -> io::Result<usize> {
            Err(io::Error::other("write unavailable"))
        }

        fn flush(&mut self) -> io::Result<()> {
            Err(io::Error::other("flush unavailable"))
        }
    }

    #[test]
    fn distinguishes_stdin_and_stdout_failures() {
        let read = read_required(&mut FailingIo).unwrap_err();
        assert_eq!(
            (read.exit, read.code, read.field),
            (4, "stdin_unavailable", "stdin")
        );

        let write = write_json(&mut FailingIo, &serde_json::json!({})).unwrap_err();
        assert_eq!(
            (write.exit, write.code, write.field),
            (4, "output_unavailable", "stdout")
        );
    }
}
