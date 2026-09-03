use super::support::*;
use serde_json::{Value, json};

#[test]
fn every_bounded_text_field_returns_a_usable_unicode_limit() {
    for (pointer, field, maximum) in [
        ("/statement", "statement", 500),
        ("/retrieval_terms/0", "retrieval_terms", 100),
        ("/proof/summary", "proof.summary", 1000),
        (
            "/oracle/human_fallback/question",
            "oracle.human_fallback.question",
            500,
        ),
        (
            "/oracle/human_fallback/valid_when",
            "oracle.human_fallback.valid_when",
            500,
        ),
        ("/oracle/outcomes/valid", "oracle.outcomes.valid", 500),
        (
            "/oracle/outcomes/invalidated",
            "oracle.outcomes.invalidated",
            500,
        ),
    ] {
        let fixture = CliFixture::new();
        let draft = fixture.git_draft("invariant", "Text diagnostics.", "text");
        let mut draft: Value = serde_yaml_ng::from_slice(&draft).unwrap();
        for text in [String::new(), "é".repeat(maximum + 1)] {
            *draft.pointer_mut(pointer).unwrap() = json!(text);
            let output = fixture.run(
                ["admit", "--format", "json"],
                &serde_json::to_vec(&draft).unwrap(),
            );
            assert_error(&output, 2, "invalid_field");
            let error: Value = serde_json::from_slice(&output.stderr).unwrap();
            assert_eq!(error["error"]["field"], field);
            assert_eq!(error["error"]["maximum"], maximum);
            assert_eq!(error["error"]["minimum"], 1);
            assert_eq!(error["error"]["unit"], "unicode_scalars");
        }
        *draft.pointer_mut(pointer).unwrap() = json!("é".repeat(maximum));
        assert_exit(
            &fixture.run(
                ["admit", "--format", "json"],
                &serde_json::to_vec(&draft).unwrap(),
            ),
            0,
        );
    }
}

#[test]
fn sensitive_rejections_name_the_pattern_without_echoing_the_content() {
    let fixture = CliFixture::new();
    for (text, criterion) in [
        ("-----BEGIN PRIVATE KEY----- private-canary", "PEM"),
        ("https://private-canary@example.invalid", "userinfo"),
        ("Authorization: private-canary", "header"),
        ("password=private-canary", "assignment"),
        ("ghp_private-canary", "prefix"),
        ("system prompt: private-canary", "marker"),
        ("user: private-canary\nassistant: private-canary", "role"),
    ] {
        let draft = fixture.git_draft("invariant", text, "text");
        let output = fixture.run(["admit", "--format", "json"], &draft);
        assert_error(&output, 2, "sensitive_content");
        let error: Value = serde_json::from_slice(&output.stderr).unwrap();
        assert!(
            error["error"]["message"]
                .as_str()
                .unwrap()
                .contains(criterion),
            "{error}"
        );
        assert!(!String::from_utf8_lossy(&output.stderr).contains("private-canary"));
    }
}

#[test]
fn list_member_rejections_locate_the_offending_item() {
    let fixture = CliFixture::new();
    let draft = fixture.git_draft("invariant", "Text diagnostics.", "text");
    let valid: Value = serde_yaml_ng::from_slice(&draft).unwrap();
    for (field, items, code) in [
        (
            "retrieval_terms",
            json!(["text", "é".repeat(101)]),
            "invalid_field",
        ),
        (
            "retrieval_terms",
            json!(["text", "secret=private-canary"]),
            "sensitive_content",
        ),
        (
            "sources",
            json!([{"kind": "git-file", "locator": "proof.txt"}, {"kind": "private-canary", "locator": "unused"}]),
            "invalid_source_kind",
        ),
        (
            "sources",
            json!([{"kind": "git-file", "locator": "proof.txt"}, {"kind": "user-decision", "locator": "secret=private-canary"}]),
            "sensitive_content",
        ),
    ] {
        let mut draft = valid.clone();
        if field == "sources" {
            draft["proof"][field] = items;
        } else {
            draft[field] = items;
        }
        let output = fixture.run(
            ["admit", "--format", "json"],
            &serde_json::to_vec(&draft).unwrap(),
        );
        assert_error(&output, 2, code);
        let error: Value = serde_json::from_slice(&output.stderr).unwrap();
        assert_eq!(error["error"]["item_index"], 1);
        assert!(!String::from_utf8_lossy(&output.stderr).contains("private-canary"));
    }
}
