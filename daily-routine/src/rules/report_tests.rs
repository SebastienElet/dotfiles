use super::*;
use crate::config::{Provider, RepoConfig, RepoKey, Track};
use crate::model::{
    Category, Dataset, Feedback, FeedbackKind, IssueState, LinearReason, PullRequestKey,
    PullRequestState, Warning,
};
use crate::util::days_from_civil;

#[test]
fn only_a_merged_pull_request_carries_a_discrepancy_of_its_own() {
    let mut issue = issue("ABC-1", IssueState::Unstarted, "2026-08-10T09:00:00Z");
    let mut pr = pull_request(1, PullRequestState::Merged, false);

    assert_eq!(
        pr_linear_reasons(&pr, &issue),
        [LinearReason::MergedIssueIncomplete]
    );

    pr.state = PullRequestState::Open;
    assert!(pr_linear_reasons(&pr, &issue).is_empty());

    issue.state_type = IssueState::Started;
    assert!(pr_linear_reasons(&pr, &issue).is_empty());

    issue.state_type = IssueState::Completed;
    pr.state = PullRequestState::Merged;
    assert!(pr_linear_reasons(&pr, &issue).is_empty());
}

#[test]
fn issues_outside_todo_and_in_progress_leave_the_report() {
    let config = config(3);
    let states = [
        IssueState::Triage,
        IssueState::Backlog,
        IssueState::Completed,
        IssueState::Canceled,
    ];
    let issues = states
        .into_iter()
        .enumerate()
        .map(|(index, state_type)| {
            let mut issue = issue(&format!("ABC-{index}"), state_type, "2026-08-08T09:00:00Z");
            issue.project = None;
            issue.labels.clear();
            issue.priority = 0;
            issue.branch_name.clear();
            issue
        })
        .collect::<Vec<_>>();
    let mut merged = pull_request(1, PullRequestState::Merged, false);
    merged.title = "Ship ABC-0".to_owned();

    let report = build_report(
        &config,
        &dataset(vec![merged], issues),
        days_from_civil(2026, 8, 11),
    );

    assert!(
        report.items.is_empty(),
        "neither a metadata gap nor a merged pull request revives an issue out of scope: {:?}",
        references(&report.items.iter().collect::<Vec<_>>())
    );
    assert!(report.warnings.is_empty());
}

#[test]
fn issue_rules_cover_started_staleness_and_missing_metadata() {
    let mut issue = issue("ABC-1", IssueState::Started, "2026-08-03T09:00:00Z");
    issue.branch_name.clear();
    issue.project = None;
    issue.labels.clear();
    issue.priority = 0;
    let today = days_from_civil(2026, 8, 11);

    assert_eq!(
        issue_linear_reasons(&issue, false, today, 7),
        [
            LinearReason::StartedWithoutBranchOrPr,
            LinearReason::StartedStale,
            LinearReason::MissingProject,
            LinearReason::MissingLabel,
            LinearReason::MissingPriority,
        ]
    );

    issue.updated_at = "2026-08-04T09:00:00Z".to_owned();
    assert!(!issue_linear_reasons(&issue, true, today, 7).contains(&LinearReason::StartedStale));
    assert!(
        !issue_linear_reasons(&issue, true, today, 7)
            .contains(&LinearReason::StartedWithoutBranchOrPr)
    );
}

#[test]
fn ticketless_pull_request_rule_honors_repo_scope_and_renovate() {
    let mut pr = pull_request(1, PullRequestState::Open, false);

    assert_eq!(
        uncorrelated_pr_reasons(&pr, true),
        [LinearReason::OpenPrWithoutIssue]
    );
    assert!(uncorrelated_pr_reasons(&pr, false).is_empty());

    pr.branch = "renovate/serde-2.x".to_owned();
    assert!(uncorrelated_pr_reasons(&pr, true).is_empty());

    pr.branch = "feature/change".to_owned();
    pr.awaiting_review = true;
    assert!(uncorrelated_pr_reasons(&pr, true).is_empty());

    pr.awaiting_review = false;
    pr.draft = true;
    assert_eq!(
        uncorrelated_pr_reasons(&pr, true),
        [LinearReason::OpenPrWithoutIssue]
    );
}

