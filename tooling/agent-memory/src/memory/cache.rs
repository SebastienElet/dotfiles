mod digest;

use self::digest::{oracle_digest, proof_digest, source_fingerprints};
use super::clock::timestamp;
use super::store::inventory::valid_memory_id;
use super::store::types::StorePhase;
use super::{MemoryEntry, MemoryError, Store, UtcTimestamp, parse_utc_timestamp};
use jiff::SignedDuration;
use serde::{Deserialize, Serialize};
use std::io::Read;

const MAX_CACHE_BYTES: u64 = 16 * 1024 * 1024;
const CACHE_LIFETIME: SignedDuration = SignedDuration::from_hours(48);

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct OracleEnvironment {
    os: String,
    arch: String,
}

impl OracleEnvironment {
    pub fn new(os: impl Into<String>, arch: impl Into<String>) -> Self {
        Self {
            os: os.into(),
            arch: arch.into(),
        }
    }

    pub fn current() -> Self {
        Self::new(std::env::consts::OS, std::env::consts::ARCH)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct CacheRecord {
    entry_id: String,
    oracle_digest: String,
    proof_digest: String,
    source_fingerprints: Vec<CacheSourceFingerprint>,
    validated_at: String,
    environment: OracleEnvironment,
    verdict: String,
}

impl CacheRecord {
    pub(crate) fn new(
        entry: &MemoryEntry,
        validated_at: &UtcTimestamp,
        environment: &OracleEnvironment,
    ) -> Result<Self, MemoryError> {
        Ok(Self {
            entry_id: entry.id().as_str().to_owned(),
            oracle_digest: oracle_digest(entry)?,
            proof_digest: proof_digest(entry)?,
            source_fingerprints: source_fingerprints(entry),
            validated_at: validated_at.as_str().to_owned(),
            environment: environment.clone(),
            verdict: "valid".to_owned(),
        })
    }

    pub(crate) fn matches(&self, entry: &MemoryEntry, environment: &OracleEnvironment) -> bool {
        self.entry_id == entry.id().as_str()
            && self.oracle_digest == oracle_digest(entry).unwrap_or_default()
            && self.proof_digest == proof_digest(entry).unwrap_or_default()
            && self.source_fingerprints == source_fingerprints(entry)
            && &self.environment == environment
            && self.verdict == "valid"
    }

    pub(crate) fn usable_at(&self, now: &UtcTimestamp) -> bool {
        let Some(validated_at) = parse_utc_timestamp(&self.validated_at)
            .ok()
            .as_ref()
            .and_then(timestamp)
        else {
            return false;
        };
        let Some(now) = timestamp(now) else {
            return false;
        };
        let age = now.duration_since(validated_at);
        !age.is_negative() && age < CACHE_LIFETIME
    }

    pub(crate) fn validated_at(&self) -> Option<UtcTimestamp> {
        parse_utc_timestamp(&self.validated_at).ok()
    }

    fn valid(&self) -> bool {
        valid_memory_id(&self.entry_id)
            && valid_digest(&self.oracle_digest)
            && valid_digest(&self.proof_digest)
            && !self.source_fingerprints.is_empty()
            && self
                .source_fingerprints
                .iter()
                .all(CacheSourceFingerprint::valid)
            && parse_utc_timestamp(&self.validated_at).is_ok()
            && !self.environment.os.is_empty()
            && !self.environment.arch.is_empty()
            && self.verdict == "valid"
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct CacheSourceFingerprint {
    pub(super) kind: String,
    pub(super) fingerprint: String,
}

impl CacheSourceFingerprint {
    fn valid(&self) -> bool {
        matches!(
            self.kind.as_str(),
            "git-file" | "local-file" | "official-url" | "user-decision"
        ) && valid_digest(&self.fingerprint)
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct CacheDocument {
    schema_version: u8,
    entries: Vec<CacheRecord>,
}

impl CacheDocument {
    fn empty() -> Self {
        Self {
            schema_version: 1,
            entries: Vec::new(),
        }
    }

    fn valid(&self) -> bool {
        self.schema_version == 1
            && self.entries.iter().all(CacheRecord::valid)
            && self
                .entries
                .windows(2)
                .all(|entries| entries[0].entry_id < entries[1].entry_id)
    }

    fn upsert(&mut self, record: CacheRecord) {
        self.entries
            .retain(|entry| entry.entry_id != record.entry_id);
        self.entries.push(record);
        self.entries
            .sort_by(|left, right| left.entry_id.cmp(&right.entry_id));
    }
}

impl Store {
    pub(crate) fn cached_validity(
        &self,
        entry: &MemoryEntry,
        environment: &OracleEnvironment,
        now: &UtcTimestamp,
    ) -> Option<CacheRecord> {
        self.read_cache()
            .ok()
            .flatten()?
            .entries
            .into_iter()
            .find(|record| record.matches(entry, environment) && record.usable_at(now))
    }

    pub(crate) fn persist_validity(&self, record: CacheRecord) -> Result<(), MemoryError> {
        let _lock = self.acquire_lock()?;
        let mut cache = self.read_cache()?.unwrap_or_else(CacheDocument::empty);
        cache.upsert(record);
        let mut bytes = serde_json::to_vec_pretty(&cache).map_err(|_| cache_unavailable())?;
        bytes.push(b'\n');
        let staged = self.publication().stage_cache(&bytes)?;
        self.publication().publish_cache(staged)
    }

    fn read_cache(&self) -> Result<Option<CacheDocument>, MemoryError> {
        let path = self.root().join("oracle-cache.json")?;
        if !path.exists()? {
            return Ok(None);
        }
        let mut file = path.open_read()?;
        let metadata = file.metadata().map_err(|_| cache_unavailable())?;
        if metadata.len() > MAX_CACHE_BYTES {
            return Ok(None);
        }
        let mut bytes = Vec::with_capacity(metadata.len() as usize);
        file.by_ref()
            .take(MAX_CACHE_BYTES + 1)
            .read_to_end(&mut bytes)
            .map_err(|_| cache_unavailable())?;
        self.hit(StorePhase::AfterCacheRead)?;
        path.ensure_same_file(&file)?;
        if bytes.len() as u64 > MAX_CACHE_BYTES {
            return Ok(None);
        }
        let Ok(document) = serde_json::from_slice::<CacheDocument>(&bytes) else {
            return Ok(None);
        };
        Ok(document.valid().then_some(document))
    }
}

fn valid_digest(value: &str) -> bool {
    value.strip_prefix("sha256:").is_some_and(|suffix| {
        suffix.len() == 64
            && suffix
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    })
}

const fn cache_unavailable() -> MemoryError {
    MemoryError::new("cache_unavailable", "cache")
}
