use crate::{
    Result, claims, content,
    model::{Capability, Epoch, PREDICATE, Receipt, VERSION, Witness},
    repository, targets,
};
use serde_json::{Value, json};
use std::{collections::BTreeSet, path::Path};

pub fn gate(epoch: &Epoch, receipt: &Receipt, policy_root: &Path) -> Result<Value> {
    repository::current(epoch, policy_root)?;
    if receipt.schema_version != VERSION
        || receipt.predicate_type != PREDICATE
        || receipt.subject_digest != epoch.subject_digest
        || receipt.policy_digest != epoch.policy_digest
        || receipt.verdict != "PROOF_ADEQUATE"
    {
        return Err("receipt schema, verdict, subject or policy mismatch".into());
    }
    let auditor = &receipt.auditor;
    if auditor.host.trim().is_empty()
        || auditor.session_id.trim().is_empty()
        || auditor.sandbox_mode.trim().is_empty()
    {
        return Err("auditor independence declarations required".into());
    }
    if let Some(isolation) = &auditor.isolation
        && (isolation.requirement.trim().is_empty()
            || isolation.enforced != Capability::Known(true)
            || isolation.evidence.trim().is_empty())
    {
        return Err("explicit security obligation requires evidenced technical isolation".into());
    }
    let high = claims::claims(epoch, receipt)?;
    let policy = content::policy(policy_root)?;
    let mut witnessed = BTreeSet::new();
    for witness in &receipt.witnesses {
        if !high.contains_key(witness.claim_id.as_str())
            || !witnessed.insert(witness.claim_id.as_str())
        {
            return Err("unknown or duplicate witness claim".into());
        }
        if witness.mutant.description.trim().is_empty()
            || content::bound(&witness.mutant)? != witness.mutant_digest
            || content::bound(&witness.evidence)? != witness.artifact_digest
        {
            return Err("mutant or evidence digest mismatch".into());
        }
        let scopes = targets::targets(epoch, policy_root, &policy, &witness.mutant.targets)?;
        let fixture_only = scopes.len() == 1 && scopes.contains("fixture");
        if fixture_only
            && (!claims::reserved(&witness.claim_id)
                || witness.claim_id == "test_oracle.defect_detection")
        {
            return Err("substantive claim requires repository or policy mutation".into());
        }
        if scopes.contains("fixture") && !fixture_only {
            return Err("fixture and file targets cannot be mixed".into());
        }
        let mode = if fixture_only {
            "input-replacement"
        } else {
            "file-overlay"
        };
        if witness.evidence.injection_mode != mode {
            return Err("injection mode does not match target scopes".into());
        }
        evidence(witness)?;
    }
    if witnessed != high.keys().copied().collect() {
        return Err("modified oracles or critical claims lack witnesses".into());
    }
    Ok(
        json!({"schema_version":VERSION,"decision":"ALLOW","reason":"PROOF_ADEQUATE","subject_digest":epoch.subject_digest,"policy_digest":epoch.policy_digest}),
    )
}
fn evidence(witness: &Witness) -> Result<()> {
    let evidence = &witness.evidence;
    claims::observation(&evidence.provenance)?;
    if evidence.targets_digest != content::bound(&witness.mutant.targets)? {
        return Err("evidence targets digest mismatch".into());
    }
    let diagnostic = &evidence.expected_failure;
    if evidence.command.trim().is_empty()
        || diagnostic.trim() != diagnostic
        || diagnostic.chars().count() < 8
        || diagnostic.chars().any(char::is_control)
        || diagnostic.contains("PROOF_")
    {
        return Err("command and normalized expected failure required".into());
    }
    let red = [&evidence.red_stdout, &evidence.red_stderr];
    if !red
        .iter()
        .any(|stream| stream.lines().any(|line| line == diagnostic))
    {
        return Err("evidence lacks exact diagnostic line".into());
    }
    if witness.red_exit_code == 0
        || witness.green_exit_code != 0
        || evidence.red_exit_code != witness.red_exit_code
        || evidence.green_exit_code != witness.green_exit_code
    {
        return Err("witness must declare matching nonzero red and zero green exit codes".into());
    }
    Ok(())
}
