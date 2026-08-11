use crate::config::Config;
use crate::model::{
    Category, Dataset, Issue, IssueState, LinearReason, PullRequest, PullRequestState, Report,
    ReportItem, Warning,
};
use crate::util::{Timestamp, find_linear_id, parse_date_days, parse_timestamp};
use std::collections::{HashMap, HashSet};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TrackSelection {
    pub track_index: usize,
    pub requires_linear: bool,
}

pub fn track_for_pr(
    config: &Config,
    pr: &PullRequest,
    issue: Option<&Issue>,
) -> Option<TrackSelection> {
    let mut fallback = None;

    for (track_index, track) in config.tracks.iter().enumerate() {
        for repo in &track.repos {
            if repo.provider != pr.key.repo.provider || repo.path != pr.key.repo.path {
                continue;
            }

            let selection = TrackSelection {
                track_index,
                requires_linear: repo.requires_linear,
            };
            fallback.get_or_insert(selection);

            if issue.is_some_and(|issue| contains_team(&track.teams, &issue.team_key)) {
                return Some(selection);
            }
        }
    }

    fallback
}

pub fn track_for_issue(config: &Config, issue: &Issue) -> Option<usize> {
    config
        .tracks
        .iter()
        .position(|track| contains_team(&track.teams, &issue.team_key))
}

fn contains_team(teams: &[String], team_key: &str) -> bool {
    teams.iter().any(|team| team.eq_ignore_ascii_case(team_key))
}

pub fn pr_linear_reasons(pr: &PullRequest, issue: &Issue) -> Vec<LinearReason> {
    match pr.state {
        PullRequestState::Merged if issue.state_type != IssueState::Completed => {
            vec![LinearReason::MergedIssueIncomplete]
        }
        PullRequestState::Open
            if matches!(issue.state_type, IssueState::Backlog | IssueState::Triage) =>
        {
            vec![LinearReason::OpenIssueNotStarted]
        }
        _ => Vec::new(),
    }
}

pub fn issue_linear_reasons(
    issue: &Issue,
    has_correlated_pr: bool,
    today_days: i64,
    stale_days: u64,
) -> Vec<LinearReason> {
    let mut reasons = Vec::new();

    if issue.state_type == IssueState::Started {
        if issue.branch_name.trim().is_empty() && !has_correlated_pr {
            reasons.push(LinearReason::StartedWithoutBranchOrPr);
        }
        if parse_date_days(&issue.updated_at).is_ok_and(|updated_days| {
            u64::try_from(today_days.saturating_sub(updated_days)).is_ok_and(|age| age > stale_days)
        }) {
            reasons.push(LinearReason::StartedStale);
        }
    }
    if issue.project.is_none() {
        reasons.push(LinearReason::MissingProject);
    }
    if issue.labels.is_empty() {
        reasons.push(LinearReason::MissingLabel);
    }
    if issue.priority == 0 {
        reasons.push(LinearReason::MissingPriority);
    }

    reasons
}

// A blocked issue is not actionable today: it belongs neither in the discrepancy list nor among
// the next candidates, whatever else is wrong with it.
fn is_blocked(issue: &Issue) -> bool {
    !issue.blockers.is_empty()
}

pub fn uncorrelated_pr_reasons(pr: &PullRequest, requires_linear: bool) -> Vec<LinearReason> {
    if requires_linear
        && pr.state == PullRequestState::Open
        && !pr.awaiting_review
        && !pr.branch.starts_with("renovate/")
    {
        vec![LinearReason::OpenPrWithoutIssue]
    } else {
        Vec::new()
    }
}

struct ScopedPullRequest<'a> {
    pr: &'a PullRequest,
    issue: Option<&'a Issue>,
    has_identifier: bool,
    selection: Option<TrackSelection>,
}

