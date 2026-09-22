use crate::support::{Fixture, Result, cli, digest, git, rebind, receipt, set};
use serde_json::{Value, json};
use std::fs;

#[test]
fn accepts_policy_targets_and_fixture_only_generic_claims() -> Result {
    let fixture = Fixture::new()?;
    let epoch = fixture.epoch()?;
    let valid = receipt(&epoch)?;
    let mut policy = valid.clone();
    for witness in policy
        .get_mut("witnesses")
        .and_then(Value::as_array_mut)
        .ok_or("witnesses")?
    {
        let target = witness.pointer_mut("/mutant/targets/0").ok_or("target")?;
        set(target, "/scope", json!("policy"))?;
        set(target, "/path", json!("SKILL.md"))?;
        set(target, "/before_content", json!("policy"))?;
        set(target, "/before_digest", json!(digest(b"policy")))?;
    }
    rebind(&mut policy)?;
    let output = fixture.gate(&epoch, &policy)?;
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let mut mixed = valid;
    set(
        &mut mixed,
        "/witnesses/0/mutant/targets/0/scope",
        json!("fixture"),
    )?;
    set(
        &mut mixed,
        "/witnesses/0/evidence/injection_mode",
        json!("input-replacement"),
    )?;
    rebind(&mut mixed)?;
    assert!(fixture.gate(&epoch, &mixed)?.status.success());
    set(
        &mut mixed,
        "/witnesses/3/mutant/targets/0/scope",
        json!("fixture"),
    )?;
    set(
        &mut mixed,
        "/witnesses/3/evidence/injection_mode",
        json!("input-replacement"),
    )?;
    rebind(&mut mixed)?;
    assert!(!fixture.gate(&epoch, &mixed)?.status.success());
    Ok(())
}
#[test]
fn binds_moving_base_head_and_repository_location() -> Result {
    for change in ["base", "head", "remote"] {
        let fixture = Fixture::new()?;
        let epoch = fixture.epoch()?;
        let valid = receipt(&epoch)?;
        match change {
            "base" => git(fixture.root(), &["branch", "-f", "base", "HEAD"])?,
            "head" => git(
                fixture.root(),
                &["commit", "--allow-empty", "-m", "new candidate"],
            )?,
            _ => git(
                fixture.root(),
                &["remote", "add", "origin", "https://example.invalid/repo"],
            )?,
        }
        assert!(
            !fixture.gate(&epoch, &valid)?.status.success(),
            "accepted {change}"
        );
    }
    let first = Fixture::new()?;
    let second = Fixture::new()?;
    let mut epoch = first.epoch()?;
    let valid = receipt(&epoch)?;
    set(&mut epoch, "/repository", json!(second.root()))?;
    assert!(!second.gate(&epoch, &valid)?.status.success());
    Ok(())
}
#[test]
fn ignored_policy_inputs_are_bound_and_ignored_repository_targets_refuse() -> Result {
    let fixture = Fixture::new()?;
    fs::write(fixture.root().join("policy/ignored"), "bound policy")?;
    fs::write(fixture.root().join("ignored"), "new")?;
    let epoch = fixture.epoch()?;
    let valid = receipt(&epoch)?;
    let mut ignored_target = valid.clone();
    set(
        &mut ignored_target,
        "/witnesses/3/mutant/targets/0/path",
        json!("ignored"),
    )?;
    rebind(&mut ignored_target)?;
    assert!(!fixture.gate(&epoch, &ignored_target)?.status.success());
    fs::write(fixture.root().join("policy/ignored"), "changed")?;
    assert!(!fixture.gate(&epoch, &valid)?.status.success());
    Ok(())
}
#[test]
fn policy_digest_is_portable_and_excludes_only_generated_directories() -> Result {
    let fixture = Fixture::new()?;
    let epoch = fixture.epoch()?;
    for directory in ["target", ".proof-integrity", ".git"] {
        fs::create_dir(fixture.root().join("policy").join(directory))?;
        fs::write(
            fixture
                .root()
                .join("policy")
                .join(directory)
                .join("artifact"),
            "runtime",
        )?;
    }
    let after = fixture.epoch()?;
    assert_eq!(epoch.get("policy_digest"), after.get("policy_digest"));
    let other = Fixture::new()?;
    assert_eq!(
        epoch.get("policy_digest"),
        other.epoch()?.get("policy_digest")
    );
    Ok(())
}
#[test]
fn renamed_and_deleted_paths_remain_in_the_epoch() -> Result {
    let fixture = Fixture::new()?;
    git(fixture.root(), &["mv", "Makefile", "Justfile"])?;
    git(fixture.root(), &["commit", "-m", "rename"])?;
    let epoch = fixture.epoch()?;
    let paths = epoch
        .pointer("/subject/changed_paths")
        .and_then(Value::as_array)
        .ok_or("paths")?;
    assert!(paths.contains(&json!("Makefile")));
    assert!(paths.contains(&json!("Justfile")));
    let states = epoch
        .pointer("/subject/path_states")
        .and_then(Value::as_array)
        .ok_or("states")?;
    assert!(
        states
            .iter()
            .any(|state| state.get("path") == Some(&json!("Makefile"))
                && state.get("kind") == Some(&json!("deleted")))
    );
    Ok(())
}
#[test]
fn worktree_mode_includes_committed_and_new_inputs() -> Result {
    let fixture = Fixture::new()?;
    fs::write(fixture.root().join("moon.yml"), "tasks: {}")?;
    let output = cli(&[
        "classify",
        "--repository",
        fixture.root().to_str().ok_or("path")?,
        "--base",
        "base",
        "--worktree",
    ])?;
    assert!(output.status.success());
    let value: Value = serde_json::from_slice(&output.stdout)?;
    let matched = value
        .get("matched_paths")
        .and_then(Value::as_array)
        .ok_or("paths")?;
    assert!(matched.contains(&json!("Makefile")));
    assert!(matched.contains(&json!("moon.yml")));
    Ok(())
}
#[cfg(unix)]
#[test]
fn tracked_symlinks_are_bound_but_mutation_and_policy_symlinks_refuse() -> Result {
    use std::os::unix::fs::symlink;
    let fixture = Fixture::new()?;
    symlink("Makefile", fixture.root().join("link"))?;
    git(fixture.root(), &["add", "link"])?;
    git(fixture.root(), &["commit", "-m", "link"])?;
    let epoch = fixture.epoch()?;
    let mut valid = receipt(&epoch)?;
    assert!(fixture.gate(&epoch, &valid)?.status.success());
    set(
        &mut valid,
        "/witnesses/3/mutant/targets/0/path",
        json!("link"),
    )?;
    rebind(&mut valid)?;
    assert!(!fixture.gate(&epoch, &valid)?.status.success());
    symlink("../Makefile", fixture.root().join("policy/link"))?;
    assert!(!fixture.gate(&epoch, &receipt(&epoch)?)?.status.success());
    Ok(())
}
#[cfg(unix)]
#[test]
fn rejects_physical_aliases_even_with_different_paths() -> Result {
    let fixture = Fixture::new()?;
    fs::hard_link(
        fixture.root().join("Makefile"),
        fixture.root().join("alias"),
    )?;
    git(fixture.root(), &["add", "alias"])?;
    git(fixture.root(), &["commit", "-m", "alias"])?;
    let epoch = fixture.epoch()?;
    let mut valid = receipt(&epoch)?;
    let mut alias = valid
        .pointer("/witnesses/3/mutant/targets/0")
        .ok_or("target")?
        .clone();
    set(&mut alias, "/path", json!("alias"))?;
    valid
        .pointer_mut("/witnesses/3/mutant/targets")
        .and_then(Value::as_array_mut)
        .ok_or("targets")?
        .push(alias);
    rebind(&mut valid)?;
    assert!(!fixture.gate(&epoch, &valid)?.status.success());
    Ok(())
}

