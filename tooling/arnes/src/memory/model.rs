use super::MemoryError;
use serde::Deserialize;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum MemoryKind {
    Goal,
    Decision,
    Evidence,
    Invariant,
    Unknown,
    Assumption,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    Active,
    Achieved,
    Abandoned,
    Superseded,
    Invalidated,
    Resolved,
    Confirmed,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TransitionVerdict {
    Valid,
    Invalid,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum SourceKind {
    GitFile,
    LocalFile,
    OfficialUrl,
    UserDecision,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ScopeDraft {
    Project,
    User,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AdmissionAuthorization {
    ExplicitRequest,
    AcceptedProposal,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OracleVerdict {
    Valid,
    Invalid,
    Unavailable,
    NeedsConfirmation,
}

#[derive(Debug)]
pub enum AdmissionResult {
    Stored {
        id: MemoryId,
        index_rebuild_required: bool,
    },
    Duplicate {
        id: MemoryId,
    },
    Rejected {
        error: MemoryError,
    },
    Conflict {
        id: MemoryId,
        error: MemoryError,
    },
}

macro_rules! validated_text {
    ($name:ident) => {
        #[derive(Clone, Debug, Eq, PartialEq)]
        pub struct $name(String);

        impl $name {
            pub(crate) fn from_validated(value: String) -> Self {
                Self(value)
            }

            pub fn as_str(&self) -> &str {
                &self.0
            }
        }
    };
}

validated_text!(MemoryId);
validated_text!(ProjectKey);
validated_text!(Statement);
validated_text!(RetrievalTerm);
validated_text!(Fingerprint);
validated_text!(UtcTimestamp);

#[derive(Debug, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
pub(crate) enum RawDraftKind {
    Goal {
        #[serde(flatten)]
        data: RawDraftData,
    },
    Decision {
        #[serde(flatten)]
        data: RawDraftData,
    },
    Evidence {
        #[serde(flatten)]
        data: RawDraftData,
    },
    Invariant {
        #[serde(flatten)]
        data: RawDraftData,
    },
    Unknown {
        #[serde(flatten)]
        data: RawDraftData,
    },
    Assumption {
        #[serde(flatten)]
        data: RawDraftData,
    },
}

impl RawDraftKind {
    pub(crate) fn split(self) -> (MemoryKind, RawDraftData) {
        match self {
            Self::Goal { data } => (MemoryKind::Goal, data),
            Self::Decision { data } => (MemoryKind::Decision, data),
            Self::Evidence { data } => (MemoryKind::Evidence, data),
            Self::Invariant { data } => (MemoryKind::Invariant, data),
            Self::Unknown { data } => (MemoryKind::Unknown, data),
            Self::Assumption { data } => (MemoryKind::Assumption, data),
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RawDraftData {
    pub(crate) schema_version: u64,
    pub(crate) statement: String,
    pub(crate) scope: ScopeDraft,
    pub(crate) retrieval_terms: Vec<String>,
    pub(crate) proof: RawDraftProof,
    pub(crate) oracle: RawOracle,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RawDraftProof {
    pub(crate) summary: String,
    pub(crate) sources: Vec<RawDraftSource>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
pub(crate) enum RawDraftSource {
    GitFile { locator: String },
    LocalFile { locator: String },
    OfficialUrl { locator: String },
    UserDecision { locator: String },
}

impl RawDraftSource {
    pub(crate) fn split(self) -> (SourceKind, String) {
        match self {
            Self::GitFile { locator } => (SourceKind::GitFile, locator),
            Self::LocalFile { locator } => (SourceKind::LocalFile, locator),
            Self::OfficialUrl { locator } => (SourceKind::OfficialUrl, locator),
            Self::UserDecision { locator } => (SourceKind::UserDecision, locator),
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RawOracle {
    pub(crate) automated: Option<RawAutomatedOracle>,
    pub(crate) human_fallback: RawHumanFallback,
    pub(crate) outcomes: RawOutcomes,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
pub(crate) enum RawAutomatedOracle {
    SourceFingerprint {
        #[serde(rename = "expected")]
        _expected: FingerprintExpectation,
    },
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum FingerprintExpectation {
    AllProofSourcesUnchanged,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RawHumanFallback {
    pub(crate) question: String,
    pub(crate) valid_when: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RawOutcomes {
    pub(crate) valid: String,
    pub(crate) invalidated: String,
}

#[derive(Debug)]
pub struct AdmissionDraft {
    pub(crate) kind: MemoryKind,
    pub(crate) data: RawDraftData,
}

impl AdmissionDraft {
    pub fn kind(&self) -> MemoryKind {
        self.kind
    }
}

#[derive(Debug)]
pub struct ValidatedDraft {
    kind: MemoryKind,
    statement: Statement,
    scope: ScopeDraft,
    retrieval_terms: Vec<RetrievalTerm>,
    proof: ValidatedDraftProof,
    oracle: ValidatedOracle,
}

impl ValidatedDraft {
    pub(crate) fn new(
        kind: MemoryKind,
        statement: Statement,
        scope: ScopeDraft,
        retrieval_terms: Vec<RetrievalTerm>,
        proof: ValidatedDraftProof,
        oracle: ValidatedOracle,
    ) -> Self {
        Self {
            kind,
            statement,
            scope,
            retrieval_terms,
            proof,
            oracle,
        }
    }

    pub fn kind(&self) -> MemoryKind {
        self.kind
    }

    pub fn statement(&self) -> &Statement {
        &self.statement
    }

    pub fn scope(&self) -> ScopeDraft {
        self.scope
    }

    pub fn retrieval_terms(&self) -> &[RetrievalTerm] {
        &self.retrieval_terms
    }

    pub fn proof(&self) -> &ValidatedDraftProof {
        &self.proof
    }

    pub fn oracle(&self) -> &ValidatedOracle {
        &self.oracle
    }
}

#[derive(Debug)]
pub struct ValidatedDraftProof {
    summary: String,
    sources: Vec<ValidatedDraftSource>,
}

impl ValidatedDraftProof {
    pub(crate) fn new(summary: String, sources: Vec<ValidatedDraftSource>) -> Self {
        Self { summary, sources }
    }

    pub fn summary(&self) -> &str {
        &self.summary
    }

    pub fn sources(&self) -> &[ValidatedDraftSource] {
        &self.sources
    }
}

#[derive(Debug)]
pub struct ValidatedDraftSource {
    kind: SourceKind,
    locator: String,
}

impl ValidatedDraftSource {
    pub(crate) fn new(kind: SourceKind, locator: String) -> Self {
        Self { kind, locator }
    }

    pub fn kind(&self) -> SourceKind {
        self.kind
    }

    pub fn locator(&self) -> &str {
        &self.locator
    }
}

#[derive(Debug)]
pub struct ValidatedOracle {
    automated: Option<RawAutomatedOracle>,
    human_fallback: RawHumanFallback,
    outcomes: RawOutcomes,
}

impl ValidatedOracle {
    pub(crate) fn new(
        automated: Option<RawAutomatedOracle>,
        human_fallback: RawHumanFallback,
        outcomes: RawOutcomes,
    ) -> Self {
        Self {
            automated,
            human_fallback,
            outcomes,
        }
    }

    pub fn has_automated_oracle(&self) -> bool {
        self.automated.is_some()
    }

    pub fn fallback_question(&self) -> &str {
        &self.human_fallback.question
    }

    pub fn fallback_valid_when(&self) -> &str {
        &self.human_fallback.valid_when
    }

    pub fn valid_outcome(&self) -> &str {
        &self.outcomes.valid
    }

    pub fn invalidated_outcome(&self) -> &str {
        &self.outcomes.invalidated
    }
}

#[derive(Debug, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
pub(crate) enum RawEntryKind {
    Goal {
        #[serde(flatten)]
        data: RawEntryData,
    },
    Decision {
        #[serde(flatten)]
        data: RawEntryData,
    },
    Evidence {
        #[serde(flatten)]
        data: RawEntryData,
    },
    Invariant {
        #[serde(flatten)]
        data: RawEntryData,
    },
    Unknown {
        #[serde(flatten)]
        data: RawEntryData,
    },
    Assumption {
        #[serde(flatten)]
        data: RawEntryData,
    },
}

impl RawEntryKind {
    pub(crate) fn split(self) -> (MemoryKind, RawEntryData) {
        match self {
            Self::Goal { data } => (MemoryKind::Goal, data),
            Self::Decision { data } => (MemoryKind::Decision, data),
            Self::Evidence { data } => (MemoryKind::Evidence, data),
            Self::Invariant { data } => (MemoryKind::Invariant, data),
            Self::Unknown { data } => (MemoryKind::Unknown, data),
            Self::Assumption { data } => (MemoryKind::Assumption, data),
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RawEntryData {
    pub(crate) schema_version: u64,
    pub(crate) id: String,
    pub(crate) status: Status,
    pub(crate) statement: String,
    pub(crate) scope: RawEntryScope,
    pub(crate) retrieval_terms: Vec<String>,
    pub(crate) proof: RawEntryProof,
    pub(crate) oracle: RawOracle,
    pub(crate) created_at: String,
    pub(crate) transition: Option<RawTransition>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase", deny_unknown_fields)]
pub(crate) enum RawEntryScope {
    Project { key: String },
    User,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RawEntryProof {
    pub(crate) summary: String,
    pub(crate) sources: Vec<RawEntrySource>,
    pub(crate) established_at: String,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
pub(crate) enum RawEntrySource {
    GitFile {
        locator: String,
        fingerprint: String,
    },
    LocalFile {
        locator: String,
        fingerprint: String,
    },
    OfficialUrl {
        locator: String,
        fingerprint: String,
    },
    UserDecision {
        locator: String,
        fingerprint: String,
    },
}

impl RawEntrySource {
    pub(crate) fn split(self) -> (SourceKind, String, String) {
        match self {
            Self::GitFile {
                locator,
                fingerprint,
            } => (SourceKind::GitFile, locator, fingerprint),
            Self::LocalFile {
                locator,
                fingerprint,
            } => (SourceKind::LocalFile, locator, fingerprint),
            Self::OfficialUrl {
                locator,
                fingerprint,
            } => (SourceKind::OfficialUrl, locator, fingerprint),
            Self::UserDecision {
                locator,
                fingerprint,
            } => (SourceKind::UserDecision, locator, fingerprint),
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RawTransition {
    pub(crate) from: Status,
    pub(crate) to: Status,
    pub(crate) at: String,
    pub(crate) verdict: TransitionVerdict,
    pub(crate) reason: String,
}

#[derive(Debug)]
pub struct MemoryEntry {
    kind: MemoryKind,
    data: EntryData,
}

impl MemoryEntry {
    pub(crate) fn new(kind: MemoryKind, data: EntryData) -> Self {
        Self { kind, data }
    }

    pub fn kind(&self) -> MemoryKind {
        self.kind
    }

    pub fn id(&self) -> &MemoryId {
        &self.data.id
    }

    pub fn status(&self) -> Status {
        self.data.status
    }

    pub fn statement(&self) -> &Statement {
        &self.data.statement
    }

    pub fn retrieval_terms(&self) -> &[RetrievalTerm] {
        &self.data.retrieval_terms
    }

    pub fn scope(&self) -> &EntryScope {
        &self.data.scope
    }

    pub fn proof(&self) -> &EntryProof {
        &self.data.proof
    }

    pub fn oracle(&self) -> &ValidatedOracle {
        &self.data.oracle
    }

    pub fn created_at(&self) -> &UtcTimestamp {
        &self.data.created_at
    }

    pub fn transition(&self) -> Option<&EntryTransition> {
        self.data.transition.as_ref()
    }
}

#[derive(Debug)]
pub(crate) struct EntryData {
    pub(crate) id: MemoryId,
    pub(crate) status: Status,
    pub(crate) statement: Statement,
    pub(crate) scope: EntryScope,
    pub(crate) retrieval_terms: Vec<RetrievalTerm>,
    pub(crate) proof: EntryProof,
    pub(crate) oracle: ValidatedOracle,
    pub(crate) created_at: UtcTimestamp,
    pub(crate) transition: Option<EntryTransition>,
}

#[derive(Debug)]
pub enum EntryScope {
    Project(ProjectKey),
    User,
}

#[derive(Debug)]
pub struct EntryProof {
    summary: String,
    sources: Vec<EntrySource>,
    established_at: UtcTimestamp,
}

impl EntryProof {
    pub(crate) fn new(
        summary: String,
        sources: Vec<EntrySource>,
        established_at: UtcTimestamp,
    ) -> Self {
        Self {
            summary,
            sources,
            established_at,
        }
    }

    pub fn summary(&self) -> &str {
        &self.summary
    }

    pub fn sources(&self) -> &[EntrySource] {
        &self.sources
    }

    pub fn established_at(&self) -> &UtcTimestamp {
        &self.established_at
    }
}

#[derive(Debug)]
pub struct EntrySource {
    kind: SourceKind,
    locator: String,
    fingerprint: Fingerprint,
}

impl EntrySource {
    pub(crate) fn new(kind: SourceKind, locator: String, fingerprint: Fingerprint) -> Self {
        Self {
            kind,
            locator,
            fingerprint,
        }
    }

    pub fn kind(&self) -> SourceKind {
        self.kind
    }

    pub fn locator(&self) -> &str {
        &self.locator
    }

    pub fn fingerprint(&self) -> &Fingerprint {
        &self.fingerprint
    }
}

#[derive(Debug)]
pub struct EntryTransition {
    from: Status,
    to: Status,
    at: UtcTimestamp,
    verdict: TransitionVerdict,
    reason: String,
}

impl EntryTransition {
    pub(crate) fn new(
        from: Status,
        to: Status,
        at: UtcTimestamp,
        verdict: TransitionVerdict,
        reason: String,
    ) -> Self {
        Self {
            from,
            to,
            at,
            verdict,
            reason,
        }
    }

    pub fn from(&self) -> Status {
        self.from
    }

    pub fn to(&self) -> Status {
        self.to
    }

    pub fn at(&self) -> &UtcTimestamp {
        &self.at
    }

    pub fn verdict(&self) -> TransitionVerdict {
        self.verdict
    }

    pub fn reason(&self) -> &str {
        &self.reason
    }
}