#[test]
fn review_and_retour_use_their_oldest_source_events() {
    let config = config(2);
    let mut review_newer = pull_request(2, PullRequestState::Open, true);
    review_newer.created_at = "2026-08-04T08:00:00Z".to_owned();
    let mut review_older = pull_request(1, PullRequestState::Open, true);
    review_older.created_at = "2026-08-04T09:30:00+02:00".to_owned();
    let mut draft_review = pull_request(3, PullRequestState::Open, true);
    draft_review.draft = true;
    let mut retour = pull_request(4, PullRequestState::Open, false);
    retour.feedback = vec![
        feedback("2026-08-06T10:00:00.355Z", FeedbackKind::Comment),
        feedback("2026-08-06T10:00:00Z", FeedbackKind::Task),
    ];
    let mut draft_retour = pull_request(5, PullRequestState::Open, false);
    draft_retour.draft = true;
    draft_retour.feedback = vec![feedback(
        "2026-08-03T10:00:00Z",
        FeedbackKind::ChangesRequested,
    )];
    let mut merged = pull_request(6, PullRequestState::Merged, false);
    merged.feedback = vec![feedback("2026-08-02T10:00:00Z", FeedbackKind::Comment)];

    let report = build_report(
        &config,
        &dataset(
            vec![
                review_newer,
                review_older,
                draft_review,
                retour,
                draft_retour,
                merged,
            ],
            Vec::new(),
        ),
        days_from_civil(2026, 8, 11),
    );

    let review = category_items(&report.items, Category::Review);
    assert_eq!(references(&review), ["#1", "#2"]);
    assert_eq!(review[0].event_at, "2026-08-04T09:30:00+02:00");

    let retour = category_items(&report.items, Category::Retour);
    assert_eq!(references(&retour), ["#5", "#4"]);
    assert_eq!(retour[0].event_at, "2026-08-03T10:00:00Z");
    assert_eq!(retour[1].event_at, "2026-08-06T10:00:00Z");
}

#[test]
fn retour_keeps_the_oldest_valid_feedback_and_warns_about_invalid_feedback() {
    let config = config(2);
    let mut pr = pull_request(7, PullRequestState::Open, false);
    pr.branch = "renovate/dependency".to_owned();
    pr.feedback = vec![
        feedback("invalid", FeedbackKind::Comment),
        feedback("2026-08-06T10:00:00Z", FeedbackKind::Task),
        feedback("2026-08-05T10:00:00Z", FeedbackKind::ChangesRequested),
    ];

    let report = build_report(
        &config,
        &dataset(vec![pr], Vec::new()),
        days_from_civil(2026, 8, 11),
    );

    let retour = category_items(&report.items, Category::Retour);
    assert_eq!(references(&retour), ["#7"]);
    assert_eq!(retour[0].event_at, "2026-08-05T10:00:00Z");
    assert!(report.warnings.iter().any(|warning| {
        warning.categories == [Category::Retour]
            && warning.message.contains("omitted feedback for #7")
    }));
}

#[test]
fn correlation_is_case_insensitive_and_any_scoped_pr_satisfies_started_issues() {
    let config = config(2);
    let mut title_issue = issue("abc-1", IssueState::Started, "2026-08-08T09:00:00Z");
    title_issue.branch_name.clear();
    let mut branch_issue = issue("ABC-2", IssueState::Started, "2026-08-08T09:00:00Z");
    branch_issue.branch_name.clear();
    let mut body_issue = issue("ABC-3", IssueState::Started, "2026-08-08T09:00:00Z");
    body_issue.branch_name.clear();
    let mut reviewer_issue = issue("ABC-4", IssueState::Started, "2026-08-08T09:00:00Z");
    reviewer_issue.branch_name.clear();
    reviewer_issue.project = None;

    let mut by_title = pull_request(1, PullRequestState::Open, false);
    by_title.title = "Ship abc-1".to_owned();
    by_title.branch = "feature/ABC-2".to_owned();
    by_title.body = "Relates ABC-3".to_owned();
    let mut by_branch = pull_request(2, PullRequestState::Open, false);
    by_branch.branch = "feature/ABC-2".to_owned();
    let mut by_body = pull_request(3, PullRequestState::Merged, false);
    by_body.body = "Relates ABC-3".to_owned();
    let mut reviewer = pull_request(4, PullRequestState::Open, true);
    reviewer.title = "Review ABC-4".to_owned();

    let report = build_report(
        &config,
        &dataset(
            vec![by_title, by_branch, by_body, reviewer],
            vec![title_issue, branch_issue, body_issue, reviewer_issue],
        ),
        days_from_civil(2026, 8, 11),
    );

    let linear = category_items(&report.items, Category::Linear);
    assert!(linear.iter().all(|item| {
        item.reference != "abc-1"
            || !item
                .reasons
                .contains(&LinearReason::StartedWithoutBranchOrPr)
    }));
    assert!(linear.iter().all(|item| {
        item.reference != "ABC-2"
            || !item
                .reasons
                .contains(&LinearReason::StartedWithoutBranchOrPr)
    }));
    assert!(linear.iter().all(|item| {
        item.reference != "ABC-3"
            || !item
                .reasons
                .contains(&LinearReason::StartedWithoutBranchOrPr)
    }));
    assert_eq!(
        linear
            .iter()
            .find(|item| item.reference == "ABC-4")
            .unwrap()
            .reasons,
        [LinearReason::MissingProject]
    );
}

