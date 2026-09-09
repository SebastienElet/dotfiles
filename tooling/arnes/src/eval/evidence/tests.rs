use super::*;
use crate::eval::{
    compare::compare,
    contracts::{Observation, Oracle, Tool},
    report::{Controls, Harness, ReportCase, Run},
    sources::load_cases,
};

fn report() -> Report {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let cases = load_cases(&root)
        .unwrap()
        .into_iter()
        .map(|case| ReportCase {
            definition: case.definition,
            prompt: case.prompt,
            prompt_fingerprint: case.prompt_fingerprint,
            sources: case.sources,
            fixture_revision: "a".repeat(64),
            runs: vec![Run {
                status: Status::Fail,
                error: None,
                observations: vec![],
                tokens: None,
                tool_calls: None,
                duration_ms: None,
            }],
        })
        .collect();
    Report {
        schema_version: 1,
        agent: "fixture-smoke".into(),
        agent_version: "1".into(),
        model: "synthetic".into(),
        date: "2026-09-09T00:00:00Z".into(),
        harness: Harness {
            git_revision: "a".repeat(40),
            instruction_fingerprint: "a".repeat(64),
            skill_fingerprint: "a".repeat(64),
            variant: "baseline".into(),
        },
        runner_revision: "a".repeat(64),
        environment: Environment::Legacy {
            platform: "darwin".into(),
            architecture: "arm64".into(),
            bun: "1".into(),
        },
        controls: Controls {
            sandbox: "workspace-write".into(),
            network: false,
            tools: "shell-with-synthetic-cat-rg-fd-colgrep-v1".into(),
            timeout_seconds: 10,
            reasoning_effort: "low".into(),
            token_budget: (),
        },
        run_count: 1,
        cases,
        limitations: vec!["Synthetic".into()],
    }
}

#[test]
fn snapshots_refuse_forgery_without_current_source_comparison() {
    let original = report();
    validate_report(&original).unwrap();
    let mut changed = original.clone();
    changed.run_count = 2;
    assert!(validate_report(&changed).is_err());
    changed = original.clone();
    changed.cases[0].runs[0].status = Status::Pass;
    assert!(validate_report(&changed).is_err());
    changed = original.clone();
    changed.cases[0].prompt = "different bytes".into();
    assert!(validate_report(&changed).is_err());
    changed = original.clone();
    changed.cases[0].sources[0].heading = "forged".into();
    assert!(validate_report(&changed).is_err());
    changed = original.clone();
    changed.cases.push(changed.cases[0].clone());
    assert!(validate_report(&changed).is_err());
}

#[test]
fn publishing_never_replaces_existing_history() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("report.json");
    publish_report(&path, &report()).unwrap();
    let bytes = fs::read(&path).unwrap();
    assert!(publish_report(&path, &report()).is_err());
    assert_eq!(fs::read(&path).unwrap(), bytes);
    assert!(assert_new_report(&directory.path().join("missing/report.json")).is_err());
    read_report(&path).unwrap();
}

#[test]
fn comparison_refuses_changed_controls_and_cross_runtime_reports() {
    let original = report();
    assert_eq!(
        compare(&original, &original).unwrap().claim,
        "descriptive-only"
    );
    let mut changed = original.clone();
    changed.model = "other".into();
    assert!(compare(&original, &changed).is_err());
    changed = original.clone();
    changed.environment = Environment::Rust {
        platform: "darwin".into(),
        architecture: "arm64".into(),
        runtime: "rust".into(),
    };
    assert!(compare(&original, &changed).is_err());
    changed = original.clone();
    changed.controls.timeout_seconds = 9;
    assert!(compare(&original, &changed).is_err());
    changed = original.clone();
    changed.cases[0].fixture_revision = "b".repeat(64);
    assert!(compare(&original, &changed).is_err());
}

#[test]
fn strict_reports_require_nullable_fields_and_valid_dates() {
    let original = report();
    for field in ["error", "tokens", "toolCalls", "durationMs"] {
        let mut value = serde_json::to_value(&original).unwrap();
        value["cases"][0]["runs"][0]
            .as_object_mut()
            .unwrap()
            .remove(field);
        assert!(serde_json::from_value::<Report>(value).is_err(), "{field}");
    }
    for date in [
        "2026-02-30T00:00:00Z",
        "2026-09-09T24:00:00Z",
        "2026-09-09T00:00:00+00:00",
    ] {
        assert!(validate_date(date).is_err());
    }
    let mut value = serde_json::to_value(&original).unwrap();
    value["environment"]["runtime"] = "rust".into();
    assert!(serde_json::from_value::<Report>(value).is_err());
}

#[test]
fn oracle_requires_successful_tools_and_expected_order() {
    let read = Observation {
        tool: Tool::Cat,
        args: vec![".agents/skills/code-search/SKILL.md".into()],
        exit_code: 0,
    };
    let search = Observation {
        tool: Tool::ColgrepSearch,
        args: vec![],
        exit_code: 0,
    };
    for oracle in [Oracle::StructuralV1, Oracle::LiteralV1, Oracle::KnownPathV1] {
        assert_eq!(evaluate(oracle, &[]), Status::Fail);
    }
    assert_eq!(
        evaluate(Oracle::StructuralV1, &[read.clone(), search.clone()]),
        Status::Pass
    );
    assert_eq!(
        evaluate(Oracle::StructuralV1, &[search.clone(), read]),
        Status::Fail
    );
    let literal = Observation {
        tool: Tool::Rg,
        args: vec!["FEATURE_FLAG_DISABLED".into()],
        exit_code: 0,
    };
    assert_eq!(
        evaluate(Oracle::LiteralV1, std::slice::from_ref(&literal)),
        Status::Pass
    );
    assert_eq!(
        evaluate(Oracle::LiteralV1, &[literal.clone(), search]),
        Status::Fail
    );
    let known = Observation {
        tool: Tool::Cat,
        args: vec!["src/auth/session.ts".into()],
        exit_code: 0,
    };
    assert_eq!(
        evaluate(Oracle::KnownPathV1, std::slice::from_ref(&known)),
        Status::Pass
    );
    assert_eq!(
        evaluate(Oracle::KnownPathV1, &[known, literal]),
        Status::Fail
    );
}

#[test]
fn invalid_runs_stay_in_denominator_and_null_measurements_propagate() {
    let mut baseline = report();
    baseline.cases[2].runs[0].status = Status::Pass;
    baseline.cases[2].runs[0].observations = vec![Observation {
        tool: Tool::Cat,
        args: vec!["src/auth/session.ts".into()],
        exit_code: 0,
    }];
    let mut candidate = baseline.clone();
    candidate.cases[2].runs[0].status = Status::Invalid;
    candidate.cases[2].runs[0].error = Some(crate::eval::report::RunError::Timeout);
    let result = compare(&baseline, &candidate).unwrap();
    assert_eq!(result.baseline.pass_rate, 1.0 / 3.0);
    assert_eq!(result.candidate.pass_rate, 0.0);
    assert_eq!(result.candidate.invalid, 1);
    assert_eq!(result.regressions[0].case_id, "code-search-known-path");
    assert_eq!(result.regressions[0].run, 1);
    assert_eq!(result.candidate.mean_tokens, None);
    assert_eq!(result.candidate.mean_duration_ms, None);
}
