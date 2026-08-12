use crate::config::RepoKey;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum PullRequestState {
    Open,
    Merged,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct PullRequestKey {
    pub repo: RepoKey,
    pub number: u64,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct PullRequest {
    pub key: PullRequestKey,
    pub title: String,
    pub body: String,
    pub branch: String,
    pub destination: String,
    pub url: String,
    pub draft: bool,
    pub state: PullRequestState,
    pub created_at: String,
    pub updated_at: String,
    pub awaiting_review: bool,
    pub feedback: Vec<Feedback>,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Feedback {
    pub created_at: String,
    pub kind: FeedbackKind,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum FeedbackKind {
    Comment,
    Task,
    ChangesRequested,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Identity {
    pub id: String,
    pub name: String,
    pub email: Option<String>,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Issue {
    pub identifier: String,
    pub title: String,
    pub url: String,
    pub priority: u8,
    pub updated_at: String,
    pub branch_name: String,
    pub state_type: IssueState,
    pub team_key: String,
    pub project: Option<String>,
    pub labels: Vec<String>,
    /// Identifiers of the issues blocking this one, excluding resolved blockers.
    pub blockers: Vec<String>,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum IssueState {
    Triage,
    Backlog,
    Unstarted,
    Started,
    Completed,
    Canceled,
}

impl IssueState {
    /// A resolved issue no longer blocks the issues depending on it.
    pub const fn is_resolved(self) -> bool {
        matches!(self, Self::Completed | Self::Canceled)
    }

    // Linear names the two columns the routine acts on Todo and In Progress; its API calls their
    // state types `unstarted` and `started`.
    /// Every other state is either not scheduled yet or already resolved.
    pub const fn is_actionable(self) -> bool {
        matches!(self, Self::Unstarted | Self::Started)
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Category {
    Review,
    Retour,
    Linear,
    Suivant,
}

impl Category {
    /// Every category, in report order; `rank` indexes into it.
    pub const ALL: [Self; 4] = [Self::Review, Self::Retour, Self::Linear, Self::Suivant];

    pub const fn rank(self) -> usize {
        match self {
            Self::Review => 0,
            Self::Retour => 1,
            Self::Linear => 2,
            Self::Suivant => 3,
        }
    }

    pub const fn label(self) -> &'static str {
        match self {
            Self::Review => "REVIEW",
            Self::Retour => "RETOUR",
            Self::Linear => "LINEAR",
            Self::Suivant => "SUIVANT",
        }
    }
}

impl Ord for Category {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.rank().cmp(&other.rank())
    }
}

impl PartialOrd for Category {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum LinearReason {
    MergedIssueIncomplete,
    StartedWithoutBranchOrPr,
    StartedStale,
    MissingProject,
    MissingLabel,
    MissingPriority,
    OpenPrWithoutIssue,
}

impl LinearReason {
    pub const fn label(self) -> &'static str {
        match self {
            Self::MergedIssueIncomplete => "merged PR with incomplete issue",
            Self::StartedWithoutBranchOrPr => "started issue without branch or PR",
            Self::StartedStale => "stale started issue",
            Self::MissingProject => "missing project",
            Self::MissingLabel => "missing label",
            Self::MissingPriority => "missing priority",
            Self::OpenPrWithoutIssue => "open PR without Linear issue",
        }
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Warning {
    pub categories: Vec<Category>,
    pub message: String,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Dataset {
    pub pull_requests: Vec<PullRequest>,
    pub issues: Vec<Issue>,
    pub warnings: Vec<Warning>,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Report {
    pub items: Vec<ReportItem>,
    pub warnings: Vec<Warning>,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct ReportItem {
    pub category: Category,
    pub track_index: usize,
    pub reference: String,
    pub title: String,
    pub url: String,
    pub event_at: String,
    pub reasons: Vec<LinearReason>,
    pub priority: Option<u8>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn category_order_follows_report_sections() {
        let mut categories = vec![
            Category::Linear,
            Category::Suivant,
            Category::Review,
            Category::Retour,
        ];

        categories.sort();

        assert_eq!(
            categories,
            [
                Category::Review,
                Category::Retour,
                Category::Linear,
                Category::Suivant,
            ]
        );
        assert_eq!(Category::Suivant.rank(), 3);
    }

    #[test]
    fn report_items_preserve_the_source_timestamp_and_allow_missing_priority() {
        let item = ReportItem {
            category: Category::Linear,
            track_index: 0,
            reference: "OPS-42".to_owned(),
            title: "Normalize report model".to_owned(),
            url: "https://example.test/issue/OPS-42".to_owned(),
            event_at: "2026-08-11T08:00:00Z".to_owned(),
            reasons: vec![LinearReason::MissingPriority],
            priority: None,
        };

        assert_eq!(item.event_at, "2026-08-11T08:00:00Z");
        assert_eq!(item.priority, None);
    }

    #[test]
    fn identities_are_owned_clonable_and_hashable() {
        let identity = Identity {
            id: "user-42".to_owned(),
            name: "Example User".to_owned(),
            email: Some("ada@example.test".to_owned()),
        };
        let identities = HashSet::from([identity.clone()]);

        assert!(identities.contains(&identity));
        assert!(format!("{identity:?}").contains("Example User"));
    }
}