#[test]
fn correlated_review_prs_emit_issue_rules_but_not_ticketless_pr_rules() {
    let config = config(2);
    let mut correlated_issue = issue("ABC-5", IssueState::Unstarted, "2026-08-08T09:00:00Z");
    correlated_issue.labels.clear();
    let mut correlated = pull_request(5, PullRequestState::Open, true);
    correlated.title = "Review ABC-5".to_owned();
    let ticketless = pull_request(6, PullRequestState::Open, true);

    let report = build_report(
        &config,
        &dataset(vec![ticketless, correlated], vec![correlated_issue]),
        days_from_civil(2026, 8, 11),
    );

    let linear = category_items(&report.items, Category::Linear);
    assert_eq!(
        linear
            .iter()
            .find(|item| item.reference == "ABC-5")
            .unwrap()
            .reasons,
        [LinearReason::MissingLabel]
    );
    assert!(linear.iter().all(|item| item.reference != "#6"));
}

#[test]
fn linear_items_consolidate_reasons_and_pull_requests_by_issue() {
    let config = config(2);
    let mut issue = issue("ABC-7", IssueState::Started, "2026-08-08T09:00:00Z");
    issue.project = None;
    issue.labels.clear();
    issue.priority = 0;
    let mut first = pull_request(7, PullRequestState::Merged, false);
    first.title = "ABC-7 first".to_owned();
    first.updated_at = "2026-08-01T09:00:00.355Z".to_owned();
    let mut second = pull_request(8, PullRequestState::Merged, false);
    second.body = "Related to ABC-7".to_owned();
    second.updated_at = "2026-08-01T09:00:00Z".to_owned();
    let no_ticket = pull_request(9, PullRequestState::Open, false);

    let report = build_report(
        &config,
        &dataset(vec![second, no_ticket.clone(), first], vec![issue.clone()]),
        days_from_civil(2026, 8, 11),
    );

    let linear = category_items(&report.items, Category::Linear);
    let issue_item = linear
        .iter()
        .find(|item| item.reference == "ABC-7")
        .unwrap();
    assert_eq!(issue_item.title, issue.title);
    assert_eq!(issue_item.url, issue.url);
    assert_eq!(issue_item.event_at, "2026-08-01T09:00:00Z");
    assert_eq!(
        issue_item.reasons,
        [
            LinearReason::MergedIssueIncomplete,
            LinearReason::MissingProject,
            LinearReason::MissingLabel,
            LinearReason::MissingPriority,
        ]
    );
    assert_eq!(
        linear
            .iter()
            .filter(|item| item.reference == "ABC-7")
            .count(),
        1
    );

    let no_ticket_item = linear.iter().find(|item| item.reference == "#9").unwrap();
    assert_eq!(no_ticket_item.title, no_ticket.title);
    assert_eq!(no_ticket_item.url, no_ticket.url);
    assert_eq!(no_ticket_item.reasons, [LinearReason::OpenPrWithoutIssue]);
}

