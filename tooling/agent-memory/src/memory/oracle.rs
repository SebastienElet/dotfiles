use super::cache::CacheRecord;
pub use super::cache::OracleEnvironment;
use super::store::inventory::valid_memory_id;
use super::{
    Clock, EntrySource, MemoryEntry, MemoryError, OracleVerdict, SourceKind, Store, UtcTimestamp,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SourceResolution {
    Fingerprint(String),
    Invalid,
    Unavailable,
}

pub trait SourceResolver {
    fn resolve(&self, source: &EntrySource) -> SourceResolution;

    fn work_cutoff_observed_expired(&self) -> bool {
        false
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct ProofValid {
    entry_id: String,
}

impl ProofValid {
    pub fn new(entry_id: impl Into<String>) -> Result<Self, MemoryError> {
        let entry_id = entry_id.into();
        if !valid_memory_id(&entry_id) {
            return Err(MemoryError::new("invalid_memory_id", "id"));
        }
        Ok(Self { entry_id })
    }

    pub(crate) fn entry_id(&self) -> &str {
        &self.entry_id
    }

    fn confirms(&self, entry: &MemoryEntry) -> bool {
        self.entry_id == entry.id().as_str()
    }
}

pub struct OracleContext<'a> {
    store: &'a Store,
    clock: &'a dyn Clock,
    resolver: &'a dyn SourceResolver,
    environment: OracleEnvironment,
    proof_valid: Option<&'a ProofValid>,
}

impl<'a> OracleContext<'a> {
    pub fn new(
        store: &'a Store,
        clock: &'a dyn Clock,
        resolver: &'a dyn SourceResolver,
        environment: OracleEnvironment,
    ) -> Self {
        Self {
            store,
            clock,
            resolver,
            environment,
            proof_valid: None,
        }
    }

    pub fn with_proof_valid(mut self, answer: &'a ProofValid) -> Self {
        self.proof_valid = Some(answer);
        self
    }
}

#[derive(Debug)]
pub struct OracleEvaluation {
    verdict: OracleVerdict,
    validated_at: Option<UtcTimestamp>,
    evaluated_at: UtcTimestamp,
    from_cache: bool,
}

impl OracleEvaluation {
    pub fn verdict(&self) -> OracleVerdict {
        self.verdict
    }

    pub fn validated_at(&self) -> Option<&UtcTimestamp> {
        self.validated_at.as_ref()
    }

    pub(crate) fn evaluated_at(&self) -> &UtcTimestamp {
        &self.evaluated_at
    }

    pub fn from_cache(&self) -> bool {
        self.from_cache
    }
}

pub fn evaluate_oracle(entry: &MemoryEntry, context: OracleContext<'_>) -> OracleEvaluation {
    let now = context.clock.now();
    if let Some(cached) = context
        .store
        .cached_validity(entry, &context.environment, &now)
    {
        match evaluate_sources(entry, context.resolver, true) {
            SourceVerdict::Valid => {
                return evaluation(OracleVerdict::Valid, cached.validated_at(), now, true);
            }
            SourceVerdict::Invalid => return transient(OracleVerdict::Invalid, now),
            SourceVerdict::Unavailable => {
                return fallback(entry, &context, now, OracleVerdict::Unavailable);
            }
        }
    }
    if !entry.oracle().has_automated_oracle() {
        return fallback(entry, &context, now, OracleVerdict::NeedsConfirmation);
    }
    match evaluate_sources(entry, context.resolver, false) {
        SourceVerdict::Valid => valid(entry, &context, now),
        SourceVerdict::Invalid => transient(OracleVerdict::Invalid, now),
        SourceVerdict::Unavailable => fallback(entry, &context, now, OracleVerdict::Unavailable),
    }
}

fn fallback(
    entry: &MemoryEntry,
    context: &OracleContext<'_>,
    now: UtcTimestamp,
    without_answer: OracleVerdict,
) -> OracleEvaluation {
    if context
        .proof_valid
        .is_some_and(|answer| answer.confirms(entry))
    {
        valid(entry, context, now)
    } else {
        transient(without_answer, now)
    }
}

fn valid(entry: &MemoryEntry, context: &OracleContext<'_>, now: UtcTimestamp) -> OracleEvaluation {
    if let Ok(record) = CacheRecord::new(entry, &now, &context.environment) {
        let _ = context.store.persist_validity(record);
    }
    evaluation(OracleVerdict::Valid, Some(now.clone()), now, false)
}

fn transient(verdict: OracleVerdict, evaluated_at: UtcTimestamp) -> OracleEvaluation {
    evaluation(verdict, None, evaluated_at, false)
}

fn evaluation(
    verdict: OracleVerdict,
    validated_at: Option<UtcTimestamp>,
    evaluated_at: UtcTimestamp,
    from_cache: bool,
) -> OracleEvaluation {
    OracleEvaluation {
        verdict,
        validated_at,
        evaluated_at,
        from_cache,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum SourceVerdict {
    Valid,
    Invalid,
    Unavailable,
}

fn evaluate_sources(
    entry: &MemoryEntry,
    resolver: &dyn SourceResolver,
    local_only: bool,
) -> SourceVerdict {
    let mut unavailable = false;
    for source in entry.proof().sources() {
        if local_only && !matches!(source.kind(), SourceKind::GitFile | SourceKind::LocalFile) {
            continue;
        }
        if resolver.work_cutoff_observed_expired() {
            return SourceVerdict::Unavailable;
        }
        match resolver.resolve(source) {
            SourceResolution::Fingerprint(actual) if actual == source.fingerprint().as_str() => {}
            SourceResolution::Fingerprint(_) | SourceResolution::Invalid => {
                return SourceVerdict::Invalid;
            }
            SourceResolution::Unavailable => unavailable = true,
        }
    }
    if unavailable {
        SourceVerdict::Unavailable
    } else {
        SourceVerdict::Valid
    }
}
