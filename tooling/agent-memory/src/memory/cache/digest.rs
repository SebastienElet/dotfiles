use super::CacheSourceFingerprint;
use crate::memory::store::canonicalization::source_name;
use crate::memory::{MemoryEntry, MemoryError};
use serde::Serialize;
use sha2::{Digest, Sha256};

pub(super) fn source_fingerprints(entry: &MemoryEntry) -> Vec<CacheSourceFingerprint> {
    entry
        .proof()
        .sources()
        .iter()
        .map(|source| CacheSourceFingerprint {
            kind: source_name(source.kind()).to_owned(),
            fingerprint: source.fingerprint().as_str().to_owned(),
        })
        .collect()
}

pub(super) fn oracle_digest(entry: &MemoryEntry) -> Result<String, MemoryError> {
    #[derive(Serialize)]
    struct AutomatedOracle<'a> {
        kind: &'a str,
        expected: &'a str,
    }
    #[derive(Serialize)]
    struct HumanFallback<'a> {
        question: &'a str,
        valid_when: &'a str,
    }
    #[derive(Serialize)]
    struct Outcomes<'a> {
        valid: &'a str,
        invalidated: &'a str,
    }
    #[derive(Serialize)]
    struct DeclarativeOracle<'a> {
        automated: Option<AutomatedOracle<'a>>,
        human_fallback: HumanFallback<'a>,
        outcomes: Outcomes<'a>,
    }
    let oracle = entry.oracle();
    digest(&DeclarativeOracle {
        automated: oracle.has_automated_oracle().then_some(AutomatedOracle {
            kind: "source-fingerprint",
            expected: "all-proof-sources-unchanged",
        }),
        human_fallback: HumanFallback {
            question: oracle.fallback_question(),
            valid_when: oracle.fallback_valid_when(),
        },
        outcomes: Outcomes {
            valid: oracle.valid_outcome(),
            invalidated: oracle.invalidated_outcome(),
        },
    })
}

pub(super) fn proof_digest(entry: &MemoryEntry) -> Result<String, MemoryError> {
    #[derive(Serialize)]
    struct ProofSource<'a> {
        kind: &'a str,
        locator: &'a str,
        fingerprint: &'a str,
    }
    let sources = entry
        .proof()
        .sources()
        .iter()
        .map(|source| ProofSource {
            kind: source_name(source.kind()),
            locator: source.locator(),
            fingerprint: source.fingerprint().as_str(),
        })
        .collect::<Vec<_>>();
    digest(&sources)
}

fn digest(value: &impl Serialize) -> Result<String, MemoryError> {
    let bytes = serde_json::to_vec(value).map_err(|_| super::cache_unavailable())?;
    Ok(format!("sha256:{:x}", Sha256::digest(bytes)))
}
