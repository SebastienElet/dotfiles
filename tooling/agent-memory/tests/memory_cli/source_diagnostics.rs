use super::support::*;
use serde_json::{Value, json};

#[test]
fn source_rejections_identify_the_criterion_and_source_without_its_locator() {
    let fixture = CliFixture::new();
    let draft = fixture.git_draft("invariant", "Source diagnostics.", "source diagnostics");
    let valid: Value = serde_yaml_ng::from_slice(&draft).unwrap();
    std::fs::write(fixture.repository().join("private-canary"), "proof").unwrap();
    let cases = [
        ("git-file", "../private-canary", "relative"),
        ("git-file", "missing-private-canary", "does not exist"),
        ("git-file", "private-canary", "tracked"),
        ("local-file", "private-canary", "absolute"),
        ("official-url", "http://private-canary.example", "HTTPS"),
    ];
    for (kind, locator, expected) in cases {
        let mut draft = valid.clone();
        draft["proof"]["sources"] = json!([
            {"kind": "user-decision", "locator": "The user approved the primary domain."},
            {"kind": kind, "locator": locator},
        ]);
        let output = fixture.run(
            ["admit", "--format", "json"],
            &serde_json::to_vec(&draft).unwrap(),
        );
        assert_error(&output, 2, "source_invalid");
        let error: Value = serde_json::from_slice(&output.stderr).unwrap();
        assert_eq!(error["error"]["item_index"], 1);
        assert!(
            error["error"]["message"]
                .as_str()
                .unwrap()
                .contains(expected),
            "{error}"
        );
        assert!(!String::from_utf8_lossy(&output.stderr).contains("private-canary"));
    }
}

#[test]
fn official_url_requires_an_explicit_user_decision_before_fetching() {
    let fixture = CliFixture::new();
    let draft = fixture.git_draft("invariant", "Source diagnostics.", "source diagnostics");
    let mut draft: Value = serde_yaml_ng::from_slice(&draft).unwrap();
    draft["proof"]["sources"][0] =
        json!({"kind": "official-url", "locator": "https://unused.invalid"});
    let output = fixture.run(
        ["admit", "--format", "json"],
        &serde_json::to_vec(&draft).unwrap(),
    );
    assert_error(&output, 2, "source_invalid");
    let error: Value = serde_json::from_slice(&output.stderr).unwrap();
    assert!(
        error["error"]["message"]
            .as_str()
            .unwrap()
            .contains("user-decision")
    );
}

#[test]
fn oversized_source_keeps_unavailable_exit_and_names_the_size_limit() {
    let fixture = CliFixture::new();
    std::fs::write(
        fixture.repository().join("proof.txt"),
        vec![b'a'; 1_048_577],
    )
    .unwrap();
    let draft = fixture.git_draft("invariant", "Source diagnostics.", "source diagnostics");
    let output = fixture.run(["admit", "--format", "json"], &draft);
    assert_error(&output, 4, "source_unavailable");
    let error: Value = serde_json::from_slice(&output.stderr).unwrap();
    assert!(
        error["error"]["message"]
            .as_str()
            .unwrap()
            .contains("1048576")
    );
}

#[test]
fn incompatible_confirmation_names_the_actual_kinds_allowed_statuses() {
    let fixture = CliFixture::new();
    let draft = fixture.git_draft("decision", "Source diagnostics.", "source diagnostics");
    let stored = fixture.run(["admit", "--format", "json"], &draft);
    let stored = stdout_json(&stored);
    let id = stored["id"].as_str().unwrap();
    let output = fixture.run(
        [
            "confirm",
            "--id",
            id,
            "--status",
            "achieved",
            "--reason-stdin",
        ],
        b"A human concluded the decision.",
    );
    assert_error(&output, 2, "invalid_human_conclusion");
    let error: Value = serde_json::from_slice(&output.stderr).unwrap();
    assert_eq!(
        error["error"]["message"],
        "For a decision, the only human terminal status is superseded."
    );
    assert_exit(
        &fixture.run(
            [
                "confirm",
                "--id",
                id,
                "--status",
                "superseded",
                "--reason-stdin",
            ],
            b"A human concluded the decision.",
        ),
        0,
    );
}
