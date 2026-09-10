use super::support::*;
#[test]
fn creates_managed_directories_and_files_with_private_modes()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let harness = Harness::new()?;
    assert_success(&harness.run(
        "codex",
        br#"{"session_id":"session","event":"SessionStart","prompt":"hello"}"#,
    )?);
    assert_private_tree(&harness.measure_root())?;
    Ok(())
}
fn assert_private_tree(root: &Path) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let _: () = for entry in walk(root)? {
        let metadata = fs::symlink_metadata(&entry)?;
        let mode = metadata.permissions().mode() & 0o777;
        if metadata.is_dir() {
            assert_eq!(mode, 0o700, "directory mode for {entry:?}");
        } else {
            assert_eq!(mode, 0o600, "file mode for {entry:?}");
        }
    };
    Ok(())
}
#[test]
fn parallel_hooks_append_complete_json_lines()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let harness = Harness::new()?;
    run_valid_batch(&harness, 0)?;
    run_valid_batch(&harness, 24)?;
    let run = harness.only_run()?;
    let events = read_jsonl(run.join("events.jsonl"))?;
    assert_eq!(events.len(), 48);
    assert!(
        events
            .iter()
            .map(
                |event| -> Result<_, Box<dyn std::error::Error + Send + Sync>> {
                    Ok(event.as_object().ok_or("expected JSON object")?.len() == 3)
                }
            )
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .all(std::convert::identity)
    );
    assert!(!run.join("prompts.jsonl").exists());
    Ok(())
}
fn run_valid_batch(
    harness: &Harness,
    offset: usize,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let children : Vec < Child > = (offset .. offset + 24) . map (| index | -> Result < _ , Box < dyn std :: error :: Error + Send + Sync > > { Ok ({ let mut child = harness . command ("cursor")  . spawn () ? ; let payload = json ! ({ "conversation_id" : "shared-session" , "hook_event_name" : "beforeSubmitPrompt" , "prompt" : format ! ("prompt-{index}") , "generation_id" : format ! ("generation-{index}") }) ; child . stdin . take () . ok_or ("required test value is missing") ? . write_all (payload . to_string () . as_bytes ()) ? ; child }) }) . collect :: < Result < Vec < _ > , _ > > () ? . into_iter () . collect () ;
    let _: () = for child in children {
        assert_success(&child.wait_with_output()?);
    };
    Ok(())
}
#[test]
fn parallel_invalid_hooks_append_complete_safe_records()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let harness = Harness::new()?;
    for _ in 0..2 {
        let children: Vec<Child> = (0..24)
            .map(|_| -> Result<_, Box<dyn std::error::Error + Send + Sync>> {
                Ok({
                    let mut child = harness.command("codex").spawn()?;
                    child
                        .stdin
                        .take()
                        .ok_or("required test value is missing")?
                        .write_all(b"{}")?;
                    child
                })
            })
            .collect::<Result<Vec<_>, _>>()?;
        for child in children {
            assert_advisory_failure(&child.wait_with_output()?);
        }
    }
    assert!(harness.runs()?.is_empty());
    let invalid = read_jsonl(harness.measure_root().join("invalid.jsonl"))?;
    assert_eq!(invalid.len(), 48);
    Ok(())
}
