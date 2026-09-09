use super::*;

#[test]
fn loads_original_three_cases_and_activation_prompt() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let cases = load_cases(&root).unwrap();
    assert_eq!(
        cases
            .iter()
            .map(|case| case.definition.id.as_str())
            .collect::<Vec<_>>(),
        [
            "code-search-structural",
            "code-search-literal",
            "code-search-known-path"
        ]
    );
    assert_eq!(
        cases[0].prompt,
        "Je découvre ce monorepo, aide-moi à le cartographier"
    );
    assert!(cases.iter().all(|case| !case.prompt.is_empty()));
}

#[test]
fn refuses_symlink_escape_and_unresolvable_sections() {
    let root = tempfile::tempdir().unwrap();
    let outside = tempfile::tempdir().unwrap();
    fs::write(outside.path().join("private.md"), "secret").unwrap();
    std::os::unix::fs::symlink(outside.path(), root.path().join("escape")).unwrap();
    assert!(read_source(root.path(), "escape/private.md").is_err());
    assert!(section("## Other\ntext", "Missing").is_err());
    assert_eq!(
        section("# Main\n## Rule\ntext\n## Next\nmore", "Rule").unwrap(),
        "## Rule\ntext"
    );
}

#[test]
fn rejects_missing_trigger_index_and_invalid_fixture() {
    let repository = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let root = tempfile::tempdir().unwrap();
    fs::create_dir_all(root.path().join("harness/evals/fixtures")).unwrap();
    fs::create_dir_all(root.path().join("x/evals")).unwrap();
    fs::write(root.path().join("rule.md"), "## Rule\ntext").unwrap();
    fs::write(root.path().join("x/evals/trigger.json"), r#"{"skill":"x","version":"1","queries":[{"query":"yes","should_activate":true,"reason":""},{"query":"no","should_activate":false,"reason":""}]}"#).unwrap();
    let mut definition = load_cases(&repository).unwrap()[0].definition.clone();
    definition.sources = vec![super::super::contracts::Source {
        path: "rule.md".into(),
        heading: "Rule".into(),
    }];
    definition.prompt = Prompt::Trigger {
        trigger_file: "x/evals/trigger.json".into(),
        query_index: 9,
    };
    assert!(
        resolve_case(root.path(), definition.clone())
            .unwrap_err()
            .contains("Unresolved prompt")
    );
    definition.prompt = Prompt::Inline {
        text: "prompt".into(),
    };
    fs::write(
        root.path()
            .join("harness/evals/fixtures/code-search-v1.json"),
        r#"{"id":"code-search-v1","files":{}}"#,
    )
    .unwrap();
    assert!(resolve_case(root.path(), definition).is_err());
}
