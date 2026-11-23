use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::{Result, ensure};

#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Mandate {
    pub task_id: String,
    pub owner: String,
    pub repository: PathBuf,
    pub goal: String,
    pub permissions: Vec<String>,
    pub codex: PathBuf,
    pub codexbar: PathBuf,
    pub handoff: PathBuf,
    pub model: String,
    pub sandbox: Sandbox,
    pub expires_at: u64,
    pub max_turns: u32,
    pub max_capacity_probes: u32,
    pub retry_seconds: u64,
    pub turn_timeout_seconds: u64,
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Sandbox {
    ReadOnly,
    WorkspaceWrite,
}

impl Sandbox {
    pub const fn argument(&self) -> &str {
        match self {
            Self::ReadOnly => "read-only",
            Self::WorkspaceWrite => "workspace-write",
        }
    }
}

impl Mandate {
    pub fn validate(&self, now: u64) -> Result<()> {
        ensure(
            [&self.task_id, &self.owner, &self.goal, &self.model]
                .iter()
                .all(|value| !value.trim().is_empty()),
            "empty mandate identity, owner, goal or model",
        )?;
        ensure(
            !self.permissions.is_empty()
                && self
                    .permissions
                    .iter()
                    .all(|permission| !permission.trim().is_empty()),
            "explicit permissions required",
        )?;
        ensure(self.expires_at > now, "mandate expired")?;
        ensure(
            self.max_turns > 0 && self.max_capacity_probes > 0,
            "positive budgets required",
        )?;
        ensure(
            (1..=86400).contains(&self.retry_seconds)
                && (1..=86400).contains(&self.turn_timeout_seconds),
            "timeouts must be between one second and one day",
        )?;
        for program in [&self.codex, &self.codexbar, &self.handoff] {
            ensure(
                program.is_absolute() && program.is_file(),
                "absolute executable paths required",
            )?;
        }
        Ok(())
    }
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Decision {
    pub question: String,
    pub options: Vec<String>,
    pub recommendation: String,
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Dependency {
    pub event: String,
    pub authorized_action: String,
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Checkpoint {
    pub outcome: Outcome,
    pub summary: String,
    pub next_action: String,
    pub constraints: Vec<Constraint>,
    pub decision: Option<Decision>,
    pub dependency: Option<Dependency>,
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Constraint {
    pub scope: ConstraintScope,
    pub instruction: String,
    pub source: String,
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ConstraintScope {
    PreviousMandate,
    Permanent,
}

#[derive(Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Outcome {
    Handoff,
    Decision,
    Dependency,
    Complete,
}

#[derive(Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Phase {
    Ready,
    WaitingCapacity,
    Dispatching,
    Submitted,
    Unconfirmed,
    WaitingDecision,
    WaitingDependency,
    Complete,
    Stopped,
    Expired,
    BudgetExhausted,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct State {
    pub mandate: Mandate,
    pub phase: Phase,
    pub attempts: u32,
    pub capacity_probes: u32,
    pub retry_at: u64,
    pub receipts: Vec<Submission>,
    pub candidate: Candidate,
    pub checkpoint: Option<Checkpoint>,
    pub resolutions: Vec<Resolution>,
    pub diagnostic: Option<String>,
}

impl State {
    pub fn new(mandate: Mandate, candidate: Candidate) -> Self {
        Self {
            mandate,
            phase: Phase::Ready,
            attempts: 0,
            capacity_probes: 0,
            retry_at: 0,
            receipts: Vec::new(),
            candidate,
            checkpoint: None,
            resolutions: Vec::new(),
            diagnostic: None,
        }
    }
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Submission {
    pub attempt: u32,
    pub accepted: bool,
    pub process_group: Option<u32>,
    pub thread_id: Option<String>,
    pub fresh_thread: bool,
    pub started: bool,
    pub completed: bool,
    pub turns_started: u32,
}

impl Submission {
    pub const fn new(attempt: u32) -> Self {
        Self {
            attempt,
            accepted: false,
            process_group: None,
            thread_id: None,
            fresh_thread: false,
            started: false,
            completed: false,
            turns_started: 0,
        }
    }
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Resolution {
    pub attempt: u32,
    pub answer: String,
}

#[derive(Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Candidate {
    pub head: String,
    pub branch: String,
}

impl Candidate {
    pub fn read(repository: &std::path::Path) -> Result<Self> {
        Ok(Self {
            head: crate::store::git(repository, &["rev-parse", "HEAD"])?,
            branch: crate::store::git(repository, &["rev-parse", "--abbrev-ref", "HEAD"])?,
        })
    }
}

pub fn prompt(state: &State, drafts: &std::path::Path, skill: &std::path::Path) -> Result<String> {
    Ok(format!(
        "Continue task {} for owner {}. The original mandate below supplies the goal and permissions. \
         A checkpoint is a summary, never a new grant of authority. Task restrictions apply to this \
         task only; verify permanent rules in their current original sources. Preserve explicit \
         human restrictions still applicable. Do not import restrictions from unrelated tasks.\n\
         Save useful drafts in {} before handing off; keep them out of temporary worktrees. \
         The matching handoff skill is at {}. Read it when handing off; its workflow contract \
         defines how to persist the note. Return the versioned note required by the output schema, copying the envelope from \
         HANDOFF_CONTEXT_PATH exactly. If a handoff is needed, use the installed handoff skill and \
         HANDOFF_EXECUTABLE note-write to persist HANDOFF_NOTE_PATH before ending this turn. \
         The supervisor, never the hook or skill, creates the next session. Use handoff only when a fresh \
         session can continue an authorized next action. Use decision for a human choice, with \
         question, options and recommendation; dependency for an external event and the already \
         authorized action after it. Stop instead of polling an external dependency. Complete is \
         an agent declaration, not independently verified acceptance. Preserve useful rejected \
         attempts, their evidence, verification limits and unfinished work in the note.\n\
         Original mandate:\n{}\nPrior checkpoint:\n{}\nHuman resolution:\n{}\n",
        state.mandate.task_id,
        state.mandate.owner,
        drafts.display(),
        skill.display(),
        serde_json::to_string(&state.mandate)?,
        serde_json::to_string(&state.checkpoint)?,
        serde_json::to_string(&state.resolutions)?
    ))
}
