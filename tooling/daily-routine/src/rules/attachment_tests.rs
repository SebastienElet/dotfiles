use super::*;
use crate::config::{Config, Provider, RepoConfig, RepoKey};
use crate::model::{Issue, IssueState, PullRequest, PullRequestKey, PullRequestState};

#[test]
fn attachment_uses_the_issue_team_to_disambiguate_a_shared_repo() {
    let mut config = example_config();
    config.tracks[1].repos[0].requires_linear = false;
    let pr = pull_request(Provider::Bitbucket, "ExampleOrg/shared-app");

    assert_eq!(
        track_for_pr(&config, &pr, Some(&issue("OPS"))),
        Some(TrackSelection {
            track_index: 1,
            requires_linear: false,
        })
    );
}

#[test]
fn attachment_selects_application_for_a_repo_declared_only_in_that_track() {
    let mut config = example_config();
    config.tracks[0].repos.push(RepoConfig {
        provider: Provider::Github,
        path: "ExampleOrg/app-service".to_owned(),
        requires_linear: true,
    });
    let pr = pull_request(Provider::Github, "ExampleOrg/app-service");

    assert_eq!(
        track_for_pr(&config, &pr, None),
        Some(TrackSelection {
            track_index: 0,
            requires_linear: true,
        })
    );
}

#[test]
fn attachment_falls_back_to_the_first_track_for_a_shared_repo_without_an_issue() {
    let mut config = example_config();
    config.tracks[1].repos[0].requires_linear = false;
    let pr = pull_request(Provider::Bitbucket, "ExampleOrg/shared-app");

    assert_eq!(
        track_for_pr(&config, &pr, None),
        Some(TrackSelection {
            track_index: 0,
            requires_linear: true,
        })
    );
}

#[test]
fn attachment_rejects_an_unknown_repo_path() {
    let config = example_config();
    let pr = pull_request(Provider::Bitbucket, "ExampleOrg/unknown");

    assert_eq!(track_for_pr(&config, &pr, None), None);
}

#[test]
fn attachment_requires_the_repo_provider_to_match() {
    let config = example_config();
    let pr = pull_request(Provider::Github, "ExampleOrg/shared-app");

    assert_eq!(track_for_pr(&config, &pr, None), None);
}

#[test]
fn attachment_keeps_a_repo_without_linear_tickets() {
    let config = example_config();
    let pr = pull_request(Provider::Github, "ExampleOrg/standalone");

    assert_eq!(
        track_for_pr(&config, &pr, None),
        Some(TrackSelection {
            track_index: 2,
            requires_linear: false,
        })
    );
}

#[test]
fn attachment_maps_issue_teams_case_insensitively() {
    let config = example_config();

    assert_eq!(track_for_issue(&config, &issue("OPS")), Some(1));
    assert_eq!(track_for_issue(&config, &issue("ops")), Some(1));
    assert_eq!(track_for_issue(&config, &issue("UNKNOWN")), None);
}

fn example_config() -> Config {
    Config::parse(include_str!("../../config.example.toml")).unwrap()
}

fn pull_request(provider: Provider, path: &str) -> PullRequest {
    PullRequest {
        key: PullRequestKey {
            repo: RepoKey {
                provider,
                path: path.to_owned(),
            },
            number: 42,
        },
        title: "Ship normalized models".to_owned(),
        body: String::new(),
        branch: "feature/normalized-models".to_owned(),
        destination: "main".to_owned(),
        url: "https://example.test/pull/42".to_owned(),
        draft: false,
        state: PullRequestState::Open,
        created_at: "2026-08-10T08:00:00Z".to_owned(),
        updated_at: "2026-08-11T08:00:00Z".to_owned(),
        awaiting_review: false,
        feedback: Vec::new(),
    }
}

fn issue(team_key: &str) -> Issue {
    Issue {
        identifier: "OPS-42".to_owned(),
        title: "Normalize models".to_owned(),
        url: "https://example.test/issue/OPS-42".to_owned(),
        priority: 2,
        updated_at: "2026-08-11T08:00:00Z".to_owned(),
        branch_name: "feature/normalized-models".to_owned(),
        state_type: IssueState::Started,
        team_key: team_key.to_owned(),
        project: Some("Platform".to_owned()),
        labels: Vec::new(),
        blockers: Vec::new(),
    }
}
