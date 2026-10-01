use crate::{
    bitbucket::read_pull_requests,
    core::{IssuePlan, Operation, attachments_match, correlate, plan_issue, same_issue},
    model::{Fallible, PullRequest, Team},
    providers::Providers,
    writes,
};
use serde::Serialize;
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    Unchanged,
    Linked,
    Updated,
    Completed,
    Ambiguous,
    Ignored,
    Failed,
}
impl Status {
    pub const fn text(self) -> &'static str {
        match self {
            Self::Unchanged => "unchanged",
            Self::Linked => "linked",
            Self::Updated => "updated",
            Self::Completed => "completed",
            Self::Ambiguous => "ambiguous",
            Self::Ignored => "ignored",
            Self::Failed => "failed",
        }
    }
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Observation {
    pub status: Status,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub identifier: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repository: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub observed_state: Option<String>,
}
impl Observation {
    pub const fn new(status: Status, reason: Option<&'static str>) -> Self {
        Self {
            status,
            reason,
            identifier: None,
            repository: None,
            url: None,
            observed_state: None,
        }
    }
    fn issue(status: Status, plan: &IssuePlan, reason: Option<&'static str>) -> Self {
        Self {
            identifier: Some(plan.issue.identifier.clone()),
            observed_state: Some(plan.issue.state.name.clone()),
            ..Self::new(status, reason)
        }
    }
}

pub struct Plan {
    identity: Value,
    pub issues: Vec<IssuePlan>,
    pub observations: Vec<Observation>,
}
struct Inventory {
    teams: BTreeMap<String, Team>,
    blocked: BTreeSet<String>,
    prs: Vec<PullRequest>,
    observations: Vec<Observation>,
}

fn inventory(providers: &Providers<'_>) -> Inventory {
    let mut inventory = Inventory {
        teams: BTreeMap::new(),
        blocked: BTreeSet::new(),
        prs: Vec::new(),
        observations: Vec::new(),
    };
    for repo in &providers.config.repositories {
        let result: Fallible<()> = (|| {
            if !inventory.teams.contains_key(&repo.team_key) {
                inventory
                    .teams
                    .insert(repo.team_key.clone(), providers.team(repo)?);
            }
            inventory.prs.extend(read_pull_requests(providers, repo)?);
            Ok(())
        })();
        if result.is_err() {
            inventory.blocked.insert(repo.team_key.clone());
            inventory.observations.push(Observation {
                repository: Some(repo.identity()),
                ..Observation::new(Status::Failed, Some("inventory-or-team-failed"))
            });
        }
    }
    inventory
}

fn associate(
    prs: Vec<PullRequest>,
    keys: &BTreeSet<String>,
    observations: &mut Vec<Observation>,
) -> (BTreeMap<String, Vec<PullRequest>>, BTreeSet<String>) {
    let mut matches: BTreeMap<String, Vec<PullRequest>> = BTreeMap::new();
    let mut blocked = BTreeSet::new();
    for pr in prs {
        let identifiers = correlate(&pr, keys);
        let first = identifiers.first();
        if identifiers.len() != 1 {
            let status = if identifiers.is_empty() {
                Status::Ignored
            } else {
                Status::Ambiguous
            };
            let reason = if identifiers.is_empty() {
                "no-configured-identifier"
            } else {
                "multiple-identifiers"
            };
            blocked.extend(identifiers.iter().cloned());
            observations.push(Observation {
                repository: Some(pr.repository.clone()),
                url: Some(pr.url.clone()),
                ..Observation::new(status, Some(reason))
            });
        } else if let Some(identifier) = first {
            if identifier.starts_with(&format!("{}-", pr.team_key)) {
                matches.entry(identifier.clone()).or_default().push(pr);
            } else {
                blocked.insert(identifier.clone());
                observations.push(Observation {
                    identifier: Some(identifier.clone()),
                    url: Some(pr.url.clone()),
                    ..Observation::new(Status::Ambiguous, Some("repository-team-mismatch"))
                });
            }
        }
    }
    (matches, blocked)
}

