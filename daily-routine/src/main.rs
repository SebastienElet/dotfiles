mod bitbucket;
mod command;
mod config;
mod github;
mod linear;
mod model;
mod parallel;
mod report;
mod rules;
mod self_check;
mod things;
mod util;

use config::Config;
use model::{Category, Dataset, Report, Warning};
use std::error::Error;
use std::io::Write;
use std::process::ExitCode;

const CONFIG_EXAMPLE: &str = include_str!("../config.example.toml");
const USAGE: &str = "Usage: daily-routine [--self-check] [--no-things]";

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct Options {
    self_check: bool,
    no_things: bool,
}

fn main() -> ExitCode {
    let options = match parse_args(std::env::args().skip(1)) {
        Ok(options) => options,
        Err(error) => {
            eprintln!("{error}\n{USAGE}");
            return ExitCode::from(2);
        }
    };

    if options.self_check {
        return match self_check::run() {
            Ok(()) => ExitCode::SUCCESS,
            Err(error) => {
                eprintln!("failed to write self-check result: {error}");
                ExitCode::FAILURE
            }
        };
    }

    run_normal(options.no_things)
}

fn parse_args(args: impl IntoIterator<Item = String>) -> Result<Options, String> {
    let mut options = Options::default();

    for argument in args {
        let selected = match argument.as_str() {
            "--self-check" => &mut options.self_check,
            "--no-things" => &mut options.no_things,
            _ => return Err(format!("unknown argument: {argument}")),
        };
        if *selected {
            return Err(format!("duplicate argument: {argument}"));
        }
        *selected = true;
    }

    Ok(options)
}

fn run_normal(no_things: bool) -> ExitCode {
    let config = match Config::load() {
        Ok(config) => config,
        Err(error) => {
            eprint!("{}", config_error_message(error.as_ref()));
            return ExitCode::FAILURE;
        }
    };

    let linear = linear::collect(&config);
    let bitbucket = bitbucket::collect(&config);
    let github = github::collect(&config);
    let _resolved_identities = (&linear.identity, &bitbucket.identity, &github.identity);

    let mut dataset = Dataset {
        pull_requests: bitbucket.pull_requests,
        issues: linear.issues,
        warnings: linear.warnings,
    };
    dataset.pull_requests.extend(github.pull_requests);
    dataset.warnings.extend(bitbucket.warnings);
    dataset.warnings.extend(github.warnings);

    let today_days = match util::today_days() {
        Ok(today_days) => today_days,
        Err(error) => {
            dataset.warnings.push(Warning {
                categories: vec![Category::Linear],
                message: format!("failed to determine the current day: {error}"),
            });
            0
        }
    };
    let report = rules::build_report(&config, &dataset, today_days);
    let stdout = std::io::stdout();
    let mut stdout = stdout.lock();
    if let Err(error) = write_report(&mut stdout, &config, &report) {
        eprintln!("failed to write report: {error}");
        return ExitCode::FAILURE;
    }
    for warning in &report.warnings {
        print_warning(warning);
    }

    match push_if_enabled(no_things, || things::push(&report)) {
        Ok(Some(outcome)) => {
            for warning in outcome.warnings {
                eprintln!("warning [THINGS]: {warning}");
            }
            ExitCode::SUCCESS
        }
        Ok(None) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}

fn write_report(writer: &mut impl Write, config: &Config, report: &Report) -> std::io::Result<()> {
    writer.write_all(report::render(config, report).as_bytes())?;
    writer.flush()
}

fn push_if_enabled<F>(no_things: bool, push: F) -> Result<Option<things::PushOutcome>, String>
where
    F: FnOnce() -> Result<things::PushOutcome, String>,
{
    if no_things {
        Ok(None)
    } else {
        push().map(Some)
    }
}

fn print_warning(warning: &Warning) {
    let categories = warning
        .categories
        .iter()
        .map(|category| category.label())
        .collect::<Vec<_>>()
        .join(", ");
    eprintln!("warning [{categories}]: {}", warning.message);
}

fn config_error_message(error: &dyn Error) -> String {
    format!("failed to load configuration: {error}\n\n{CONFIG_EXAMPLE}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use model::Report;
    use std::io;

    struct FailingWriter;

    impl Write for FailingWriter {
        fn write(&mut self, _buffer: &[u8]) -> io::Result<usize> {
            Err(io::Error::new(io::ErrorKind::BrokenPipe, "closed pipe"))
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    #[test]
    fn config_error_message_includes_diagnostic_and_complete_example() {
        let error = std::io::Error::other("invalid configuration");

        let message = config_error_message(&error);

        assert_eq!(
            message,
            format!(
                "failed to load configuration: {error}\n\n{}",
                include_str!("../config.example.toml")
            )
        );
    }

    #[test]
    fn parses_both_supported_flags() {
        assert_eq!(
            parse_args(["--no-things".to_owned(), "--self-check".to_owned()]),
            Ok(Options {
                self_check: true,
                no_things: true,
            })
        );
    }

    #[test]
    fn no_things_bypasses_today_read_and_writes() {
        let mut called = false;

        let outcome = push_if_enabled(true, || {
            called = true;
            Ok(things::PushOutcome {
                added: 1,
                skipped: 0,
                warnings: Vec::new(),
            })
        })
        .unwrap();

        assert_eq!(outcome, None);
        assert!(!called);
    }

    #[test]
    fn report_output_returns_broken_pipe_errors() {
        let config = Config::parse(include_str!("../config.example.toml")).unwrap();
        let report = Report {
            items: Vec::new(),
            warnings: Vec::new(),
        };

        let error = write_report(&mut FailingWriter, &config, &report).unwrap_err();

        assert_eq!(error.kind(), io::ErrorKind::BrokenPipe);
    }
}
