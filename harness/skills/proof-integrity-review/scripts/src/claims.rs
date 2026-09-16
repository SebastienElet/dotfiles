use crate::{
    Result,
    model::{Claim, Epoch, Receipt},
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
    let matched: BTreeSet<_> = epoch
        .classifier
        .matched_paths
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
        ]
        .iter()
        .any(|s| s.trim().is_empty())
            || !matches!(claim.impact.as_str(), "high" | "low")
            || !all.insert(&claim.id)
        {
            return Err("invalid or duplicate claim".into());
        }
        let paths: BTreeSet<_> = claim.paths.iter().map(String::as_str).collect();
        if paths.is_empty() || paths.len() != claim.paths.len() || !paths.is_subset(&matched) {
            return Err("claim paths must uniquely reference the triggered surface".into());
        }
        if claim.impact == "high" {
            high.insert(claim.id.as_str(), claim);
            if !reserved(&claim.id) {
                if paths.len() != 1 || claim.id == "material.changed_surface" {
                    return Err("material claim must identify one concrete path".into());
                }
                covered.extend(paths);
            }
        }
    }
    if covered != matched {
        return Err("triggered paths lack concrete high-impact claims".into());
    }
    for required in BASE {
        if !high.contains_key(required) {
            return Err(format!("missing required claim: {required}").into());
        }
    }
    for (category, required) in [
        ("authority_contracts", AUTHORITY.as_slice()),
        ("test_oracles", ORACLES.as_slice()),
    ] {
        if let Some(paths) = epoch.classifier.categories.get(category) {
            for id in required {
                let claim = high
                    .get(id)
                    .ok_or_else(|| format!("missing category claim: {id}"))?;
                if paths.iter().any(|path| !claim.paths.contains(path)) {
                    return Err("category claim misses affected paths".into());
                }
            }
        }
    }
    Ok(high)
}
