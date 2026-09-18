use crate::{
    Result, claims, content,
    model::{Epoch, PREDICATE, Receipt, VERSION, Witness},
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
        || auditor.sandbox_mode != "read-only"
    {
        return Err("auditor independence declarations required".into());
    }
    if !epoch.classifier.triggered {
        return Ok(json!({"schema_version":VERSION,"decision":"ALLOW","reason":"NOT_APPLICABLE"}));
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
        evidence(epoch, witness)?;
    }
    if witnessed != high.keys().copied().collect() {
        return Err("high-impact claims lack witnesses".into());
    }
    Ok(
        json!({"schema_version":VERSION,"decision":"ALLOW","reason":"PROOF_ADEQUATE","subject_digest":epoch.subject_digest,"policy_digest":epoch.policy_digest}),
    )
}
fn evidence(epoch: &Epoch, witness: &Witness) -> Result<()> {
    let evidence = &witness.evidence;
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
    let green = [&evidence.green_stdout, &evidence.green_stderr];
    let red_marker = format!("PROOF_RED:{}:{}", witness.claim_id, witness.mutant_digest);
    let green_marker = format!("PROOF_GREEN:{}:{}", witness.claim_id, epoch.subject_digest);
    if !red
        .iter()
        .any(|stream| stream.lines().any(|line| line == diagnostic))
        || !red
            .iter()
            .any(|stream| stream.lines().any(|line| line == red_marker))
        || !green
            .iter()
            .any(|stream| stream.lines().any(|line| line == green_marker))
    {
        return Err("evidence lacks exact diagnostic or bound marker lines".into());
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
