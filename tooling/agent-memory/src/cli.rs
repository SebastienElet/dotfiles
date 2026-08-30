mod arguments;
mod boundary;
mod commands;
mod retrieval;

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
    complete(
        commands::dispatch(arguments.command, &mut input),
        &mut output,
        &mut diagnostics,
    )
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

    impl Write for FailingWriter {
        fn write(&mut self, _buffer: &[u8]) -> io::Result<usize> {
            Err(io::Error::other("write unavailable"))
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
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
}
