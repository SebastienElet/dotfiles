use crate::{
    Result,
    model::{Claim, Epoch, Observation, Receipt},
};
use std::collections::{BTreeMap, BTreeSet};

pub const BASE: [&str; 3] = [
    "proof.trigger_coverage",
    "proof.epoch_binding",
    "proof.fail_closed",
];
pub const AUTHORITY: [&str; 4] = [
    "authority.source_resolution",
    "authority.allowed_values",
    "authority.enforcement_coverage",
    "authority.citation_fidelity",
];
pub const ORACLES: [&str; 2] = [
    "test_oracle.invocation_coverage",
    "test_oracle.defect_detection",
];
pub fn reserved(id: &str) -> bool {
    BASE.contains(&id) || AUTHORITY.contains(&id) || ORACLES.contains(&id)
}
pub fn claims<'a>(epoch: &Epoch, receipt: &'a Receipt) -> Result<BTreeMap<&'a str, &'a Claim>> {
    let changed: BTreeSet<_> = epoch
        .subject
        .changed_paths
        .iter()
        .map(String::as_str)
        .collect();
    let mut all = BTreeSet::new();
    let mut high = BTreeMap::new();
    let mut covered = BTreeSet::new();
    for claim in &receipt.claims {
        if [
            &claim.id,
            &claim.statement,
            &claim.source,
            &claim.enforcement,
            &claim.oracle,
            &claim.rationale,
        ]
        .iter()
        .any(|s| s.trim().is_empty())
            || !matches!(claim.impact.as_str(), "high" | "low")
            || !matches!(
                claim.kind.as_str(),
                "modified-oracle" | "critical-guarantee" | "other"
            )
            || !all.insert(&claim.id)
        {
            return Err("invalid or duplicate claim".into());
        }
        observation(&claim.positive_evidence)?;
        let paths: BTreeSet<_> = claim.paths.iter().map(String::as_str).collect();
        if paths.is_empty() || paths.len() != claim.paths.len() || !paths.is_subset(&changed) {
            return Err("claim paths must uniquely reference the changed surface".into());
        }
        covered.extend(paths);
        if claim.impact == "high" || claim.kind != "other" {
            high.insert(claim.id.as_str(), claim);
        }
    }
    if covered != changed || receipt.claims.is_empty() {
        return Err("changed paths lack semantic assessment".into());
    }
    Ok(high)
}

pub fn observation(evidence: &Observation) -> Result<()> {
    if [
        &evidence.artifact,
        &evidence.input_basis,
        &evidence.environment,
    ]
    .iter()
    .any(|value| value.trim().is_empty())
        || !matches!(
            evidence.origin.as_str(),
            "reviewer" | "ci" | "author" | "retained"
        )
    {
        return Err(
            "traceable evidence origin, artifact, input basis and environment required".into(),
        );
    }
    Ok(())
}
