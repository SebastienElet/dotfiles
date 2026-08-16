use arnes::diagnostic::{Diagnostic, Report, State};

fn report() -> Report {
    Report::new(vec![
        Diagnostic::new("manifest", State::Healthy, "manifest is valid"),
        Diagnostic::new(
            "rules",
            State::Unsupported,
            "cursor does not expose native user rules",
        ),
        Diagnostic::new("skills", State::Drift, "destination is missing"),
        Diagnostic::new("config", State::Error, "settings.json could not be read"),
    ])
}

#[test]
fn human_output_matches_the_exact_fixture() {
    assert_eq!(
        format!("{}\n", report().human()),
        include_str!("fixtures/diagnostic/report.txt")
    );
}

#[test]
fn json_output_matches_the_exact_fixture() {
    assert_eq!(
        format!("{}\n", report().json().unwrap()),
        include_str!("fixtures/diagnostic/report.json")
    );
}

#[test]
fn report_preserves_diagnostic_order() {
    let report = report();
    let resources = report
        .diagnostics()
        .iter()
        .map(|diagnostic| diagnostic.resource.as_str())
        .collect::<Vec<_>>();

    assert_eq!(resources, ["manifest", "rules", "skills", "config"]);
}

#[test]
fn human_output_keeps_each_diagnostic_on_one_line() {
    let diagnostic = Diagnostic::new("skills\nconfig", State::Drift, "missing\r\ndestination");

    assert_eq!(
        diagnostic.to_string(),
        "drift skills\\nconfig: missing\\r\\ndestination"
    );
}

#[test]
fn exit_codes_prioritize_errors_over_drift() {
    for (states, expected) in [
        (vec![], 0),
        (vec![State::Healthy], 0),
        (vec![State::Unsupported], 0),
        (vec![State::Drift], 1),
        (vec![State::Error], 2),
        (vec![State::Error, State::Drift], 2),
    ] {
        let report = Report::new(
            states
                .into_iter()
                .map(|state| Diagnostic::new("resource", state, "message"))
                .collect(),
        );

        assert_eq!(report.exit_code(), expected);
    }
}