#[test]
fn pr_discrepancy_uses_the_repository_track_when_the_issue_team_differs() {
    let mut config = config(0);
    config.tracks.push(Track {
        name: "Other team".to_owned(),
        teams: vec!["XYZ".to_owned()],
        repos: Vec::new(),
    });
    let mut issue = issue("XYZ-1", IssueState::Started, "2026-08-08T09:00:00Z");
    issue.team_key = "XYZ".to_owned();
    let mut pr = pull_request(1, PullRequestState::Merged, false);
    pr.title = "Ship XYZ-1".to_owned();

    let report = build_report(
        &config,
        &dataset(vec![pr], vec![issue]),
        days_from_civil(2026, 8, 11),
    );

    let linear = category_items(&report.items, Category::Linear);
    assert_eq!(linear.len(), 1);
    assert_eq!(linear[0].reference, "XYZ-1");
    assert_eq!(linear[0].track_index, 0);
    assert_eq!(linear[0].reasons, [LinearReason::MergedIssueIncomplete]);
}

#[test]
fn pr_discrepancy_overrides_the_track_of_consolidated_issue_reasons() {
    let mut config = config(0);
    config.tracks.push(Track {
        name: "Other team".to_owned(),
        teams: vec!["XYZ".to_owned()],
        repos: Vec::new(),
    });
    let mut issue = issue("XYZ-1", IssueState::Started, "2026-08-08T09:00:00Z");
    issue.team_key = "XYZ".to_owned();
    issue.priority = 0;
    let mut pr = pull_request(1, PullRequestState::Merged, false);
    pr.title = "Ship XYZ-1".to_owned();

    let report = build_report(
        &config,
        &dataset(vec![pr], vec![issue]),
        days_from_civil(2026, 8, 11),
    );

    let linear = category_items(&report.items, Category::Linear);
    assert_eq!(linear.len(), 1);
    assert_eq!(linear[0].track_index, 0);
    assert_eq!(
        linear[0].reasons,
        [
            LinearReason::MergedIssueIncomplete,
            LinearReason::MissingPriority,
        ]
    );
}

#[test]
fn oldest_pr_event_deterministically_owns_a_consolidated_discrepancy() {
    let mut config = config(0);
    config.tracks.push(Track {
        name: "Issue team".to_owned(),
        teams: vec!["XYZ".to_owned()],
        repos: Vec::new(),
    });
    config.tracks.push(Track {
        name: "Other repository".to_owned(),
        teams: vec!["OTH".to_owned()],
        repos: vec![repo(Provider::Github, "owner/other", true)],
    });
    let mut issue = issue("XYZ-1", IssueState::Started, "2026-08-08T09:00:00Z");
    issue.team_key = "XYZ".to_owned();
    let mut newer = pull_request(1, PullRequestState::Merged, false);
    newer.title = "Ship XYZ-1 from the primary repository".to_owned();
    newer.updated_at = "2026-08-06T09:00:00Z".to_owned();
    let mut older = pull_request(2, PullRequestState::Merged, false);
    older.key.repo = RepoKey {
        provider: Provider::Github,
        path: "owner/other".to_owned(),
    };
    older.title = "Ship XYZ-1 from the other repository".to_owned();
    older.updated_at = "2026-08-05T09:00:00Z".to_owned();

    for pull_requests in [
        vec![newer.clone(), older.clone()],
        vec![older.clone(), newer.clone()],
    ] {
        let report = build_report(
            &config,
            &dataset(pull_requests, vec![issue.clone()]),
            days_from_civil(2026, 8, 11),
        );
        let linear = category_items(&report.items, Category::Linear);

        assert_eq!(linear.len(), 1);
        assert_eq!(linear[0].track_index, 2);
        assert_eq!(linear[0].event_at, "2026-08-05T09:00:00Z");
    }
}

