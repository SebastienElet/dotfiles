use serde::{Deserialize, Deserializer, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use uuid::Uuid;

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(transparent)]
pub struct TeamId(Uuid);

impl std::fmt::Display for TeamId {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(formatter)
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(transparent)]
pub struct IssueId(Uuid);

impl std::fmt::Display for IssueId {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(formatter)
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(transparent)]
pub struct StateId(Uuid);

impl std::fmt::Display for StateId {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(formatter)
    }
}

pub type Fallible<T> = Result<T, &'static str>;

pub fn nullable<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    Option::<T>::deserialize(deserializer)
}

pub fn present<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    T::deserialize(deserializer).map(Some)
}

#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Config {
    pub bkt_context: String,
    pub linear_workspace: String,
    pub repositories: Vec<RepositoryConfig>,
}

#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RepositoryConfig {
    pub workspace: String,
    pub repository: String,
    pub team_key: String,
    pub team_id: TeamId,
    #[serde(default, deserialize_with = "present")]
    pub completed_state_id: Option<StateId>,
}

impl RepositoryConfig {
    pub fn identity(&self) -> String {
        format!("{}/{}", self.workspace, self.repository)
    }
}

fn name(value: &str) -> bool {
    value
        .as_bytes()
        .first()
        .is_some_and(u8::is_ascii_alphanumeric)
        && value
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || b"_.-".contains(&c))
}

pub fn team_key(value: &str) -> bool {
    value.as_bytes().first().is_some_and(u8::is_ascii_uppercase)
        && value
            .bytes()
            .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit())
}

impl Config {
    pub fn validate(&self) -> Fallible<()> {
        if !name(&self.bkt_context) || !name(&self.linear_workspace) || self.repositories.is_empty()
        {
            return Err("invalid explicit context or repositories");
        }
        let mut repositories = BTreeSet::new();
        let mut teams = BTreeMap::new();
        for repo in &self.repositories {
            if !name(&repo.workspace)
                || !name(&repo.repository)
                || !team_key(&repo.team_key)
                || !repositories.insert(repo.identity())
            {
                return Err("invalid or duplicate repository");
            }
            let settings = (repo.team_id, repo.completed_state_id);
            if teams
                .insert(&repo.team_key, settings)
                .is_some_and(|previous| previous != settings)
            {
                return Err("inconsistent team settings");
            }
        }
        if teams
            .values()
            .map(|(id, _)| id)
            .collect::<BTreeSet<_>>()
            .len()
            != teams.len()
        {
            return Err("team identity maps to multiple keys");
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum StateType {
    Triage,
    Backlog,
    Unstarted,
    Started,
    Completed,
    Canceled,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct State {
    pub id: StateId,
    pub name: String,
    #[serde(rename = "type")]
    pub kind: StateType,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Attachment {
    pub id: String,
    pub url: String,
    pub title: String,
    #[serde(deserialize_with = "nullable")]
    pub subtitle: Option<String>,
}

#[derive(Clone, Debug)]
pub struct Team {
    pub id: TeamId,
    pub key: String,
    pub states: Vec<State>,
    pub completed_state_id: Option<StateId>,
}

#[derive(Clone, Debug)]
pub struct Issue {
    pub id: IssueId,
    pub identifier: String,
    pub team_id: TeamId,
    pub state: State,
    pub archived_at: Option<String>,
    pub attachments: Vec<Attachment>,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "UPPERCASE")]
pub enum PullRequestState {
    Open,
    Merged,
    Declined,
}

impl PullRequestState {
    pub const fn text(self) -> &'static str {
        match self {
            Self::Open => "OPEN",
            Self::Merged => "MERGED",
            Self::Declined => "DECLINED",
        }
    }
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PullRequest {
    pub id: u64,
    #[serde(skip_serializing)]
    pub title: String,
    #[serde(skip_serializing)]
    pub description: String,
    #[serde(skip_serializing)]
    pub branch: String,
    pub state: PullRequestState,
    pub url: String,
    pub repository: String,
    pub source_repository: String,
    pub source_repository_id: String,
    pub destination_repository_id: String,
    pub author_id: String,
    #[serde(skip_serializing)]
    pub team_key: String,
}
