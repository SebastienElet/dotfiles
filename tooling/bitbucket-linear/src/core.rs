use crate::model::{Issue, PullRequest, PullRequestState, StateId, StateType, Team};
use serde::Serialize;
use std::collections::BTreeSet;

#[derive(Clone, Copy, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Operation {
    Link,
    Update,
}
#[derive(Clone, Serialize)]
pub struct AttachmentAction {
    pub url: String,
    pub title: String,
    pub subtitle: String,
    pub operation: Operation,
}

pub struct IssuePlan {
    pub issue: Issue,
    pub pull_requests: Vec<PullRequest>,
    pub attachments: Vec<AttachmentAction>,
    pub complete_state_id: Option<StateId>,
    pub problems: Vec<&'static str>,
}

const fn identifier_character(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_'
}

pub fn correlate(pr: &PullRequest, keys: &BTreeSet<String>) -> BTreeSet<String> {
    let mut identifiers = BTreeSet::new();
    for text in [&pr.title, &pr.branch, &pr.description] {
        let upper = text.to_ascii_uppercase();
        for key in keys {
            let prefix = format!("{key}-");
            for (start, _) in upper.match_indices(&prefix) {
                if start
                    .checked_sub(1)
                    .and_then(|i| upper.as_bytes().get(i))
                    .copied()
                    .is_some_and(identifier_character)
                {
                    continue;
                }
                let number = start + prefix.len();
                let digits = upper.as_bytes().get(number..).map_or(0, |suffix| {
                    suffix.iter().take_while(|b| b.is_ascii_digit()).count()
                });
                let end = number + digits;
                if digits == 0
                    || upper
                        .as_bytes()
                        .get(end)
                        .copied()
                        .is_some_and(identifier_character)
                {
                    continue;
                }
                if let Some(identifier) = upper.get(start..end) {
                    identifiers.insert(identifier.into());
                }
            }
        }
    }
    identifiers
}

pub fn plan_issue(
    issue: Issue,
    team: &Team,
    mut prs: Vec<PullRequest>,
    complete: bool,
) -> IssuePlan {
    prs.sort_by(|left, right| left.url.cmp(&right.url));
    let mut problems = Vec::new();
    let duplicate = prs
        .iter()
        .any(|pr| issue.attachments.iter().filter(|a| a.url == pr.url).count() > 1);
    if duplicate {
        problems.push("duplicate-attachment");
    }
    if issue.archived_at.is_some() {
        problems.push("archived-issue");
    }
    if issue.state.kind == StateType::Completed
        && prs.iter().any(|pr| pr.state == PullRequestState::Open)
    {
        problems.push("completed-with-open-pr");
    }
    let attachments = if duplicate || issue.archived_at.is_some() {
        Vec::new()
    } else {
        prs.iter()
            .filter_map(|pr| {
                let existing = issue.attachments.iter().find(|a| a.url == pr.url);
                if existing.is_some_and(|a| {
                    a.title == pr.title && a.subtitle.as_deref() == Some(pr.state.text())
                }) {
                    return None;
                }
                Some(AttachmentAction {
                    url: pr.url.clone(),
                    title: pr.title.clone(),
                    subtitle: pr.state.text().into(),
                    operation: if existing.is_some() {
                        Operation::Update
                    } else {
                        Operation::Link
                    },
                })
            })
            .collect()
    };
    let eligible = !matches!(issue.state.kind, StateType::Completed | StateType::Canceled)
        && prs.iter().any(|pr| pr.state == PullRequestState::Merged)
        && prs.iter().all(|pr| pr.state != PullRequestState::Open);
    let mut complete_state_id = None;
    if eligible && complete && problems.is_empty() {
        let completed: Vec<_> = team
            .states
            .iter()
            .filter(|s| {
                s.kind == StateType::Completed
                    && team.completed_state_id.is_none_or(|id| id == s.id)
            })
            .collect();
        if completed.len() == 1 {
            complete_state_id = completed.first().map(|s| s.id);
        } else {
            problems.push("completed-state-unresolved");
        }
    }
    if !complete {
        problems.push("incomplete-inventory");
    }
    IssuePlan {
        issue,
        pull_requests: prs,
        attachments,
        complete_state_id,
        problems,
    }
}

pub fn same_issue(previous: &Issue, current: &Issue) -> bool {
    previous.id == current.id
        && previous.identifier == current.identifier
        && previous.team_id == current.team_id
        && previous.state.id == current.state.id
        && previous.state.kind == current.state.kind
        && previous.archived_at == current.archived_at
}

pub fn attachments_match(prs: &[PullRequest], issue: &Issue) -> bool {
    prs.iter().all(|pr| {
        let matches: Vec<_> = issue
            .attachments
            .iter()
            .filter(|a| a.url == pr.url)
            .collect();
        matches.len() == 1
            && matches.first().is_some_and(|a| {
                a.title == pr.title && a.subtitle.as_deref() == Some(pr.state.text())
            })
    })
}