#[test]
fn linear_scope_keeps_drafts_but_excludes_no_linear_and_teamless_tracks() {
    let mut config = config(2);
    config.tracks.push(Track {
        name: "No Linear".to_owned(),
        teams: vec!["NOL".to_owned()],
        repos: vec![repo(Provider::Github, "owner/no-linear", false)],
    });
    config.tracks.push(Track {
        name: "Review only".to_owned(),
        teams: Vec::new(),
        repos: vec![repo(Provider::Github, "owner/review-only", true)],
    });
    let mut draft = pull_request(1, PullRequestState::Open, false);
    draft.draft = true;
    draft.title = "Draft ABC-1".to_owned();
    draft.destination = "stack/base".to_owned();
    let mut correlated_issue = issue("ABC-1", IssueState::Unstarted, "2026-08-09T09:00:00Z");
    correlated_issue.labels.clear();
    let mut renovate = pull_request(2, PullRequestState::Open, false);
    renovate.branch = "renovate/serde".to_owned();
    let mut no_linear = pull_request(3, PullRequestState::Open, false);
    no_linear.key.repo = RepoKey {
        provider: Provider::Github,
        path: "owner/no-linear".to_owned(),
    };
    let mut teamless = pull_request(4, PullRequestState::Open, false);
    teamless.key.repo = RepoKey {
        provider: Provider::Github,
        path: "owner/review-only".to_owned(),
    };
    teamless.feedback = vec![feedback("2026-08-01T09:00:00Z", FeedbackKind::Comment)];

    let report = build_report(
        &config,
        &dataset(
            vec![draft, renovate, no_linear, teamless],
            vec![correlated_issue],
        ),
        days_from_civil(2026, 8, 11),
    );

    let linear = category_items(&report.items, Category::Linear);
    assert_eq!(references(&linear), ["ABC-1"]);
    assert_eq!(linear[0].reasons, [LinearReason::MissingLabel]);
    assert_eq!(
        references(&category_items(&report.items, Category::Retour)),
        ["#4"]
    );
}

#[test]
fn linear_correlation_excludes_issues_from_unconfigured_teams() {
    let config = config(2);
    let mut unconfigured = issue("ABC-1", IssueState::Unstarted, "2026-08-09T09:00:00Z");
    unconfigured.team_key = "OUTSIDE".to_owned();
    let mut pr = pull_request(1, PullRequestState::Open, false);
    pr.title = "Ship ABC-1".to_owned();

    let report = build_report(
        &config,
        &dataset(vec![pr], vec![unconfigured]),
        days_from_civil(2026, 8, 11),
    );

    assert!(category_items(&report.items, Category::Linear).is_empty());
}

#[test]
fn the_limit_keeps_the_oldest_items_of_each_section_and_warns_about_the_rest() {
    let config = config(0);
    let issues = ["ABC-1", "ABC-2", "ABC-3", "ABC-4"]
        .into_iter()
        .zip([
            "2026-08-04T09:00:00Z",
            "2026-08-01T09:00:00Z",
            "2026-08-03T09:00:00Z",
            "2026-08-02T09:00:00Z",
        ])
        .map(|(identifier, updated_at)| {
            let mut issue = issue(identifier, IssueState::Unstarted, updated_at);
            issue.labels = Vec::new();
            issue
        })
        .collect::<Vec<_>>();
    let awaiting_review = pull_request(1, PullRequestState::Open, true);
    let mut report = build_report(
        &config,
        &dataset(vec![awaiting_review], issues),
        days_from_civil(2026, 8, 11),
    );

    withhold_beyond_limit(&mut report, 2);

    assert_eq!(
        references(&category_items(&report.items, Category::Linear)),
        ["ABC-2", "ABC-4"],
        "the two oldest discrepancies come first"
    );
    assert_eq!(
        references(&category_items(&report.items, Category::Review)).len(),
        1,
        "a section under the limit keeps every item"
    );
    assert_eq!(
        report.warnings,
        [Warning {
            categories: vec![Category::Linear],
            message: "2 further items withheld by --limit 2".to_owned(),
        }],
        "only the truncated section warns, and it says how many remain"
    );
}

#[test]
fn the_limit_leaves_a_report_it_does_not_truncate_untouched() {
    let config = config(0);
    let mut issue = issue("ABC-1", IssueState::Unstarted, "2026-08-04T09:00:00Z");
    issue.labels = Vec::new();
    let mut report = build_report(
        &config,
        &dataset(Vec::new(), vec![issue]),
        days_from_civil(2026, 8, 11),
    );
    let untouched = report.clone();

    withhold_beyond_limit(&mut report, 10);

    assert_eq!(report, untouched);
}

#[test]
fn blocked_issues_leave_the_discrepancy_list() {
    let config = config(0);
    let mut blocked = issue("ABC-1", IssueState::Started, "2026-08-01T09:00:00Z");
    blocked.labels = Vec::new();
    blocked.project = None;
    blocked.priority = 0;
    blocked.branch_name = String::new();
    blocked.blockers = vec!["ABC-9".to_owned()];
    let mut open = issue("ABC-2", IssueState::Started, "2026-08-01T09:00:00Z");
    open.labels = Vec::new();

    let report = build_report(
        &config,
        &dataset(Vec::new(), vec![blocked, open]),
        days_from_civil(2026, 8, 11),
    );

    assert_eq!(
        references(&category_items(&report.items, Category::Linear)),
        ["ABC-2"],
        "every discrepancy of a blocked issue waits for it to be unblocked"
    );
}

