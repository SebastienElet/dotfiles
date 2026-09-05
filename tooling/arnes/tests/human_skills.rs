#[path = "support/skills.rs"]
mod skill_support;
mod support;

use arnes::diagnostic::{
    Diagnostic, HumanContext, HumanDetail, HumanOptions, HumanSection, Report, State,
};
use skill_support::{configured_fixture, run};

fn section(key: &str, label: &str) -> HumanSection {
    HumanSection::new(key, label)
}

fn diagnostic(section: HumanSection, state: State, message: &str, summary: &str) -> Diagnostic {
    Diagnostic::new("skills", state, message)
        .with_human_summary(summary)
        .with_human_section(section)
}

fn report() -> Report {
    let claude = section("claude:user", "CLAUDE");
    let cursor = section("cursor:user", "CURSOR");
    let codex = section("codex:user", "CODEX");
    Report::new(vec![
        diagnostic(claude.clone(), State::Healthy, "handoff current", "handoff"),
        diagnostic(
            claude,
            State::Unsupported,
            "unsupported registry version 3",
            "plugin registry version",
        ),
        diagnostic(
            cursor.clone(),
            State::Healthy,
            "pr-verdict current",
            "pr-verdict",
        ),
        diagnostic(
            cursor.clone(),
            State::Unsupported,
            "extension skill exposure unavailable",
            "extension skill exposure",
        ),
        diagnostic(
            cursor,
            State::Drift,
            "destination is missing",
            "code-enforcement",
        )
        .with_human_details([
            HumanDetail::new("expected", "managed skill present"),
            HumanDetail::new("actual", "destination missing"),
            HumanDetail::new("path", "~/.cursor/skills/code-enforcement"),
        ]),
        diagnostic(
            codex,
            State::Unsupported,
            "browser plugin version unavailable",
            "browser plugin version/cache",
        ),
    ])
}

fn context() -> HumanContext {
    HumanContext::new("Skills")
        .with_qualifier("user scope")
        .with_section_count("agent", "agents", "all agents")
}

#[test]
fn normal_skills_output_matches_the_exact_fixture() {
    assert_eq!(
        format!("{}\n", report().human(&context(), HumanOptions::normal())),
        include_str!("fixtures/skills/report.txt")
    );
}

#[test]
fn verbose_skills_output_matches_the_exact_fixture() {
    assert_eq!(
        format!("{}\n", report().human(&context(), HumanOptions::verbose())),
        include_str!("fixtures/skills/report-verbose.txt")
    );
}

#[test]
fn skills_doctor_attaches_agent_sections_without_reading_real_home() {
    let fixture = configured_fixture();
    std::fs::remove_file(fixture.home().join(".claude/skills/alpha")).unwrap();
    skill_support::link_home_relative(&fixture, "harness/skills/alpha", ".claude/skills/alpha");
    std::fs::remove_file(fixture.home().join(".cursor/skills/alpha")).unwrap();

    let (code, normal, stderr) = run(&fixture, &["doctor", "skills"]);
    let (_, verbose, verbose_stderr) = run(&fixture, &["doctor", "skills", "-v"]);

    assert_eq!(code, 1, "{normal}");
    assert!(normal.contains("CURSOR"), "{normal}");
    assert!(verbose.find("CURSOR").unwrap() < verbose.find("CLAUDE").unwrap());
    assert!(!normal.contains("HEALTHY"));
    assert!(verbose.contains("HEALTHY"));
    assert!(stderr.is_empty());
    assert!(verbose_stderr.is_empty());
}
