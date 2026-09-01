mod arguments;
mod boundary;
mod commands;
mod retrieval;
mod trace;

use arguments::Arguments;
use boundary::write_json;
use clap::{Parser, error::ErrorKind};
use serde_json::json;
use std::io;

pub fn run_cli() -> u8 {
    let arguments = match Arguments::try_parse() {
        Ok(arguments) => arguments,
        Err(error) => return report_argument_error(error),
    };
    let stdin = io::stdin();
    let mut input = stdin.lock();
    let stdout = io::stdout();
    let mut output = stdout.lock();
    let stderr = io::stderr();
    let mut diagnostics = stderr.lock();
    let mut trace = match trace::EvaluationTrace::from_environment(arguments.command.trace_name()) {
        Ok(trace) => trace,
        Err(failure) => return write_failure(&mut diagnostics, failure),
    };
    let result = commands::dispatch(arguments.command, &mut input);
    let exit = complete(result, &mut output, &mut diagnostics);
    if let Err(failure) = trace.finish(exit) {
        return write_failure(&mut diagnostics, failure);
    }
    exit
}

fn report_argument_error(error: clap::Error) -> u8 {
    if matches!(
        error.kind(),
        ErrorKind::DisplayHelp | ErrorKind::DisplayVersion
    ) {
        let exit = error.exit_code();
        let _ = error.print();
        return u8::try_from(exit).unwrap_or(0);
    }
    let stderr = io::stderr();
    write_failure(&mut stderr.lock(), CliFailure::invalid_arguments())
}

fn complete(
    result: Result<serde_json::Value, CliFailure>,
    output: &mut dyn io::Write,
    diagnostics: &mut dyn io::Write,
) -> u8 {
    match result {
        Ok(value) => match write_json(output, &value) {
            Ok(()) => 0,
            Err(failure) => write_failure(diagnostics, failure),
        },
        Err(failure) => write_failure(diagnostics, failure),
    }
}

fn write_failure(diagnostics: &mut dyn io::Write, failure: CliFailure) -> u8 {
    let value = json!({"error": {"code": failure.code, "field": failure.field}});
    write_json(diagnostics, &value).map_or(4, |()| failure.exit)
}

#[derive(Clone, Copy)]
struct CliFailure {
    exit: u8,
    code: &'static str,
    field: &'static str,
}

impl CliFailure {
    fn invalid_arguments() -> Self {
        Self {
            exit: 2,
            code: "invalid_arguments",
            field: "arguments",
        }
    }

    fn evaluation_trace_unavailable() -> Self {
        Self {
            exit: 4,
            code: "evaluation_trace_unavailable",
            field: "trace",
        }
    }

    fn from_memory(error: crate::MemoryError) -> Self {
        let exit = match error.class() {
            crate::MemoryErrorClass::Rejection => 2,
            crate::MemoryErrorClass::Conflict => 3,
            crate::MemoryErrorClass::Unavailable => 4,
        };
        Self {
            exit,
            code: error.code(),
            field: error.field(),
        }
    }

    fn from_hook(error: crate::HookError) -> Self {
        let exit = match error.class() {
            crate::HookErrorClass::Rejection => 2,
            crate::HookErrorClass::Unavailable => 4,
        };
        Self {
            exit,
            code: error.code(),
            field: error.field(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{self, Write};

    struct FailingWriter;

    #[derive(Default)]
    struct FlushFailingWriter {
        bytes: Vec<u8>,
    }

    impl Write for FailingWriter {
        fn write(&mut self, _buffer: &[u8]) -> io::Result<usize> {
            Err(io::Error::other("write unavailable"))
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    impl Write for FlushFailingWriter {
        fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
            self.bytes.extend_from_slice(buffer);
            Ok(buffer.len())
        }

        fn flush(&mut self) -> io::Result<()> {
            Err(io::Error::other("flush unavailable"))
        }
    }

    #[test]
    fn reports_stdout_failure_on_stderr() {
        let mut diagnostics = Vec::new();

        let exit = complete(
            Ok(json!({"status": "stored"})),
            &mut FailingWriter,
            &mut diagnostics,
        );

        assert_eq!(exit, 4);
        let value: serde_json::Value = serde_json::from_slice(&diagnostics).unwrap();
        assert_eq!(value["error"]["code"], "output_unavailable");
        assert_eq!(value["error"]["field"], "stdout");
    }

    #[test]
    fn reports_flush_only_failure_without_copying_context_to_diagnostics() {
        let context = "AGENT_MEMORY_CONTEXT_V1 secret statement";
        let mut output = FlushFailingWriter::default();
        let mut diagnostics = Vec::new();

        let exit = complete(
            Ok(json!({"additionalContext": context})),
            &mut output,
            &mut diagnostics,
        );

        assert_eq!(exit, 4);
        assert!(!String::from_utf8_lossy(&diagnostics).contains(context));
        let value: serde_json::Value = serde_json::from_slice(&diagnostics).unwrap();
        assert_eq!(value["error"]["code"], "output_unavailable");
    }
}
