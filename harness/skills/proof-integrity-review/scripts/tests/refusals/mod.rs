use crate::support::{Fixture, Result, bound, cli, digest, git, rebind, receipt, set};
use serde_json::{Value, json};
use std::fs;

#[test]
fn refuses_rebound_mutations_with_unsafe_paths_or_inadequate_sources() -> Result {
    let fixture = Fixture::new()?;
    let epoch = fixture.epoch()?;
    let valid = receipt(&epoch)?;
    for path in [
        "../Makefile",
        "./Makefile",
        "/Makefile",
        "a/../Makefile",
        "Makefile/",
        "a//b",
        "C:/Makefile",
        "a\\b",
        "Makefile\n",
    ] {
        let mut changed = valid.clone();
        set(
            &mut changed,
            "/witnesses/3/mutant/targets/0/path",
            json!(path),
        )?;
        rebind(&mut changed)?;
        assert!(
            !fixture.gate(&epoch, &changed)?.status.success(),
            "accepted path {path}"
        );
    }
    for (pointer, value) in [
        ("/witnesses/3/mutant/targets/0/scope", json!("fixture")),
        ("/witnesses/3/mutant/targets/0/scope", json!("unknown")),
        ("/witnesses/3/mutant/targets/0/path", json!("missing")),
        (
            "/witnesses/3/mutant/targets/0/before_digest",
            json!(digest(b"bad")),
        ),
        ("/witnesses/3/mutant/targets/0/after_content", json!("new")),
        ("/witnesses/3/mutant/targets", json!([])),
    ] {
        let mut changed = valid.clone();
        set(&mut changed, pointer, value)?;
        rebind(&mut changed)?;
        assert!(
            !fixture.gate(&epoch, &changed)?.status.success(),
            "accepted {pointer}"
        );
    }
    Ok(())
}
#[test]
fn refuses_rebound_evidence_without_exact_diagnostic_or_provenance() -> Result {
    let fixture = Fixture::new()?;
    let epoch = fixture.epoch()?;
    let valid = receipt(&epoch)?;
    for (pointer, value) in [
        ("/command", json!("")),
        ("/expected_failure", json!("PROOF_BAD")),
        ("/expected_failure", json!("missing diagnostic")),
        ("/expected_failure", json!("too\nshort")),
        ("/red_stdout", json!("unexpected failure")),
        (
            "/red_stdout",
            json!(format!(
                "prefix{}",
                valid
                    .pointer("/witnesses/0/evidence/red_stdout")
                    .and_then(Value::as_str)
                    .ok_or("output")?
            )),
        ),
        ("/provenance/input_basis", json!("")),
        ("/provenance/artifact", json!("")),
        ("/provenance/environment", json!("")),
        ("/provenance/origin", json!("invented")),
        ("/green_exit_code", json!(1)),
        ("/red_exit_code", json!(0)),
        ("/targets_digest", json!(digest(b"other"))),
        ("/injection_mode", json!("input-replacement")),
    ] {
        let mut changed = valid.clone();
        let witness = changed.pointer_mut("/witnesses/0").ok_or("witness")?;
        set(witness, &format!("/evidence{pointer}"), value)?;
        let digest = bound(witness.get("evidence").ok_or("evidence")?)?;
        set(witness, "/artifact_digest", json!(digest))?;
        assert!(
            !fixture.gate(&epoch, &changed)?.status.success(),
            "accepted {pointer}"
        );
    }
    Ok(())
}
#[test]
fn refuses_duplicate_and_oversized_targets() -> Result {
    let fixture = Fixture::new()?;
    let epoch = fixture.epoch()?;
    let valid = receipt(&epoch)?;
    for count in [2, 65] {
        let mut changed = valid.clone();
        let target = changed
            .pointer("/witnesses/0/mutant/targets/0")
            .ok_or("target")?
            .clone();
        set(
            &mut changed,
            "/witnesses/0/mutant/targets",
            json!(vec![target; count]),
        )?;
        rebind(&mut changed)?;
        assert!(!fixture.gate(&epoch, &changed)?.status.success());
    }
    let mut changed = valid;
    let after = "x".repeat(8 * 1024 * 1024);
    set(
        &mut changed,
        "/witnesses/0/mutant/targets/0/after_digest",
        json!(digest(after.as_bytes())),
    )?;
    set(
        &mut changed,
        "/witnesses/0/mutant/targets/0/after_content",
        json!(after),
    )?;
    rebind(&mut changed)?;
    assert!(!fixture.gate(&epoch, &changed)?.status.success());
    Ok(())
}
#[test]
fn freezes_index_contents_even_when_porcelain_status_is_unchanged() -> Result {
    let fixture = Fixture::new()?;
    fs::write(fixture.root().join("Makefile"), "staged first")?;
    git(fixture.root(), &["add", "Makefile"])?;
    fs::write(fixture.root().join("Makefile"), "live")?;
    let epoch = fixture.epoch()?;
    let before = epoch.get("subject_digest").ok_or("digest")?;
    fs::write(fixture.root().join("Makefile"), "staged second")?;
    git(fixture.root(), &["add", "Makefile"])?;
    fs::write(fixture.root().join("Makefile"), "live")?;
    let after = fixture.epoch()?;
    assert_ne!(before, after.get("subject_digest").ok_or("digest")?);
    Ok(())
}
#[test]
fn refuses_duplicate_json_keys_even_inside_category_maps() -> Result {
    let fixture = Fixture::new()?;
    let epoch = fixture.epoch()?;
    let valid = receipt(&epoch)?;
    assert!(fixture.gate(&epoch, &valid)?.status.success());
    let epoch_path = fixture.root().join(".proof-integrity/epoch.json");
    let text = serde_json::to_string(&epoch)?.replace(
        "\"verification_routing\":[\"Makefile\"]",
        "\"verification_routing\":[\"Makefile\"],\"verification_routing\":[\"Makefile\"]",
    );
    fs::write(&epoch_path, text)?;
    let output = cli(&[
        "gate",
        "--epoch",
        epoch_path.to_str().ok_or("path")?,
        "--receipt",
        fixture
            .root()
            .join(".proof-integrity/receipt.json")
            .to_str()
            .ok_or("path")?,
        "--policy-root",
        fixture.root().join("policy").to_str().ok_or("path")?,
    ])?;
    assert!(!output.status.success());
    Ok(())
}