#[test]
fn blocked_issues_drop_their_pull_request_discrepancies_too() {
    let config = config(0);
    let mut blocked = issue("ABC-1", IssueState::Started, "2026-08-08T09:00:00Z");
    blocked.blockers = vec!["ABC-9".to_owned()];
    let mut pr = pull_request(1, PullRequestState::Merged, false);
    pr.title = "Ship ABC-1".to_owned();

    let report = build_report(
        &config,
        &dataset(vec![pr], vec![blocked]),
        days_from_civil(2026, 8, 11),
    );

    assert!(
        category_items(&report.items, Category::Linear).is_empty(),
        "a merged PR on a blocked issue reports no discrepancy either"
    );
}

#[test]
fn blocked_issues_never_become_next_candidates() {
    let config = config(2);
    let mut blocked = issue("ABC-1", IssueState::Unstarted, "2026-08-10T09:00:00Z");
    blocked.priority = 1;
    blocked.blockers = vec!["ABC-9".to_owned()];
    let mut ready = issue("ABC-2", IssueState::Unstarted, "2026-08-09T09:00:00Z");
    ready.priority = 2;
    let mut lower = issue("ABC-3", IssueState::Unstarted, "2026-08-08T09:00:00Z");
    lower.priority = 3;

    let report = build_report(
        &config,
        &dataset(Vec::new(), vec![blocked, ready, lower]),
        days_from_civil(2026, 8, 11),
    );

    assert_eq!(
        references(&category_items(&report.items, Category::Suivant)),
        ["ABC-3", "ABC-2"],
        "the blocked top priority frees its slot for the next actionable candidate, \
         and selection still precedes the chronological output order"
    );
}

#[test]
fn suivant_selects_by_linear_priority_then_sorts_chronologically() {
    let mut config = config(2);
    config.tracks.push(Track {
        name: "Second".to_owned(),
        teams: vec!["XYZ".to_owned()],
        repos: Vec::new(),
    });
    let mut newest_priority_one = issue("ABC-1", IssueState::Unstarted, "2026-08-10T09:00:00Z");
    newest_priority_one.priority = 1;
    let mut oldest_priority_one = issue("ABC-2", IssueState::Unstarted, "2026-08-09T09:00:00Z");
    oldest_priority_one.priority = 1;
    let mut priority_two = issue("ABC-3", IssueState::Unstarted, "2026-08-01T09:00:00Z");
    priority_two.priority = 2;
    let mut zero = issue("ABC-4", IssueState::Unstarted, "2026-07-01T09:00:00Z");
    zero.priority = 0;
    let completed = issue("ABC-5", IssueState::Completed, "2026-06-01T09:00:00Z");
    let canceled = issue("ABC-6", IssueState::Canceled, "2026-06-01T09:00:00Z");
    let mut second_track = issue("XYZ-1", IssueState::Unstarted, "2026-08-02T09:00:00Z");
    second_track.team_key = "XYZ".to_owned();
    second_track.priority = 3;

    let report = build_report(
        &config,
        &dataset(
            Vec::new(),
            vec![
                newest_priority_one,
                oldest_priority_one,
                priority_two,
                zero,
                completed,
                canceled,
                second_track,
            ],
        ),
        days_from_civil(2026, 8, 11),
    );

    let next = category_items(&report.items, Category::Suivant);
    assert_eq!(references(&next), ["XYZ-1", "ABC-2", "ABC-1"]);
    assert_eq!(
        next.iter().map(|item| item.priority).collect::<Vec<_>>(),
        [Some(3), Some(1), Some(1)]
    );
    assert!(next.iter().all(|item| item.reasons.is_empty()));
}