pub fn build_report(config: &Config, dataset: &Dataset, today_days: i64) -> Report {
    let mut items = Vec::new();
    let mut warnings = dataset.warnings.clone();
    let team_keys = config.team_keys();
    let mut issue_by_identifier = HashMap::new();
    for issue in &dataset.issues {
        if track_for_issue(config, issue).is_none() {
            continue;
        }
        issue_by_identifier
            .entry(issue.identifier.to_ascii_uppercase())
            .or_insert(issue);
    }

    let pull_requests = dataset
        .pull_requests
        .iter()
        .map(|pr| {
            let identifier = find_linear_id(&pr.title, &pr.branch, &pr.body, &team_keys);
            let issue = identifier
                .as_ref()
                .and_then(|identifier| issue_by_identifier.get(identifier).copied());
            ScopedPullRequest {
                pr,
                issue,
                has_identifier: identifier.is_some(),
                selection: track_for_pr(config, pr, issue),
            }
        })
        .collect::<Vec<_>>();

    add_review_items(&pull_requests, &mut items, &mut warnings);
    add_retour_items(&pull_requests, &mut items, &mut warnings);
    add_linear_items(
        config,
        dataset,
        &pull_requests,
        today_days,
        &mut items,
        &mut warnings,
    );
    add_suivant_items(config, dataset, &mut items, &mut warnings);

    items.sort_by(|left, right| {
        left.category
            .cmp(&right.category)
            .then_with(|| compare_event_at(&left.event_at, &right.event_at))
            .then_with(|| left.track_index.cmp(&right.track_index))
            .then_with(|| left.reference.cmp(&right.reference))
            .then_with(|| left.url.cmp(&right.url))
    });

    Report { items, warnings }
}

