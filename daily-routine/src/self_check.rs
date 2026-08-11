use crate::config::{Config, Provider, RepoKey};
use crate::model::{
    Category, Feedback, Issue, IssueState, LinearReason, PullRequest, PullRequestKey,
    PullRequestState,
};
use crate::rules::{
    issue_linear_reasons, pr_linear_reasons, track_for_pr, uncorrelated_pr_reasons,
};
use crate::util::{days_from_civil, find_linear_id, percent_encode};

pub fn run() {
    check_config_parsing();
    check_correlation();
    check_six_linear_rules();
    check_track_attachment();
    check_requires_linear();
    check_category_order();
    check_days_from_civil();
    check_percent_encoding();
    println!("daily-routine self-check: ok");
}

fn check_config_parsing() {
    let config = config();

    assert_eq!(config.stale_days, 7);
    assert_eq!(config.next_count, 2);
    assert_eq!(config.tracks.len(), 3);
    assert!(config.tracks[0].repos[0].requires_linear);
    assert!(!config.tracks[2].repos[0].requires_linear);
    assert_eq!(config.unique_repos().len(), 3);
}

fn check_correlation() {
    let keys = ["ALP".to_owned(), "BET".to_owned()];

    assert_eq!(
        find_linear_id("ALP-12 Validate input", "feature/no-id", "", &keys),
        Some("ALP-12".to_owned())
    );
    assert_eq!(
        find_linear_id("Validate input", "feature/bet-34-validation", "", &keys),
        Some("BET-34".to_owned())
    );
    assert_eq!(
        find_linear_id("Validate input", "feature/no-ticket", "No issue", &keys),
        None
    );
}

fn check_six_linear_rules() {
    let mut issue = issue("ALP-42", IssueState::Started);
    let mut pull_request = pull_request(
        Provider::Github,
        "ExampleOrg/alpha",
        PullRequestState::Merged,
    );

    assert_eq!(
        pr_linear_reasons(&pull_request, &issue),
        [LinearReason::MergedIssueIncomplete]
    );

    pull_request.state = PullRequestState::Open;
    issue.state_type = IssueState::Backlog;
    assert_eq!(
        pr_linear_reasons(&pull_request, &issue),
        [LinearReason::OpenIssueNotStarted]
    );

    issue.state_type = IssueState::Started;
    issue.branch_name.clear();
    issue.updated_at = "2026-08-01T08:00:00Z".to_owned();
    issue.project = None;
    issue.labels.clear();
    issue.priority = 0;
    let reasons = issue_linear_reasons(&issue, false, days_from_civil(2026, 8, 11), 7);
    for reason in [
        LinearReason::StartedWithoutBranchOrPr,
        LinearReason::StartedStale,
        LinearReason::MissingProject,
        LinearReason::MissingLabel,
        LinearReason::MissingPriority,
    ] {
        assert!(reasons.contains(&reason));
    }

    assert_eq!(
        uncorrelated_pr_reasons(&pull_request, true),
        [LinearReason::OpenPrWithoutIssue]
    );
}

fn check_track_attachment() {
    let config = config();
    let mono = pull_request(Provider::Github, "ExampleOrg/alpha", PullRequestState::Open);
    assert_eq!(track_for_pr(&config, &mono, None).unwrap().track_index, 0);

    let shared = pull_request(
        Provider::Bitbucket,
        "ExampleOrg/shared",
        PullRequestState::Open,
    );
    let shared_issue = issue("BET-42", IssueState::Started);
    assert_eq!(
        track_for_pr(&config, &shared, Some(&shared_issue))
            .unwrap()
            .track_index,
        1
    );
    assert_eq!(track_for_pr(&config, &shared, None).unwrap().track_index, 0);

    let ticketless = pull_request(
        Provider::Github,
        "ExampleOrg/standalone",
        PullRequestState::Open,
    );
    assert_eq!(
        track_for_pr(&config, &ticketless, None)
            .unwrap()
            .track_index,
        2
    );
}

fn check_requires_linear() {
    let config = config();
    let ticketless = pull_request(
        Provider::Github,
        "ExampleOrg/standalone",
        PullRequestState::Open,
    );
    let selection = track_for_pr(&config, &ticketless, None).unwrap();

    assert!(!selection.requires_linear);
    assert!(uncorrelated_pr_reasons(&ticketless, selection.requires_linear).is_empty());
}

fn check_category_order() {
    let mut categories = [
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
}

fn check_days_from_civil() {
    assert_eq!(days_from_civil(1970, 1, 1), 0);
    assert_eq!(
        days_from_civil(2000, 3, 1) - days_from_civil(2000, 2, 28),
        2
    );
    assert_eq!(
        days_from_civil(1900, 3, 1) - days_from_civil(1900, 2, 28),
        1
    );
}

fn check_percent_encoding() {
    assert_eq!(percent_encode("Été quotidien"), "%C3%89t%C3%A9%20quotidien");
}

fn config() -> Config {
    Config::parse(
        r#"
            stale_days = 7
            next_count = 2

            [[tracks]]
            name = "Alpha"
            teams = ["ALP"]

            [[tracks.repos]]
            provider = "github"
            path = "ExampleOrg/alpha"

            [[tracks.repos]]
            provider = "bitbucket"
            path = "ExampleOrg/shared"

            [[tracks]]
            name = "Beta"
            teams = ["BET"]

            [[tracks.repos]]
            provider = "bitbucket"
            path = "ExampleOrg/shared"

            [[tracks]]
            name = "Standalone"
            teams = []

            [[tracks.repos]]
            provider = "github"
            path = "ExampleOrg/standalone"
            requires_linear = false
        "#,
    )
    .expect("the embedded self-check configuration must remain valid")
}

fn issue(identifier: &str, state_type: IssueState) -> Issue {
    Issue {
        identifier: identifier.to_owned(),
        title: "Validate daily state".to_owned(),
        url: format!("https://example.test/issues/{identifier}"),
        priority: 2,
        updated_at: "2026-08-10T08:00:00Z".to_owned(),
        branch_name: format!("feature/{}", identifier.to_ascii_lowercase()),
        state_type,
        team_key: identifier.split_once('-').unwrap().0.to_owned(),
        project: Some("Example project".to_owned()),
        labels: vec!["maintenance".to_owned()],
    }
}

fn pull_request(provider: Provider, path: &str, state: PullRequestState) -> PullRequest {
    PullRequest {
        key: PullRequestKey {
            repo: RepoKey {
                provider,
                path: path.to_owned(),
            },
            number: 42,
        },
        title: "ALP-42 Validate daily state".to_owned(),
        body: String::new(),
        branch: "feature/alp-42-validation".to_owned(),
        destination: "main".to_owned(),
        url: "https://example.test/pull/42".to_owned(),
        draft: false,
        state,
        created_at: "2026-08-09T08:00:00Z".to_owned(),
        updated_at: "2026-08-10T08:00:00Z".to_owned(),
        awaiting_review: false,
        feedback: Vec::<Feedback>::new(),
    }
}