pub fn collect(providers: &Providers<'_>) -> Fallible<Plan> {
    let identity = providers.identity()?;
    let source = inventory(providers);
    let mut observations = source.observations;
    let keys = providers
        .config
        .repositories
        .iter()
        .map(|r| r.team_key.clone())
        .collect();
    let (matches, blocked) = associate(source.prs, &keys, &mut observations);
    let mut issues = Vec::new();
    for (identifier, prs) in matches {
        let Some(team) = prs.first().and_then(|pr| source.teams.get(&pr.team_key)) else {
            continue;
        };
        match providers.issue(&identifier) {
            Ok(issue) if issue.identifier == identifier && issue.team_id == team.id => {
                let complete =
                    !blocked.contains(&identifier) && !source.blocked.contains(&team.key);
                issues.push(plan_issue(issue, team, prs, complete));
            }
            Ok(_) => observations.push(Observation {
                identifier: Some(identifier),
                ..Observation::new(Status::Ambiguous, Some("resolved-issue-team-mismatch"))
            }),
            Err(_) => observations.push(Observation {
                identifier: Some(identifier),
                ..Observation::new(Status::Failed, Some("issue-or-attachments-incomplete"))
            }),
        }
    }
    Ok(Plan {
        identity,
        issues,
        observations,
    })
}

impl Plan {
    pub fn public(&self) -> Value {
        json!({"phase":"plan", "identity":self.identity, "observations":self.observations,
            "issues":self.issues.iter().map(|entry| json!({"issueId":entry.issue.id,"identifier":entry.issue.identifier,
                "teamId":entry.issue.team_id,"observedState":entry.issue.state,"problems":entry.problems,
                "pullRequests":entry.pull_requests,"attachments":entry.attachments,"completeStateId":entry.complete_state_id})).collect::<Vec<_>>()})
    }
}

fn apply_issue(plan: &IssuePlan, providers: &Providers<'_>) -> Vec<Observation> {
    let mut observations = Vec::new();
    let mut attachment_failed = false;
    for action in &plan.attachments {
        let observation = if writes::attach(providers, plan.issue.id, action).is_ok() {
            Observation::issue(
                match action.operation {
                    Operation::Link => Status::Linked,
                    Operation::Update => Status::Updated,
                },
                plan,
                None,
            )
        } else {
            attachment_failed = true;
            Observation::issue(
                Status::Failed,
                plan,
                Some("attachment-mutation-or-readback-failed"),
            )
        };
        observations.push(Observation {
            url: Some(action.url.clone()),
            ..observation
        });
    }
    if let Some(state_id) = plan.complete_state_id.filter(|_| !attachment_failed) {
        let completion: Fallible<()> = (|| {
            let fresh = providers.issue(&plan.issue.id.to_string())?;
            if !same_issue(&plan.issue, &fresh) || !attachments_match(&plan.pull_requests, &fresh) {
                return Err("issue changed since planning");
            }
            writes::complete(providers, plan.issue.id, state_id)
        })();
        if completion.is_ok() {
            observations.push(Observation {
                observed_state: Some(state_id.to_string()),
                ..Observation::issue(Status::Completed, plan, None)
            });
        } else {
            observations.push(Observation::issue(
                Status::Failed,
                plan,
                Some("state-or-attachments-changed-or-mutation-or-readback-failed"),
            ));
        }
    }
    if observations.is_empty() {
        observations.push(Observation::issue(Status::Unchanged, plan, None));
    }
    observations
}

pub fn finish(mut plan: Plan, providers: &Providers<'_>, apply: bool) -> (Vec<Observation>, u8) {
    for issue in &plan.issues {
        for reason in &issue.problems {
            plan.observations
                .push(Observation::issue(Status::Failed, issue, Some(reason)));
        }
        if apply {
            plan.observations.extend(apply_issue(issue, providers));
        } else {
            plan.observations.push(Observation::issue(
                Status::Unchanged,
                issue,
                Some("inspection"),
            ));
        }
    }
    let code = u8::from(
        plan.observations
            .iter()
            .any(|o| matches!(o.status, Status::Failed | Status::Ambiguous)),
    );
    (plan.observations, code)
}
