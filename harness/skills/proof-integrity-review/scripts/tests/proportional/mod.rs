use crate::support::{Fixture, Result, bound, receipt, set};
use serde_json::json;

#[test]
fn accepts_write_capability_and_unknown_memory_on_stable_candidate() -> Result {
    let fixture = Fixture::new()?;
    let epoch = fixture.epoch()?;
    let mut valid = receipt(&epoch)?;
    set(&mut valid, "/auditor/write_tools_enabled", json!(true))?;
    set(&mut valid, "/auditor/persistent_memory", json!(null))?;
    set(
        &mut valid,
        "/auditor/sandbox_mode",
        json!("workspace-write"),
    )?;
    let result = fixture.gate(&epoch, &valid)?;
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    std::fs::write(fixture.root().join("Makefile"), "changed during audit")?;
    assert!(!fixture.gate(&epoch, &valid)?.status.success());
    Ok(())
}

#[test]
fn accepts_low_impact_surface_without_artificial_mutations() -> Result {
    let fixture = Fixture::new()?;
    let epoch = fixture.epoch()?;
    let mut valid = receipt(&epoch)?;
    let mut claim = valid.pointer("/claims/3").ok_or("claim")?.clone();
    set(&mut claim, "/impact", json!("low"))?;
    set(&mut valid, "/claims", json!([claim]))?;
    set(&mut valid, "/witnesses", json!([]))?;
    let result = fixture.gate(&epoch, &valid)?;
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    Ok(())
}

#[test]
fn explicit_security_obligation_requires_technical_isolation() -> Result {
    let fixture = Fixture::new()?;
    let epoch = fixture.epoch()?;
    let mut valid = receipt(&epoch)?;
    for enforced in [json!(false), json!(null), json!(true)] {
        set(
            &mut valid,
            "/auditor/isolation",
            json!({
                "requirement": "security policy section 2 prohibits candidate writes",
                "enforced": enforced, "evidence": "host sandbox attestation"
            }),
        )?;
        assert_eq!(
            fixture.gate(&epoch, &valid)?.status.success(),
            enforced == true
        );
    }
    set(&mut valid, "/auditor/isolation/evidence", json!(""))?;
    assert!(!fixture.gate(&epoch, &valid)?.status.success());
    Ok(())
}

#[test]
fn low_impact_cannot_waive_modified_oracle_or_critical_witness() -> Result {
    let fixture = Fixture::new()?;
    let epoch = fixture.epoch()?;
    for kind in ["modified-oracle", "critical-guarantee"] {
        let mut valid = receipt(&epoch)?;
        set(&mut valid, "/claims/3/impact", json!("low"))?;
        set(&mut valid, "/claims/3/kind", json!(kind))?;
        assert!(fixture.gate(&epoch, &valid)?.status.success());
        valid
            .get_mut("witnesses")
            .and_then(serde_json::Value::as_array_mut)
            .ok_or("witnesses")?
            .pop();
        assert!(!fixture.gate(&epoch, &valid)?.status.success());
    }
    Ok(())
}

#[test]
fn accepts_traceable_ci_and_retained_observations_but_refuses_missing_basis() -> Result {
    let fixture = Fixture::new()?;
    let epoch = fixture.epoch()?;
    for origin in ["ci", "retained", "author"] {
        let mut valid = receipt(&epoch)?;
        set(
            &mut valid,
            "/claims/3/positive_evidence/origin",
            json!(origin),
        )?;
        assert!(fixture.gate(&epoch, &valid)?.status.success());
        for field in ["artifact", "input_basis", "environment"] {
            let mut missing = valid.clone();
            set(
                &mut missing,
                &format!("/claims/3/positive_evidence/{field}"),
                json!(""),
            )?;
            assert!(!fixture.gate(&epoch, &missing)?.status.success());
        }
    }
    Ok(())
}

#[test]
fn unknown_capabilities_do_not_assert_known_values_or_independence() -> Result {
    let fixture = Fixture::new()?;
    let epoch = fixture.epoch()?;
    let mut valid = receipt(&epoch)?;
    set(&mut valid, "/auditor/write_tools_enabled", json!(null))?;
    set(&mut valid, "/auditor/persistent_memory", json!(null))?;
    assert!(fixture.gate(&epoch, &valid)?.status.success());
    for field in [
        "fresh_session",
        "author_independent",
        "independent_first_pass",
    ] {
        let mut unknown = valid.clone();
        set(&mut unknown, &format!("/auditor/{field}"), json!(null))?;
        assert!(!fixture.gate(&epoch, &unknown)?.status.success());
    }
    Ok(())
}

#[test]
fn accepts_retained_raw_witness_logs_without_current_receipt_markers() -> Result {
    let fixture = Fixture::new()?;
    let epoch = fixture.epoch()?;
    let mut valid = receipt(&epoch)?;
    let witness = valid.pointer_mut("/witnesses/3").ok_or("witness")?;
    set(witness, "/evidence/provenance/origin", json!("retained"))?;
    set(
        witness,
        "/evidence/red_stdout",
        json!("invalid input rejected\n"),
    )?;
    set(witness, "/evidence/green_stdout", json!("1 test passed\n"))?;
    let digest = bound(witness.get("evidence").ok_or("evidence")?)?;
    set(witness, "/artifact_digest", json!(digest))?;
    let result = fixture.gate(&epoch, &valid)?;
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    Ok(())
}