#[test]
fn suivant_selection_precedes_chronological_output_order() {
    let config = config(5);
    let mut priority_one = issue("ABC-1", IssueState::Unstarted, "2026-08-05T09:00:00Z");
    priority_one.priority = 1;
    let mut priority_two = issue("ABC-2", IssueState::Unstarted, "2026-08-04T09:00:00Z");
    priority_two.priority = 2;
    let mut priority_three = issue("ABC-3", IssueState::Unstarted, "2026-08-03T09:00:00Z");
    priority_three.priority = 3;
    let mut priority_four = issue("ABC-4", IssueState::Unstarted, "2026-08-02T09:00:00Z");
    priority_four.priority = 4;
    let mut no_priority = issue("ABC-0", IssueState::Unstarted, "2026-08-01T09:00:00Z");
    no_priority.priority = 0;
    let dataset = dataset(
        Vec::new(),
        vec![
            no_priority,
            priority_four,
            priority_three,
            priority_two,
            priority_one,
        ],
    );
    let mut warnings = Vec::new();

    let selected = select_suivant_candidates(&config, &dataset, 0, &mut warnings);

    assert_eq!(
        selected
            .iter()
            .map(|(issue, _)| issue.priority)
            .collect::<Vec<_>>(),
        [1, 2, 3, 4, 0]
    );
    assert!(warnings.is_empty());

    let report = build_report(&config, &dataset, days_from_civil(2026, 8, 11));
    assert_eq!(
        references(&category_items(&report.items, Category::Suivant)),
        ["ABC-0", "ABC-4", "ABC-3", "ABC-2", "ABC-1"]
    );
}

#[test]
fn report_orders_categories_then_event_track_and_reference() {
    let mut config = config(3);
    config.tracks.push(Track {
        name: "Second".to_owned(),
        teams: vec!["XYZ".to_owned()],
        repos: vec![repo(Provider::Github, "owner/second", true)],
    });
    let mut review_b = pull_request(2, PullRequestState::Open, true);
    review_b.created_at = "2026-08-02T09:00:00Z".to_owned();
    let mut review_a = pull_request(1, PullRequestState::Open, true);
    review_a.created_at = review_b.created_at.clone();
    let mut review_second_track = pull_request(3, PullRequestState::Open, true);
    review_second_track.key.repo = RepoKey {
        provider: Provider::Github,
        path: "owner/second".to_owned(),
    };
    review_second_track.created_at = review_a.created_at.clone();
    let mut retour = pull_request(4, PullRequestState::Open, false);
    retour.feedback = vec![feedback("2026-08-01T09:00:00Z", FeedbackKind::Comment)];
    let mut linear_issue = issue("ABC-8", IssueState::Started, "2026-07-01T09:00:00Z");
    linear_issue.branch_name.clear();
    let next_issue = issue("ABC-9", IssueState::Unstarted, "2026-06-01T09:00:00Z");

    let report = build_report(
        &config,
        &dataset(
            vec![review_b, review_second_track, retour, review_a],
            vec![next_issue, linear_issue],
        ),
        days_from_civil(2026, 8, 11),
    );

    assert!(
        report
            .items
            .windows(2)
            .all(|items| items[0].category <= items[1].category)
    );
    assert_eq!(
        references(&category_items(&report.items, Category::Review)),
        ["#1", "#2", "#3"]
    );
    assert!(report.items.iter().all(|item| {
        if item.category == Category::Suivant {
            item.priority.is_some()
        } else {
            item.priority.is_none()
        }
    }));
}

#[test]
fn report_uses_url_as_the_final_stable_tie_breaker() {
    let mut config = config(2);
    config.tracks[0]
        .repos
        .push(repo(Provider::Github, "owner/other", true));
    let mut later_url = pull_request(1, PullRequestState::Open, true);
    later_url.url = "https://z.example.test/pull/1".to_owned();
    let mut earlier_url = pull_request(1, PullRequestState::Open, true);
    earlier_url.key.repo = RepoKey {
        provider: Provider::Github,
        path: "owner/other".to_owned(),
    };
    earlier_url.url = "https://a.example.test/pull/1".to_owned();
    let expected = [
        "https://a.example.test/pull/1",
        "https://z.example.test/pull/1",
    ];

    for pull_requests in [
        vec![later_url.clone(), earlier_url.clone()],
        vec![earlier_url.clone(), later_url.clone()],
    ] {
        let report = build_report(
            &config,
            &dataset(pull_requests, Vec::new()),
            days_from_civil(2026, 8, 11),
        );
        let review = category_items(&report.items, Category::Review);
        assert_eq!(
            review
                .iter()
                .map(|item| item.url.as_str())
                .collect::<Vec<_>>(),
            expected
        );
    }
}

