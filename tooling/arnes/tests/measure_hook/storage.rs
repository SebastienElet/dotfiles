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
    run_valid_batch(&harness, 0);
    run_valid_batch(&harness, 24);

    let run = harness.only_run();
    let events = read_jsonl(run.join("events.jsonl"));
    assert_eq!(events.len(), 48);
    assert!(
        events
            .iter()
            .all(|event| event.as_object().unwrap().len() == 3)
    );
    assert!(!run.join("prompts.jsonl").exists());
}

fn run_valid_batch(harness: &Harness, offset: usize) {
    let children: Vec<Child> = (offset..offset + 24)
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
}

#[test]
fn parallel_invalid_hooks_append_complete_safe_records() {
    let harness = Harness::new();
    for _ in 0..2 {
        let children: Vec<Child> = (0..24)
            .map(|_| {
                let mut child = harness.command("codex").spawn().unwrap();
                child.stdin.take().unwrap().write_all(b"{}").unwrap();
                child
            })
            .collect();
        for child in children {
            assert_advisory_failure(&child.wait_with_output().unwrap());
        }
    }

    assert!(harness.runs().is_empty());
    let invalid = read_jsonl(harness.measure_root().join("invalid.jsonl"));
    assert_eq!(invalid.len(), 48);
}
