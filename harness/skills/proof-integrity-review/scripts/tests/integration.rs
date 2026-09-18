mod bindings;
mod refusals;
mod support;
use serde_json::{Value, json};
use std::fs;
use support::{Fixture, Result, cli, git, receipt, set};

#[test]
fn accepts_complete_bound_declarations() -> Result {
    let fixture = Fixture::new()?;
    let epoch = fixture.epoch()?;
    let output = fixture.gate(&epoch, &receipt(&epoch)?)?;
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    Ok(())
}
#[test]
fn refuses_malformed_and_incomplete_receipts() -> Result {
    let fixture = Fixture::new()?;
    let epoch = fixture.epoch()?;
    let valid = receipt(&epoch)?;
    let cases = [
        ("/schema_version", json!(true)),
        ("/verdict", json!("ALLOW")),
        ("/auditor/fresh_session", json!(false)),
        ("/auditor/forked", json!(true)),
        ("/auditor/author_independent", json!(false)),
        ("/auditor/write_tools_enabled", json!(true)),
        ("/auditor/persistent_memory", json!(true)),
        ("/auditor/session_id", json!("")),
        ("/claims", json!([])),
        ("/witnesses", json!([])),
        ("/claims/3/paths", json!(["missing"])),
        ("/claims/3/id", json!("proof.fail_closed")),
        ("/claims/3/impact", json!("low")),
        ("/witnesses/0/red_exit_code", json!(true)),
        ("/witnesses/0/green_exit_code", json!(1)),
        ("/witnesses/0/mutant_digest", json!("stale")),
        ("/witnesses/0/evidence/green_stdout", json!("forged marker")),
        ("/witnesses/0/mutant/targets/0/path", json!("../Makefile")),
    ];
    for (pointer, replacement) in cases {
        let mut changed = valid.clone();
        set(&mut changed, pointer, replacement)?;
        assert!(
            !fixture.gate(&epoch, &changed)?.status.success(),
            "accepted {pointer}"
        );
    }
    let mut unknown = valid;
    unknown
        .as_object_mut()
        .ok_or("object")?
        .insert("extra".into(), json!(true));
    assert!(!fixture.gate(&epoch, &unknown)?.status.success());
    Ok(())
}
#[test]
fn refuses_stale_candidate_policy_branch_and_untracked_input() -> Result {
    for target in ["Makefile", "policy/SKILL.md", "new-input", "branch"] {
        let fixture = Fixture::new()?;
        let epoch = fixture.epoch()?;
        let receipt = receipt(&epoch)?;
        if target == "branch" {
            git(fixture.root(), &["checkout", "-b", "other"])?;
        } else {
            fs::write(fixture.root().join(target), "changed")?;
        }
        assert!(
            !fixture.gate(&epoch, &receipt)?.status.success(),
            "accepted {target}"
        );
    }
    Ok(())
}
#[test]
fn classifier_handles_moon_harness_and_newline_git_paths() -> Result {
    let output = cli(&[
        "classify",
        "--paths",
        ".moon/tasks.yml",
        "harness/skills/a/SKILL.md",
        "tests/line\nbreak.rs",
    ])?;
    assert!(output.status.success());
    let result: Value = serde_json::from_slice(&output.stdout)?;
    assert_eq!(
        result
            .get("matched_paths")
            .and_then(Value::as_array)
            .map(Vec::len),
        Some(3)
    );
    let fixture = Fixture::new()?;
    fs::create_dir(fixture.root().join("tests"))?;
    fs::write(fixture.root().join("tests/line\nbreak.rs"), [0, 255, 10])?;
    git(fixture.root(), &["add", "."])?;
    git(fixture.root(), &["commit", "-m", "binary filename"])?;
    let epoch = fixture.epoch()?;
    assert!(
        epoch
            .pointer("/subject/changed_paths")
            .and_then(Value::as_array)
            .ok_or("paths")?
            .contains(&json!("tests/line\nbreak.rs"))
    );
    Ok(())
}
#[test]
fn refuses_hidden_index_flags() -> Result {
    for flag in ["--assume-unchanged", "--skip-worktree"] {
        let fixture = Fixture::new()?;
        git(fixture.root(), &["update-index", flag, "Makefile"])?;
        let output = cli(&[
            "epoch",
            "--repository",
            fixture.root().to_str().ok_or("path")?,
            "--base",
            "base",
            "--head",
            "HEAD",
            "--policy-root",
            fixture.root().join("policy").to_str().ok_or("path")?,
        ])?;
        assert!(!output.status.success());
    }
    Ok(())
}

#[test]
fn digest_hashes_raw_bytes_and_strict_canonical_json() -> Result {
    let directory = tempfile::tempdir()?;
    let file = directory.path().join("input");
    fs::write(&file, b"abc")?;
    let output = cli(&["digest", "--file", file.to_str().ok_or("path")?])?;
    assert!(output.status.success());
    let value: Value = serde_json::from_slice(&output.stdout)?;
    assert_eq!(
        value.get("digest"),
        Some(&json!(
            "sha256:ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        ))
    );
    fs::write(&file, b"{\"z\":1,\"a\":2}")?;
    let first = cli(&["digest", "--file", file.to_str().ok_or("path")?, "--json"])?;
    assert!(first.status.success());
    fs::write(&file, b"{ \"a\": 2, \"z\": 1 }")?;
    assert_eq!(
        first.stdout,
        cli(&["digest", "--file", file.to_str().ok_or("path")?, "--json"])?.stdout
    );
    fs::write(&file, b"{\"a\":2,\"a\":2}")?;
    assert!(
        !cli(&["digest", "--file", file.to_str().ok_or("path")?, "--json"])?
            .status
            .success()
    );
    Ok(())
}
