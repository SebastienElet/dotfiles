use super::support::*;
#[test]
fn records_git_metadata_model_and_deployed_harness_fingerprint()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let harness = Harness::new()?;
    init_repository(&harness.repository, "measurement", "tracked")?;
    fs::write(harness.repository.join("dirty"), "dirty")?;
    fs::create_dir(harness.home.join(".codex"))?;
    fs::write(harness.home.join(".codex/config.toml"), "model='one'")?;
    let first = json ! ({ "session_id" : "one" , "event" : "SessionStart" , "model" : "gpt-test" });
    assert_success(&harness.run("codex", first.to_string().as_bytes())?);
    let first_run = run_record(&harness, "one")?;
    fs::write(harness.home.join(".codex/config.toml"), "model='two'")?;
    let second = json ! ({ "session_id" : "two" , "event" : "SessionStart" });
    assert_success(&harness.run("codex", second.to_string().as_bytes())?);
    let second_run = run_record(&harness, "two")?;
    assert_eq!(
        (*(first_run)
            .get("model_fingerprint")
            .ok_or("missing fixture index model_fingerprint")?)
        .as_str()
        .ok_or("expected JSON string")?
        .len(),
        64
    );
    assert!(!first_run.to_string().contains("gpt-test"));
    assert!(first_run.get("repository").is_none());
    assert!(first_run.get("repository_branch").is_none());
    assert_eq!(
        *(first_run)
            .get("repository_dirty")
            .ok_or("missing fixture index repository_dirty")?,
        true
    );
    assert_eq!(
        (*(first_run)
            .get("repository_commit")
            .ok_or("missing fixture index repository_commit")?)
        .as_str()
        .ok_or("expected JSON string")?
        .len(),
        40
    );
    assert_eq!(
        first_run.as_object().ok_or("expected JSON object")?.len(),
        11
    );
    assert_ne!(
        *(first_run)
            .get("harness_fingerprint")
            .ok_or("missing fixture index harness_fingerprint")?,
        *(second_run)
            .get("harness_fingerprint")
            .ok_or("missing fixture index harness_fingerprint")?
    );
    assert!(!first_run.to_string().contains("model='one'"));
    Ok(())
}
