use super::support::*;

#[test]
fn records_git_metadata_model_and_deployed_harness_fingerprint() {
    let harness = Harness::new();
    init_repository(&harness.repository, "measurement", "tracked");
    fs::write(harness.repository.join("dirty"), "dirty").unwrap();
    fs::create_dir(harness.home.join(".codex")).unwrap();
    fs::write(harness.home.join(".codex/config.toml"), "model='one'").unwrap();
    let first = json!({"session_id":"one","event":"SessionStart","model":"gpt-test"});
    assert_success(&harness.run("codex", first.to_string().as_bytes()));
    let first_run = harness
        .runs()
        .into_iter()
        .map(|path| read_json(path.join("run.json")))
        .next()
        .unwrap();
    fs::write(harness.home.join(".codex/config.toml"), "model='two'").unwrap();
    let second = json!({"session_id":"two","event":"SessionStart"});
    assert_success(&harness.run("codex", second.to_string().as_bytes()));
    let runs: Vec<Value> = harness
        .runs()
        .into_iter()
        .map(|path| read_json(path.join("run.json")))
        .collect();
    let second_run = runs.iter().find(|run| run["session_id"] == "two").unwrap();

    assert_eq!(first_run["model"], "gpt-test");
    assert_eq!(
        first_run["repository"],
        fs::canonicalize(&harness.repository)
            .unwrap()
            .to_str()
            .unwrap()
    );
    assert_eq!(first_run["repository_branch"], "measurement");
    assert_eq!(first_run["repository_dirty"], true);
    assert_eq!(first_run["repository_commit"].as_str().unwrap().len(), 40);
    assert_eq!(first_run.as_object().unwrap().len(), 12);
    assert_ne!(
        first_run["harness_fingerprint"],
        second_run["harness_fingerprint"]
    );
    assert!(!first_run.to_string().contains("model='one'"));
}
