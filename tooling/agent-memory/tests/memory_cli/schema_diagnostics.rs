use super::support::*;
use serde_json::{Value, json};

#[test]
fn unknown_keys_identify_the_containing_mapping_without_echoing_the_key() {
    let fixture = CliFixture::new();
    let draft = fixture.git_draft("invariant", "Schema diagnostics.", "schema");
    let valid: Value = serde_yaml_ng::from_slice(&draft).unwrap();
    for (pointer, field, allowed) in [
        ("/proof", "proof", "summary and sources"),
        ("/proof/sources/0", "proof.sources", "kind and locator"),
        (
            "/oracle/human_fallback",
            "oracle.human_fallback",
            "question and valid_when",
        ),
    ] {
        let mut draft = valid.clone();
        draft.pointer_mut(pointer).unwrap()["private-canary"] = json!("private-value");
        let output = fixture.run(
            ["admit", "--format", "json"],
            &serde_json::to_vec(&draft).unwrap(),
        );
        assert_error(&output, 2, "unknown_field");
        let error: Value = serde_json::from_slice(&output.stderr).unwrap();
        assert_eq!(error["error"]["field"], field);
        assert!(
            error["error"]["message"]
                .as_str()
                .unwrap()
                .contains(allowed)
        );
        assert!(!String::from_utf8_lossy(&output.stderr).contains("private-"));
    }
}

#[test]
fn numeric_scalars_remain_rejected_in_persisted_text_fields() {
    let fixture = CliFixture::new();
    let draft = fixture.git_draft("invariant", "Schema diagnostics.", "schema");
    let valid: Value = serde_yaml_ng::from_slice(&draft).unwrap();
    for pointer in [
        "/statement",
        "/retrieval_terms/0",
        "/proof/summary",
        "/proof/sources/0/locator",
        "/oracle/human_fallback/question",
        "/oracle/outcomes/valid",
    ] {
        for scalar in [json!(123), json!(true), json!(null), json!(1.2)] {
            let mut draft = valid.clone();
            *draft.pointer_mut(pointer).unwrap() = scalar;
            let output = fixture.run(
                ["admit", "--format", "json"],
                &serde_json::to_vec(&draft).unwrap(),
            );
            assert_error(&output, 2, "invalid_field");
        }
    }
}

#[test]
fn custom_yaml_tags_are_rejected_with_a_safe_repair() {
    let fixture = CliFixture::new();
    let draft =
        String::from_utf8(fixture.git_draft("invariant", "Schema diagnostics.", "schema")).unwrap();
    for (from, to, field) in [
        ("statement:", "statement: !private-canary", "statement"),
        ("summary:", "summary: !private-canary", "proof.summary"),
        ("proof:", "proof: !private-canary", "proof"),
    ] {
        let output = fixture.run(
            ["admit", "--format", "json"],
            draft.replace(from, to).as_bytes(),
        );
        assert_error(&output, 2, "invalid_field");
        let error: Value = serde_json::from_slice(&output.stderr).unwrap();
        assert_eq!(error["error"]["field"], field);
        assert!(error["error"]["message"].as_str().unwrap().contains("tag"));
        assert!(!String::from_utf8_lossy(&output.stderr).contains("private-canary"));
    }
    let output = fixture.run(
        ["admit", "--format", "json"],
        draft
            .replace("kind: invariant", "kind: !custom invariant")
            .as_bytes(),
    );
    assert_exit(&output, 0);
}

#[test]
fn an_accepted_kind_tag_does_not_hide_the_actual_rejected_field() {
    let fixture = CliFixture::new();
    let draft =
        String::from_utf8(fixture.git_draft("invariant", "Schema diagnostics.", "schema")).unwrap();
    let draft = draft
        .replace("kind: invariant", "kind: !custom invariant")
        .replace("statement: \"Schema diagnostics.\"", "statement: []");
    let output = fixture.run(["admit", "--format", "json"], draft.as_bytes());
    assert_error(&output, 2, "invalid_field");
    let error: Value = serde_json::from_slice(&output.stderr).unwrap();
    assert_eq!(error["error"]["field"], "statement");
}

#[test]
fn missing_nested_fields_identify_the_required_field() {
    let fixture = CliFixture::new();
    let draft = fixture.git_draft("invariant", "Schema diagnostics.", "schema");
    let valid: Value = serde_yaml_ng::from_slice(&draft).unwrap();
    for (pointer, key, field) in [
        ("", "statement", "statement"),
        ("/proof", "summary", "proof.summary"),
        ("/proof/sources/0", "locator", "proof.sources.locator"),
        (
            "/oracle/human_fallback",
            "question",
            "oracle.human_fallback.question",
        ),
        ("/oracle/automated", "expected", "oracle.automated.expected"),
    ] {
        let mut draft = valid.clone();
        draft
            .pointer_mut(pointer)
            .unwrap()
            .as_object_mut()
            .unwrap()
            .remove(key);
        let output = fixture.run(
            ["admit", "--format", "json"],
            &serde_json::to_vec(&draft).unwrap(),
        );
        assert_error(&output, 2, "invalid_field");
        let error: Value = serde_json::from_slice(&output.stderr).unwrap();
        assert_eq!(error["error"]["field"], field);
    }
}
