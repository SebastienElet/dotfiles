use crate::command;
use crate::model::Report;
use crate::report::things_title;
use crate::util::percent_encode;
use std::collections::HashSet;

const TODAY_SCRIPT: &str =
    "tell application \"Things3\" to return (name of to dos of list \"Today\") as text";

#[derive(Debug, Eq, PartialEq)]
pub struct PushOutcome {
    pub added: usize,
    pub skipped: usize,
    pub warnings: Vec<String>,
}

trait Runner {
    fn run(&mut self, program: &str, args: &[String]) -> Result<String, String>;
}

struct SystemRunner;

impl Runner for SystemRunner {
    fn run(&mut self, program: &str, args: &[String]) -> Result<String, String> {
        command::run(program, args).map_err(|error| error.to_string())
    }
}

pub fn parse_today(output: &str) -> HashSet<String> {
    output.lines().map(str::to_owned).collect()
}

pub fn push(report: &Report) -> Result<PushOutcome, String> {
    push_with_runner(report, &mut SystemRunner)
}

fn push_with_runner<R: Runner>(report: &Report, runner: &mut R) -> Result<PushOutcome, String> {
    let (program, args) = today_command();
    let today = runner
        .run(&program, &args)
        .map_err(|error| format!("failed to read Things Today: {error}"))?;
    let mut seen = parse_today(&today);
    let mut attempted = HashSet::new();
    let mut outcome = PushOutcome {
        added: 0,
        skipped: 0,
        warnings: Vec::new(),
    };

    for item in &report.items {
        let title = things_title(item);
        if seen.contains(&title) || !attempted.insert(title.clone()) {
            outcome.skipped += 1;
            continue;
        }

        let url = format!(
            "things:///add?title={}&notes={}&when=today",
            percent_encode(&title),
            percent_encode(&item.url)
        );
        if let Err(error) = runner.run("open", &[url]) {
            outcome
                .warnings
                .push(format!("failed to add Things todo {title:?}: {error}"));
            continue;
        }

        seen.insert(title);
        outcome.added += 1;
    }

    Ok(outcome)
}

fn today_command() -> (String, Vec<String>) {
    (
        "osascript".to_owned(),
        vec![
            "-e".to_owned(),
            "set AppleScript's text item delimiters to linefeed".to_owned(),
            "-e".to_owned(),
            TODAY_SCRIPT.to_owned(),
        ],
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Category, Report, ReportItem};
    use std::collections::{HashSet, VecDeque};

    #[derive(Default)]
    struct FakeRunner {
        calls: Vec<(String, Vec<String>)>,
        results: VecDeque<Result<String, String>>,
    }

    impl FakeRunner {
        fn returning(results: impl IntoIterator<Item = Result<String, String>>) -> Self {
            Self {
                calls: Vec::new(),
                results: results.into_iter().collect(),
            }
        }
    }

    impl Runner for FakeRunner {
        fn run(&mut self, program: &str, args: &[String]) -> Result<String, String> {
            self.calls.push((program.to_owned(), args.to_vec()));
            self.results
                .pop_front()
                .expect("unexpected command invocation")
        }
    }

    fn item(reference: &str, title: &str, url: &str) -> ReportItem {
        ReportItem {
            category: Category::Retour,
            track_index: 0,
            reference: reference.to_owned(),
            title: title.to_owned(),
            url: url.to_owned(),
            event_at: "2026-08-11T08:00:00Z".to_owned(),
            reasons: Vec::new(),
            priority: None,
        }
    }

    fn report(items: Vec<ReportItem>) -> Report {
        Report {
            items,
            warnings: Vec::new(),
        }
    }

    #[test]
    fn today_titles_are_split_only_on_lines() {
        assert_eq!(
            parse_today("One, with comma\nTwo\n"),
            HashSet::from(["One, with comma".to_owned(), "Two".to_owned()])
        );
    }

    #[test]
    fn push_reads_today_before_writing_and_skips_existing_titles() {
        let existing = "[RETOUR] #1 Existing";
        let mut runner = FakeRunner::returning([Ok(format!("{existing}\n")), Ok(String::new())]);
        let report = report(vec![
            item("#1", "Existing", "https://example.test/pr/1"),
            item("#2", "Nouveau", "https://example.test/pr/2?a=b c"),
        ]);

        let outcome = push_with_runner(&report, &mut runner).unwrap();

        assert_eq!(runner.calls.len(), 2);
        assert_eq!(runner.calls[0], today_command());
        assert_eq!(runner.calls[1].0, "open");
        assert_eq!(
            runner.calls[1].1,
            [
                "things:///add?title=%5BRETOUR%5D%20%232%20Nouveau&notes=https%3A%2F%2Fexample.test%2Fpr%2F2%3Fa%3Db%20c&when=today"
                    .to_owned()
            ]
        );
        assert_eq!(outcome.added, 1);
        assert_eq!(outcome.skipped, 1);
        assert!(outcome.warnings.is_empty());
    }

    #[test]
    fn failed_today_read_prevents_every_write() {
        let mut runner = FakeRunner::returning([Err("Things unavailable".to_owned())]);

        let error = push_with_runner(
            &report(vec![item("#2", "New", "https://example.test/pr/2")]),
            &mut runner,
        )
        .unwrap_err();

        assert!(error.contains("failed to read Things Today"));
        assert_eq!(runner.calls, [today_command()]);
    }

    #[test]
    fn individual_write_failures_warn_and_do_not_stop_later_items() {
        let mut runner = FakeRunner::returning([
            Ok(String::new()),
            Err("open failed".to_owned()),
            Ok(String::new()),
        ]);
        let report = report(vec![
            item("#1", "First", "https://example.test/pr/1"),
            item("#2", "Second", "https://example.test/pr/2"),
        ]);

        let outcome = push_with_runner(&report, &mut runner).unwrap();

        assert_eq!(runner.calls.len(), 3);
        assert_eq!(outcome.added, 1);
        assert_eq!(outcome.skipped, 0);
        assert_eq!(outcome.warnings.len(), 1);
        assert!(outcome.warnings[0].contains("[RETOUR] #1 First"));
    }

    #[test]
    fn duplicate_titles_are_attempted_at_most_once_per_run() {
        let duplicate = item("#1", "First", "https://example.test/pr/1");
        let mut runner = FakeRunner::returning([Ok(String::new()), Err("open failed".to_owned())]);

        let outcome =
            push_with_runner(&report(vec![duplicate.clone(), duplicate]), &mut runner).unwrap();

        assert_eq!(runner.calls.len(), 2);
        assert_eq!(outcome.added, 0);
        assert_eq!(outcome.skipped, 1);
        assert_eq!(outcome.warnings.len(), 1);
    }
}
