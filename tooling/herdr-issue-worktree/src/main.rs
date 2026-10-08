mod storage;
#[cfg(test)]
mod storage_tests;
mod unique_maps;

use std::error::Error;
use std::io::{self, Read, Write};
use std::path::PathBuf;
use std::process::ExitCode;

fn run() -> Result<(), Box<dyn Error>> {
    let directory = PathBuf::from(
        std::env::args_os()
            .nth(1)
            .ok_or("State directory required")?,
    );
    if !directory.is_absolute() {
        return Err("State directory must be absolute".into());
    }
    let mut input = String::new();
    io::stdin().take(1_048_577).read_to_string(&mut input)?;
    if input.len() > 1_048_576 {
        return Err("Request exceeds one MiB".into());
    }
    let request = serde_json::from_str(&input)?;
    let result = storage::dispatch(&directory, request)?;
    serde_json::to_writer(io::stdout().lock(), &result)?;
    Ok(())
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            let _ = writeln!(io::stderr().lock(), "{error}");
            if error.downcast_ref::<storage::Conflict>().is_some() {
                ExitCode::from(2)
            } else {
                ExitCode::FAILURE
            }
        }
    }
}
