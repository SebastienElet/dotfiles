use super::*;

#[test]
fn refuses_escaping_paths_and_missing_trigger_polarity() -> Result<(), Box<dyn std::error::Error>> {
    for path in ["../private.md", "a/./b", "/absolute", "a//b", "a b"] {
        assert!(validate_path(path).is_err());
    }
    let trigger: Trigger = serde_json::from_str(
        r#"{"skill":"x","version":"1","queries":[{"query":"Find x","should_activate":true,"reason":"literal"}]}"#,
    )?;
    assert!(trigger.validate().is_err());
    Ok(())
}

#[test]
fn contracts_refuse_unknown_fields_and_unknown_oracles() {
    assert!(
        serde_json::from_str::<Observation>(
            r#"{"tool":"cat","args":[],"exitCode":0,"extra":true}"#
        )
        .is_err()
    );
    assert!(serde_json::from_str::<Oracle>(r#""unknown""#).is_err());
}