#[test]
fn refuses_policy_root_missing_any_essential_owned_source() -> Result {
    for missing in [
        "scripts/src/gate.rs",
        "scripts/src/rules.json",
        "scripts/Cargo.lock",
        "references/cli.md",
    ] {
        let fixture = Fixture::new()?;
        let path = fixture.root().join("policy").join(missing);
        if path.exists() {
            fs::remove_file(path)?;
        }
        let output = cli(&[
            "epoch",
            "--repository",
            fixture.root().to_str().ok_or("path")?,
            "--base",
            "base",
            "--policy-root",
            fixture.root().join("policy").to_str().ok_or("path")?,
        ])?;
        assert!(
            !output.status.success(),
            "accepted incomplete policy missing {missing}"
        );
    }
    Ok(())
}

#[test]
fn committed_receipt_cannot_replace_candidate_with_dirty_live_target() -> Result {
    for target in ["Makefile", "policy/SKILL.md"] {
        let fixture = Fixture::new()?;
        fs::write(fixture.root().join(target), "dirty live input")?;
        let epoch = fixture.epoch()?;
        let mut valid = receipt(&epoch)?;
        for witness in valid
            .get_mut("witnesses")
            .and_then(Value::as_array_mut)
            .ok_or("witnesses")?
        {
            let mutation = witness.pointer_mut("/mutant/targets/0").ok_or("target")?;
            set(mutation, "/path", json!(target))?;
            set(mutation, "/before_content", json!("dirty live input"))?;
            set(
                mutation,
                "/before_digest",
                json!(digest(b"dirty live input")),
            )?;
        }
        rebind(&mut valid)?;
        assert!(
            !fixture.gate(&epoch, &valid)?.status.success(),
            "accepted dirty {target}"
        );
    }
    Ok(())
}

#[test]
fn categories_do_not_require_artificial_high_impact_claims() -> Result {
    for path in ["permissions.toml", "test_example.py", "custom-check.conf"] {
        let fixture = Fixture::new()?;
        fs::write(fixture.root().join(path), "editorial change")?;
        git(fixture.root(), &["add", path])?;
        git(fixture.root(), &["commit", "-m", "category input"])?;
        let epoch = fixture.epoch()?;
        let mut valid = receipt(&epoch)?;
        assert!(fixture.gate(&epoch, &valid)?.status.success());
        valid
            .get_mut("claims")
            .and_then(Value::as_array_mut)
            .ok_or("claims")?
            .pop();
        assert!(!fixture.gate(&epoch, &valid)?.status.success());
    }
    Ok(())
}