fn add_review_items(
    pull_requests: &[ScopedPullRequest<'_>],
    items: &mut Vec<ReportItem>,
    warnings: &mut Vec<Warning>,
) {
    for scoped in pull_requests {
        let pr = scoped.pr;
        let Some(selection) = scoped.selection else {
            continue;
        };
        if pr.state != PullRequestState::Open || !pr.awaiting_review || pr.draft {
            continue;
        }
        if parse_event_timestamp(
            &pr.created_at,
            Category::Review,
            &pr_reference(pr),
            "creation",
            warnings,
        )
        .is_none()
        {
            continue;
        }

        items.push(pr_item(
            Category::Review,
            selection.track_index,
            pr,
            pr.created_at.clone(),
            Vec::new(),
        ));
    }
}

fn add_retour_items(
    pull_requests: &[ScopedPullRequest<'_>],
    items: &mut Vec<ReportItem>,
    warnings: &mut Vec<Warning>,
) {
    for scoped in pull_requests {
        let pr = scoped.pr;
        let Some(selection) = scoped.selection else {
            continue;
        };
        if pr.state != PullRequestState::Open || pr.awaiting_review || pr.feedback.is_empty() {
            continue;
        }

        let reference = pr_reference(pr);
        let feedback_context = format!("feedback for {reference}");
        let mut oldest = None;
        for feedback in &pr.feedback {
            let Some(timestamp) = parse_event_timestamp(
                &feedback.created_at,
                Category::Retour,
                &feedback_context,
                "feedback",
                warnings,
            ) else {
                continue;
            };
            if oldest.is_none_or(|(_, oldest_timestamp)| timestamp < oldest_timestamp) {
                oldest = Some((feedback.created_at.as_str(), timestamp));
            }
        }
        let Some((oldest, _)) = oldest else {
            continue;
        };

        items.push(pr_item(
            Category::Retour,
            selection.track_index,
            pr,
            oldest.to_owned(),
            Vec::new(),
        ));
    }
}

fn add_linear_items(
    config: &Config,
    dataset: &Dataset,
    pull_requests: &[ScopedPullRequest<'_>],
    today_days: i64,
    items: &mut Vec<ReportItem>,
    warnings: &mut Vec<Warning>,
) {
    let has_correlated_pr = pull_requests
        .iter()
        .filter(|scoped| scoped.selection.is_some())
        .filter_map(|scoped| scoped.issue)
        .map(|issue| issue.identifier.to_ascii_uppercase())
        .collect::<HashSet<_>>();
    let mut issue_items = HashMap::<String, LinearIssueItem>::new();

    for issue in &dataset.issues {
        let Some(track_index) = track_for_issue(config, issue) else {
            continue;
        };
        if is_blocked(issue) {
            continue;
        }
        let reasons = issue_linear_reasons(
            issue,
            has_correlated_pr.contains(&issue.identifier.to_ascii_uppercase()),
            today_days,
            config.stale_days,
        );
        let timestamp_needed = issue.state_type == IssueState::Started || !reasons.is_empty();
        let timestamp = if timestamp_needed {
            let Some(timestamp) = parse_event_timestamp(
                &issue.updated_at,
                Category::Linear,
                &issue.identifier,
                "issue update",
                warnings,
            ) else {
                continue;
            };
            Some(timestamp)
        } else {
            None
        };
        if !reasons.is_empty() {
            let Some(timestamp) = timestamp else {
                continue;
            };
            add_issue_linear_reasons(
                &mut issue_items,
                issue,
                track_index,
                &issue.updated_at,
                timestamp,
                LinearReasonOrigin::Issue,
                reasons,
            );
        }
    }

    for scoped in pull_requests {
        let pr = scoped.pr;
        let Some(selection) = scoped.selection else {
            continue;
        };
        if config.tracks[selection.track_index].teams.is_empty() {
            continue;
        }

        if let Some(issue) = scoped.issue {
            if is_blocked(issue) {
                continue;
            }
            let reasons = pr_linear_reasons(pr, issue);
            if reasons.is_empty() {
                continue;
            }
            let event_at = match pr.state {
                PullRequestState::Open => &pr.created_at,
                PullRequestState::Merged => &pr.updated_at,
            };
            let Some(event_timestamp) = parse_event_timestamp(
                event_at,
                Category::Linear,
                &format!("{} ({})", issue.identifier, pr_reference(pr)),
                "pull request event",
                warnings,
            ) else {
                continue;
            };
            add_issue_linear_reasons(
                &mut issue_items,
                issue,
                selection.track_index,
                event_at,
                event_timestamp,
                LinearReasonOrigin::PullRequest,
                reasons,
            );
        } else if !scoped.has_identifier {
            let reasons = uncorrelated_pr_reasons(pr, selection.requires_linear);
            if reasons.is_empty() {
                continue;
            }
            if parse_event_timestamp(
                &pr.created_at,
                Category::Linear,
                &pr_reference(pr),
                "creation",
                warnings,
            )
            .is_none()
            {
                continue;
            }
            items.push(pr_item(
                Category::Linear,
                selection.track_index,
                pr,
                pr.created_at.clone(),
                reasons,
            ));
        }
    }

    items.extend(issue_items.into_values().map(|item| item.report_item));
}

struct LinearIssueItem {
    report_item: ReportItem,
    event_timestamp: Timestamp,
    pull_request_track: Option<(Timestamp, usize)>,
}

#[derive(Clone, Copy)]
enum LinearReasonOrigin {
    Issue,
    PullRequest,
}

fn add_issue_linear_reasons(
    issue_items: &mut HashMap<String, LinearIssueItem>,
    issue: &Issue,
    track_index: usize,
    event_at: &str,
    event_timestamp: Timestamp,
    origin: LinearReasonOrigin,
    reasons: Vec<LinearReason>,
) {
    let item = issue_items
        .entry(issue.identifier.to_ascii_uppercase())
        .or_insert_with(|| LinearIssueItem {
            report_item: ReportItem {
                category: Category::Linear,
                track_index,
                reference: issue.identifier.clone(),
                title: issue.title.clone(),
                url: issue.url.clone(),
                event_at: event_at.to_owned(),
                reasons: Vec::new(),
                priority: None,
            },
            event_timestamp,
            pull_request_track: None,
        });
    if matches!(origin, LinearReasonOrigin::PullRequest)
        && item
            .pull_request_track
            .is_none_or(|current| (event_timestamp, track_index) < current)
    {
        item.report_item.track_index = track_index;
        item.pull_request_track = Some((event_timestamp, track_index));
    }
    if event_timestamp < item.event_timestamp {
        item.report_item.event_at = event_at.to_owned();
        item.event_timestamp = event_timestamp;
    }
    item.report_item.reasons.extend(reasons);
    item.report_item
        .reasons
        .sort_by_key(|reason| linear_reason_rank(*reason));
    item.report_item.reasons.dedup();
}

fn add_suivant_items(
    config: &Config,
    dataset: &Dataset,
    items: &mut Vec<ReportItem>,
    warnings: &mut Vec<Warning>,
) {
    for (track_index, track) in config.tracks.iter().enumerate() {
        if track.teams.is_empty() {
            continue;
        }
        let candidates = select_suivant_candidates(config, dataset, track_index, warnings);

        items.extend(candidates.into_iter().map(|(issue, _)| ReportItem {
            category: Category::Suivant,
            track_index,
            reference: issue.identifier.clone(),
            title: issue.title.clone(),
            url: issue.url.clone(),
            event_at: issue.updated_at.clone(),
            reasons: Vec::new(),
            priority: Some(issue.priority),
        }));
    }
}

fn select_suivant_candidates<'a>(
    config: &Config,
    dataset: &'a Dataset,
    track_index: usize,
    warnings: &mut Vec<Warning>,
) -> Vec<(&'a Issue, Timestamp)> {
    if config
        .tracks
        .get(track_index)
        .is_none_or(|track| track.teams.is_empty())
    {
        return Vec::new();
    }

    let mut candidates = Vec::new();
    for issue in &dataset.issues {
        if track_for_issue(config, issue) != Some(track_index)
            || is_blocked(issue)
            || !matches!(
                issue.state_type,
                IssueState::Triage | IssueState::Backlog | IssueState::Unstarted
            )
        {
            continue;
        }
        let Some(timestamp) = parse_event_timestamp(
            &issue.updated_at,
            Category::Suivant,
            &issue.identifier,
            "issue update",
            warnings,
        ) else {
            continue;
        };
        candidates.push((issue, timestamp));
    }
    candidates.sort_by(|left, right| {
        priority_rank(left.0.priority)
            .cmp(&priority_rank(right.0.priority))
            .then_with(|| left.1.cmp(&right.1))
            .then_with(|| left.0.identifier.cmp(&right.0.identifier))
    });
    candidates.truncate(config.next_count);
    candidates
}

fn pr_item(
    category: Category,
    track_index: usize,
    pr: &PullRequest,
    event_at: String,
    reasons: Vec<LinearReason>,
) -> ReportItem {
    ReportItem {
        category,
        track_index,
        reference: pr_reference(pr),
        title: pr.title.clone(),
        url: pr.url.clone(),
        event_at,
        reasons,
        priority: None,
    }
}

fn pr_reference(pr: &PullRequest) -> String {
    format!("#{}", pr.key.number)
}

fn parse_event_timestamp(
    value: &str,
    category: Category,
    context: &str,
    field: &str,
    warnings: &mut Vec<Warning>,
) -> Option<Timestamp> {
    match parse_timestamp(value) {
        Ok(timestamp) => Some(timestamp),
        Err(error) => {
            warnings.push(Warning {
                categories: vec![category],
                message: format!("omitted {context}: invalid {field} timestamp `{value}`: {error}"),
            });
            None
        }
    }
}

fn compare_event_at(left: &str, right: &str) -> std::cmp::Ordering {
    parse_timestamp(left).ok().cmp(&parse_timestamp(right).ok())
}

const fn linear_reason_rank(reason: LinearReason) -> usize {
    match reason {
        LinearReason::MergedIssueIncomplete => 0,
        LinearReason::OpenIssueNotStarted => 1,
        LinearReason::StartedWithoutBranchOrPr => 2,
        LinearReason::StartedStale => 3,
        LinearReason::MissingProject => 4,
        LinearReason::MissingLabel => 5,
        LinearReason::MissingPriority => 6,
        LinearReason::OpenPrWithoutIssue => 7,
    }
}

const fn priority_rank(priority: u8) -> u16 {
    match priority {
        1..=4 => (priority - 1) as u16,
        0 => 4,
        _ => 5 + priority as u16,
    }
}

#[cfg(test)]
mod attachment_tests;

#[cfg(test)]
mod report_tests;
