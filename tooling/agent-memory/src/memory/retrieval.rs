mod operation;
mod transition;

use super::validation::validate_transition_reason;
use super::{
    Clock, MemoryError, MemoryKind, OracleEnvironment, ProjectKey, ProofValid, SearchSelection,
    SourceKind, SourceResolver, Status, Store,
};
pub use operation::{retrieve, retrieve_for_injection};
use serde::Serialize;
use std::collections::BTreeMap;
pub use transition::confirm;

#[derive(Debug)]
pub struct HumanConclusion {
    terminal: HumanTerminal,
    reason: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum HumanTerminal {
    GoalAchieved,
    GoalAbandoned,
    DecisionSuperseded,
    UnknownResolved,
    AssumptionConfirmed,
}

impl HumanConclusion {
    pub fn goal_achieved(reason: &str) -> Result<Self, MemoryError> {
        Self::new(HumanTerminal::GoalAchieved, reason)
    }

    pub fn goal_abandoned(reason: &str) -> Result<Self, MemoryError> {
        Self::new(HumanTerminal::GoalAbandoned, reason)
    }

    pub fn decision_superseded(reason: &str) -> Result<Self, MemoryError> {
        Self::new(HumanTerminal::DecisionSuperseded, reason)
    }

    pub fn unknown_resolved(reason: &str) -> Result<Self, MemoryError> {
        Self::new(HumanTerminal::UnknownResolved, reason)
    }

    pub fn assumption_confirmed(reason: &str) -> Result<Self, MemoryError> {
        Self::new(HumanTerminal::AssumptionConfirmed, reason)
    }

    fn new(terminal: HumanTerminal, reason: &str) -> Result<Self, MemoryError> {
        validate_transition_reason(reason)?;
        Ok(Self {
            terminal,
            reason: reason.to_owned(),
        })
    }

    fn status_for(&self, kind: MemoryKind) -> Option<Status> {
        match (kind, self.terminal) {
            (MemoryKind::Goal, HumanTerminal::GoalAchieved) => Some(Status::Achieved),
            (MemoryKind::Goal, HumanTerminal::GoalAbandoned) => Some(Status::Abandoned),
            (MemoryKind::Decision, HumanTerminal::DecisionSuperseded) => Some(Status::Superseded),
            (MemoryKind::Unknown, HumanTerminal::UnknownResolved) => Some(Status::Resolved),
            (MemoryKind::Assumption, HumanTerminal::AssumptionConfirmed) => Some(Status::Confirmed),
            _ => None,
        }
    }
}

pub struct TransitionContext<'a> {
    store: &'a Store,
    clock: &'a dyn Clock,
}

impl<'a> TransitionContext<'a> {
    pub fn new(store: &'a Store, clock: &'a dyn Clock) -> Self {
        Self { store, clock }
    }
}

#[derive(Debug)]
pub struct TransitionResult {
    status: Status,
    index_rebuild_required: bool,
}

impl TransitionResult {
    pub fn status(&self) -> Status {
        self.status
    }

    pub fn index_rebuild_required(&self) -> bool {
        self.index_rebuild_required
    }
}

#[derive(Default)]
pub struct ProofAnswers {
    valid: BTreeMap<String, ProofValid>,
}

impl ProofAnswers {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, answer: ProofValid) {
        self.valid.insert(answer.entry_id().to_owned(), answer);
    }

    fn for_entry(&self, entry_id: &str) -> Option<&ProofValid> {
        self.valid.get(entry_id)
    }
}

pub struct RetrievalRequest<'a> {
    selection: &'a SearchSelection,
    project_key: &'a ProjectKey,
    include_user: bool,
}

impl<'a> RetrievalRequest<'a> {
    pub fn new(
        selection: &'a SearchSelection,
        project_key: &'a ProjectKey,
        include_user: bool,
    ) -> Self {
        Self {
            selection,
            project_key,
            include_user,
        }
    }
}

pub struct RetrievalContext<'a> {
    store: &'a Store,
    clock: &'a dyn Clock,
    resolver: &'a dyn SourceResolver,
    environment: OracleEnvironment,
    proof_answers: Option<&'a ProofAnswers>,
    deadline: Option<std::time::Instant>,
}

impl<'a> RetrievalContext<'a> {
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
            proof_answers: None,
            deadline: None,
        }
    }

    pub fn with_proof_answers(mut self, answers: &'a ProofAnswers) -> Self {
        self.proof_answers = Some(answers);
        self
    }

    pub fn with_deadline(mut self, deadline: std::time::Instant) -> Self {
        self.deadline = Some(deadline);
        self
    }

    fn deadline_exceeded(&self) -> bool {
        self.deadline
            .is_some_and(|deadline| std::time::Instant::now() >= deadline)
    }

    fn proof_valid(&self, entry_id: &str) -> Option<&ProofValid> {
        self.proof_answers
            .and_then(|answers| answers.for_entry(entry_id))
    }
}

#[derive(Debug, Eq, PartialEq, Serialize)]
pub struct RetrievalReport {
    pub injected: Vec<InjectedMemory>,
    pub omitted: Vec<OmittedMemory>,
    pub omitted_by_limit: usize,
}

#[derive(Debug, Eq, PartialEq, Serialize)]
pub struct InjectedMemory {
    pub id: String,
    pub kind: MemoryKind,
    pub statement: String,
    pub sources: Vec<SourceSummary>,
    pub verdict_age_milliseconds: u64,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct SourceSummary {
    pub kind: SourceKind,
    pub locator: Option<String>,
}

impl SourceSummary {
    pub fn with_locator(kind: SourceKind, locator: impl Into<String>) -> Self {
        Self {
            kind,
            locator: Some(locator.into()),
        }
    }

    pub fn redacted(kind: SourceKind) -> Self {
        Self {
            kind,
            locator: None,
        }
    }
}

#[derive(Debug, Eq, PartialEq, Serialize)]
pub struct OmittedMemory {
    pub id: String,
    pub code: String,
    pub question: Option<String>,
    pub effect: OmissionEffect,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum OmissionEffect {
    NotApplied,
}

impl OmissionEffect {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::NotApplied => "not_applied",
        }
    }
}