#[test]
fn invalid_required_timestamps_omit_items_and_add_contextual_warnings() {
    let config = config(2);
    let mut invalid_review = pull_request(1, PullRequestState::Open, true);
    invalid_review.created_at = "2026-08-11".to_owned();
    let mut invalid_retour = pull_request(2, PullRequestState::Open, false);
    invalid_retour.branch = "renovate/provider-update".to_owned();
    invalid_retour.feedback = vec![feedback("2026-08-11T99:00:00Z", FeedbackKind::Comment)];
    let mut invalid_no_ticket = pull_request(3, PullRequestState::Open, false);
    invalid_no_ticket.created_at = "2026-08-11T99:00:00Z".to_owned();
    let mut invalid_linear = issue("ABC-1", IssueState::Started, "2026-08-11T99:00:00Z");
    invalid_linear.branch_name.clear();
    let invalid_next = issue("ABC-2", IssueState::Unstarted, "2026-08-11");
    let source_warning = Warning {
        categories: vec![Category::Review],
        message: "provider partial result".to_owned(),
    };
    let dataset = Dataset {
        pull_requests: vec![invalid_review, invalid_retour, invalid_no_ticket],
        issues: vec![invalid_linear, invalid_next],
        warnings: vec![source_warning.clone()],
    };

    let report = build_report(&config, &dataset, days_from_civil(2026, 8, 11));

    assert!(report.items.is_empty());
    assert_eq!(report.warnings[0], source_warning);
    for (category, context) in [
        (Category::Review, "#1"),
        (Category::Retour, "#2"),
        (Category::Linear, "#3"),
        (Category::Linear, "ABC-1"),
        (Category::Suivant, "ABC-2"),
    ] {
        assert!(report.warnings.iter().any(|warning| {
            warning.categories.contains(&category) && warning.message.contains(context)
        }));
    }
}

fn config(next_count: usize) -> Config {
    Config {
        stale_days: 7,
        next_count,
        tracks: vec![Track {
            name: "Primary".to_owned(),
            teams: vec!["ABC".to_owned()],
            repos: vec![repo(Provider::Bitbucket, "owner/repo", true)],
        }],
    }
}

fn repo(provider: Provider, path: &str, requires_linear: bool) -> RepoConfig {
    RepoConfig {
        provider,
        path: path.to_owned(),
        requires_linear,
    }
}

fn pull_request(number: u64, state: PullRequestState, awaiting_review: bool) -> PullRequest {
    PullRequest {
        key: PullRequestKey {
            repo: RepoKey {
                provider: Provider::Bitbucket,
                path: "owner/repo".to_owned(),
            },
            number,
        },
        title: format!("Pull request {number}"),
        body: String::new(),
        branch: format!("feature/change-{number}"),
        destination: "main".to_owned(),
        url: format!("https://example.test/pull/{number}"),
        draft: false,
        state,
        created_at: "2026-08-05T09:00:00Z".to_owned(),
        updated_at: "2026-08-06T09:00:00Z".to_owned(),
        awaiting_review,
        feedback: Vec::new(),
    }
}

fn feedback(created_at: &str, kind: FeedbackKind) -> Feedback {
    Feedback {
        created_at: created_at.to_owned(),
        kind,
    }
}

fn issue(identifier: &str, state_type: IssueState, updated_at: &str) -> Issue {
    Issue {
        identifier: identifier.to_owned(),
        title: format!("Issue {identifier}"),
        url: format!("https://example.test/issue/{identifier}"),
        priority: 2,
        updated_at: updated_at.to_owned(),
        branch_name: format!("feature/{identifier}"),
        state_type,
        team_key: "ABC".to_owned(),
        project: Some("Project".to_owned()),
        labels: vec!["label".to_owned()],
        blockers: Vec::new(),
    }
}

fn dataset(pull_requests: Vec<PullRequest>, issues: Vec<Issue>) -> Dataset {
    Dataset {
        pull_requests,
        issues,
        warnings: Vec::new(),
    }
}

fn category_items(
    items: &[crate::model::ReportItem],
    category: Category,
) -> Vec<&crate::model::ReportItem> {
    items
        .iter()
        .filter(|item| item.category == category)
        .collect()
}

fn references(items: &[&crate::model::ReportItem]) -> Vec<String> {
    items.iter().map(|item| item.reference.clone()).collect()
}
