use super::support::*;
use serde_json::{Value, json};
#[test]
fn every_bounded_text_field_returns_a_usable_unicode_limit()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let _: () = for (pointer, field, maximum) in [
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
        let fixture = CliFixture::new()?;
        let draft = CliFixture::git_draft("invariant", "Text diagnostics.", "text")?;
        let mut draft: Value = serde_yaml_ng::from_slice(&draft)?;
        for text in [String::new(), "é".repeat(maximum + 1)] {
            *draft.pointer_mut(pointer).ok_or("expected JSON pointer")? = json!(text);
            let output =
                fixture.run(["admit", "--format", "json"], &serde_json::to_vec(&draft)?)?;
            assert_error(&output, 2, "invalid_field")?;
            let error: Value = serde_json::from_slice(&output.stderr)?;
            assert_eq!(
                (*(*(error)
                    .get("error")
                    .ok_or(concat!("missing fixture index ", stringify!("error")))?)
                .get("field")
                .ok_or(concat!("missing fixture index ", stringify!("field")))?),
                field
            );
            assert_eq!(
                (*(*(error)
                    .get("error")
                    .ok_or(concat!("missing fixture index ", stringify!("error")))?)
                .get("maximum")
                .ok_or(concat!("missing fixture index ", stringify!("maximum")))?),
                maximum
            );
            assert_eq!(
                (*(*(error)
                    .get("error")
                    .ok_or(concat!("missing fixture index ", stringify!("error")))?)
                .get("minimum")
                .ok_or(concat!("missing fixture index ", stringify!("minimum")))?),
                1
            );
            assert_eq!(
                (*(*(error)
                    .get("error")
                    .ok_or(concat!("missing fixture index ", stringify!("error")))?)
                .get("unit")
                .ok_or(concat!("missing fixture index ", stringify!("unit")))?),
                "unicode_scalars"
            );
        }
        *draft.pointer_mut(pointer).ok_or("expected JSON pointer")? = json!("é".repeat(maximum));
        assert_exit(
            &fixture.run(["admit", "--format", "json"], &serde_json::to_vec(&draft)?)?,
            0,
        );
    };
    Ok(())
}
#[test]
fn sensitive_rejections_name_the_pattern_without_echoing_the_content()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = CliFixture::new()?;
    let _: () = for (text, criterion) in [
        ("-----BEGIN PRIVATE KEY----- private-canary", "PEM"),
        ("https://private-canary@example.invalid", "userinfo"),
        ("Authorization: private-canary", "header"),
        ("password=private-canary", "assignment"),
        ("ghp_private-canary", "prefix"),
        ("system prompt: private-canary", "marker"),
        ("user: private-canary\nassistant: private-canary", "role"),
    ] {
        let draft = CliFixture::git_draft("invariant", text, "text")?;
        let output = fixture.run(["admit", "--format", "json"], &draft)?;
        assert_error(&output, 2, "sensitive_content")?;
        let error: Value = serde_json::from_slice(&output.stderr)?;
        assert!(
            (*(*(error)
                .get("error")
                .ok_or(concat!("missing fixture index ", stringify!("error")))?)
            .get("message")
            .ok_or(concat!("missing fixture index ", stringify!("message")))?)
            .as_str()
            .ok_or("expected JSON string")?
            .contains(criterion),
            "{error}"
        );
        assert!(!String::from_utf8_lossy(&output.stderr).contains("private-canary"));
    };
    Ok(())
}
#[test]
fn list_member_rejections_locate_the_offending_item()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = CliFixture::new()?;
    let draft = CliFixture::git_draft("invariant", "Text diagnostics.", "text")?;
    let valid: Value = serde_yaml_ng::from_slice(&draft)?;
    let _: () = for (field, items, code) in [
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
            json ! ([{ "kind" : "git-file" , "locator" : "proof.txt" } , { "kind" : "private-canary" , "locator" : "unused" }]),
            "invalid_source_kind",
        ),
        (
            "sources",
            json ! ([{ "kind" : "git-file" , "locator" : "proof.txt" } , { "kind" : "user-decision" , "locator" : "secret=private-canary" }]),
            "sensitive_content",
        ),
    ] {
        let mut draft = valid.clone();
        if field == "sources" {
            (*(*(draft)
                .get_mut("proof")
                .ok_or(concat!("missing fixture index ", stringify!("proof")))?)
            .get_mut(field)
            .ok_or(concat!("missing fixture index ", stringify!(field)))?) = items;
        } else {
            (*(draft)
                .get_mut(field)
                .ok_or(concat!("missing fixture index ", stringify!(field)))?) = items;
        }
        let output = fixture.run(["admit", "--format", "json"], &serde_json::to_vec(&draft)?)?;
        assert_error(&output, 2, code)?;
        let error: Value = serde_json::from_slice(&output.stderr)?;
        assert_eq!(
            (*(*(error)
                .get("error")
                .ok_or(concat!("missing fixture index ", stringify!("error")))?)
            .get("item_index")
            .ok_or(concat!("missing fixture index ", stringify!("item_index")))?),
            1
        );
        assert!(!String::from_utf8_lossy(&output.stderr).contains("private-canary"));
    };
    Ok(())
}
