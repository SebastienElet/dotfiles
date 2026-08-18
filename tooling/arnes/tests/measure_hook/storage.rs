use super::support::*;

#[test]
fn creates_managed_directories_and_files_with_private_modes() {
    let harness = Harness::new();
    assert_success(&harness.run(
        "codex",
        br#"{"session_id":"session","event":"SessionStart","prompt":"hello"}"#,
    ));

    assert_private_tree(&harness.measure_root());
}

fn assert_private_tree(root: &Path) {
    for entry in walk(root) {
        let metadata = fs::symlink_metadata(&entry).unwrap();
        let mode = metadata.permissions().mode() & 0o777;
        if metadata.is_dir() {
            assert_eq!(mode, 0o700, "directory mode for {entry:?}");
        } else {
            assert_eq!(mode, 0o600, "file mode for {entry:?}");
        }
    }
}

#[test]
fn parallel_hooks_append_complete_json_lines() {
    let harness = Harness::new();
    let children: Vec<Child> = (0..24)
        .map(|index| {
            let mut child = harness.command("cursor").spawn().unwrap();
            let payload = json!({
                "conversation_id":"shared-session",
                "hook_event_name":"beforeSubmitPrompt",
                "prompt":format!("prompt-{index}"),
                "generation_id":format!("generation-{index}")
            });
            child
                .stdin
                .take()
                .unwrap()
                .write_all(payload.to_string().as_bytes())
                .unwrap();
            child
        })
        .collect();
    for child in children {
        assert_success(&child.wait_with_output().unwrap());
    }

    let run = harness.only_run();
    let events = read_jsonl(run.join("events.jsonl"));
    let prompts = read_jsonl(run.join("prompts.jsonl"));
    assert_eq!(events.len(), 24);
    assert_eq!(prompts.len(), 24);
    let mut ids: Vec<&str> = events
        .iter()
        .map(|event| event["event_id"].as_str().unwrap())
        .collect();
    ids.sort_unstable();
    ids.dedup();
    assert_eq!(ids.len(), 24);
}
