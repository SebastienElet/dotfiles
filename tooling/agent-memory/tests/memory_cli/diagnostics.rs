use super::support::*;
use serde_json::{Value, json};

fn rejection(output: &std::process::Output, code: &str, field: &str) -> Value {
    assert_error(output, 2, code);
    let error: Value = serde_json::from_slice(&output.stderr).unwrap();
    let error = error["error"].clone();
    assert_eq!(error["field"], field);
    assert!(
        error["message"]
            .as_str()
            .is_some_and(|text| text.len() > 20)
    );
    error
}

#[test]
fn statement_rejection_supplies_the_boundary_for_a_single_repair() {
    let fixture = CliFixture::new();
    let draft = fixture.git_draft("invariant", &"é".repeat(558), "diagnostic");
    let error = rejection(
        &fixture.run(["admit", "--format", "json"], &draft),
        "invalid_field",
        "statement",
    );
    assert_eq!(error["minimum"], 1);
    assert_eq!(error["maximum"], 500);
    assert_eq!(error["unit"], "unicode_scalars");
    assert!(!fixture.root().exists());
    let repaired = fixture.git_draft(
        "invariant",
        &"é".repeat(error["maximum"].as_u64().unwrap() as usize),
        "diagnostic",
    );
    assert_exit(&fixture.run(["admit", "--format", "json"], &repaired), 0);
}

#[test]
fn nested_schema_errors_name_a_safe_field_and_expected_shape() {
    let fixture = CliFixture::new();
    let draft = fixture.git_draft("invariant", "Diagnostic memory.", "diagnostic");
    let mut valid: Value = serde_yaml_ng::from_slice(&draft).unwrap();
    valid["scope"] = json!("project");
    let cases = [
        ("/statement", json!([]), "statement", "string"),
        ("/scope", json!("private-canary"), "scope", "project"),
        ("/kind", json!("private-canary"), "kind", "invariant"),
        (
            "/proof/summary",
            json!({"private-canary": "secret"}),
            "proof.summary",
            "string",
        ),
        (
            "/oracle/human_fallback/question",
            json!([]),
            "oracle.human_fallback.question",
            "string",
        ),
        (
            "/oracle/automated/expected",
            json!("private-canary"),
            "oracle.automated.expected",
            "all-proof-sources-unchanged",
        ),
        (
            "/proof/sources/0/locator",
            json!([]),
            "proof.sources.locator",
            "string",
        ),
    ];
    for (pointer, value, field, expected) in cases {
        let mut draft = valid.clone();
        *draft.pointer_mut(pointer).unwrap() = value;
        let output = fixture.run(
            ["admit", "--format", "json"],
            &serde_json::to_vec(&draft).unwrap(),
        );
        let error = rejection(&output, "invalid_field", field);
        assert!(
            error["message"].as_str().unwrap().contains(expected),
            "{error}"
        );
        assert!(!String::from_utf8_lossy(&output.stderr).contains("private-canary"));
    }
}

#[test]
fn admission_rejection_classes_explain_the_required_correction() {
    let fixture = CliFixture::new();
    let draft = fixture.git_draft("invariant", "Diagnostic memory.", "diagnostic");
    let mut valid: Value = serde_yaml_ng::from_slice(&draft).unwrap();
    valid["scope"] = json!("project");
    let cases = [
        (
            "/schema_version",
            json!(2),
            "unsupported_schema",
            "schema_version",
            "1",
        ),
        (
            "/proof/sources",
            json!([]),
            "missing_proof",
            "proof.sources",
            "source",
        ),
        (
            "/oracle/automated",
            Value::Null,
            "missing_oracle",
            "oracle.automated",
            "source-fingerprint",
        ),
        (
            "/proof/sources/0/kind",
            json!("private-canary"),
            "invalid_source_kind",
            "proof.sources.kind",
            "git-file",
        ),
        (
            "/statement",
            json!("secret=private-canary"),
            "sensitive_content",
            "statement",
            "secret",
        ),
        (
            "/retrieval_terms",
            json!(vec!["term"; 21]),
            "too_many_items",
            "retrieval_terms",
            "items",
        ),
    ];
    for (pointer, value, code, field, expected) in cases {
        let mut draft = valid.clone();
        *draft.pointer_mut(pointer).unwrap() = value;
        let output = fixture.run(
            ["admit", "--format", "json"],
            &serde_json::to_vec(&draft).unwrap(),
        );
        let error = rejection(&output, code, field);
        assert!(
            error["message"].as_str().unwrap().contains(expected),
            "{error}"
        );
        if code == "too_many_items" {
            assert_eq!(error["maximum"], 20);
        }
        assert!(!String::from_utf8_lossy(&output.stderr).contains("private-canary"));
    }
}

#[test]
fn yaml_errors_provide_locations_without_echoing_keys_or_values() {
    let fixture = CliFixture::new();
    for (input, code) in [
        ("private-canary: [broken", "malformed_yaml"),
        ("private-canary: 1\nprivate-canary: 2\n", "duplicate_field"),
    ] {
        let output = fixture.run(["admit", "--format", "json"], input.as_bytes());
        let error = rejection(&output, code, "document");
        assert!(error["line"].as_u64().is_some_and(|line| line > 0));
        assert!(!String::from_utf8_lossy(&output.stderr).contains("private-canary"));
    }
}

#[test]
fn retrieve_confirm_and_hook_explain_rejections_without_echoing_input() {
    let fixture = CliFixture::new();
    rejection(
        &fixture.run(["retrieve", "--query-stdin", "--format", "json"], b" \n"),
        "empty_query",
        "query",
    );
    rejection(
        &fixture.run(["retrieve", "--query-stdin", "--format", "json"], &[255]),
        "invalid_utf8",
        "query",
    );
    let error = rejection(
        &fixture.run(
            [
                "confirm",
                "--id",
                "private-canary",
                "--status",
                "confirmed",
                "--reason-stdin",
            ],
            "é".repeat(501).as_bytes(),
        ),
        "invalid_field",
        "transition.reason",
    );
    assert_eq!(error["maximum"], 500);
    for agent in ["claude", "codex"] {
        let payload = json!({"hook_event_name": "UserPromptSubmit", "prompt": "private-canary", "cwd": "relative"});
        let output = fixture.run(
            ["hook", "--agent", agent],
            &serde_json::to_vec(&payload).unwrap(),
        );
        let error = rejection(&output, "invalid_hook_cwd", "cwd");
        assert!(error["message"].as_str().unwrap().contains("absolute"));
        assert!(!String::from_utf8_lossy(&output.stderr).contains("private-canary"));
    }
    let error = rejection(
        &fixture.run(["admit", "--format", "json"], &vec![b'a'; 1_048_577]),
        "input_too_large",
        "stdin",
    );
    assert_eq!(error["maximum"], 1_048_576);
    assert_eq!(error["unit"], "bytes");
}
