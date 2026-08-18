use arnes::diagnostic::{Diagnostic, HumanContext, HumanOptions, HumanSection, Report, State};

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

fn context() -> HumanContext {
    HumanContext::new("Diagnostics")
}

fn section(key: &str, label: &str) -> HumanSection {
    HumanSection::new(key, label)
}

#[test]
fn normal_human_output_hides_healthy_details() {
    assert_eq!(
        format!("{}\n", report().human(&context(), HumanOptions::normal())),
        include_str!("fixtures/diagnostic/report.txt")
    );
}

#[test]
fn verbose_human_output_includes_healthy_details() {
    assert_eq!(
        format!("{}\n", report().human(&context(), HumanOptions::verbose())),
        include_str!("fixtures/diagnostic/report-verbose.txt")
    );
}

#[test]
fn empty_human_report_does_not_claim_health() {
    let report = Report::new(Vec::new());

    assert_eq!(
        report.human(&context(), HumanOptions::normal()),
        "No diagnostics"
    );
}

#[test]
fn report_without_healthy_diagnostics_displays_zero() {
    let report = Report::new(vec![Diagnostic::new(
        "skills",
        State::Unsupported,
        "inventory unavailable",
    )]);

    assert!(
        report
            .human(&context(), HumanOptions::normal())
            .starts_with("Diagnostics\n✓ 0 healthy\n")
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

#[test]
fn human_groups_compact_diagnostics_without_changing_json() {
    let report = Report::new(vec![
        Diagnostic::new("skills", State::Healthy, "verbose first")
            .with_human("claude user external skills", "ponytail · allowed"),
        Diagnostic::new("skills", State::Drift, "verbose second")
            .with_human("claude user external skills", "ponytail-audit · unexpected"),
    ]);

    assert_eq!(
        report.human(&context(), HumanOptions::verbose()),
        "Diagnostics\n✓ 1 healthy\n\nclaude user external skills\n  healthy     ponytail · allowed\n  drift       ponytail-audit · unexpected"
    );
    let json = report.json().unwrap();
    assert!(json.contains("verbose first"));
    assert!(!json.contains("ponytail · allowed"));
}

#[test]
fn structured_sections_sort_by_severity_without_mutating_report_order() {
    let report = Report::new(vec![
        Diagnostic::new("skills", State::Unsupported, "claude limitation")
            .with_human_section(section("claude:user", "CLAUDE")),
        Diagnostic::new("skills", State::Healthy, "cursor current")
            .with_human_section(section("cursor:user", "CURSOR")),
        Diagnostic::new("skills", State::Drift, "cursor missing")
            .with_human_section(section("cursor:user", "CURSOR")),
    ]);

    let output = report.human(&context(), HumanOptions::normal());

    assert!(output.find("CURSOR").unwrap() < output.find("CLAUDE").unwrap());
    assert!(!output.contains("cursor current"));
    assert_eq!(
        report
            .diagnostics()
            .iter()
            .map(|diagnostic| diagnostic.message.as_str())
            .collect::<Vec<_>>(),
        ["claude limitation", "cursor current", "cursor missing"]
    );
}

#[test]
fn verbose_places_healthy_diagnostics_after_other_states() {
    let report = Report::new(vec![
        Diagnostic::new("skills", State::Healthy, "current")
            .with_human_section(section("cursor:user", "CURSOR")),
        Diagnostic::new("skills", State::Unsupported, "inventory unavailable")
            .with_human_section(section("cursor:user", "CURSOR")),
    ]);

    let output = report.human(&context(), HumanOptions::verbose());

    assert!(output.find("inventory unavailable").unwrap() < output.find("current").unwrap());
}

#[test]
fn healthy_only_section_is_hidden_until_verbose() {
    let report = Report::new(vec![
        Diagnostic::new("skills", State::Healthy, "claude current")
            .with_human_section(section("claude:user", "CLAUDE")),
    ]);

    assert!(
        !report
            .human(&context(), HumanOptions::normal())
            .contains("CLAUDE")
    );
    assert!(
        report
            .human(&context(), HumanOptions::verbose())
            .contains("CLAUDE")
    );
}
