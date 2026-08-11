use crate::command;
use crate::config::{Config, Provider, RepoKey};
use crate::model::{
    Category, Feedback, FeedbackKind, Identity, PullRequest, PullRequestKey, PullRequestState,
    Warning,
};
use crate::parallel;
use serde::Deserialize;
use serde::de::DeserializeOwned;
use std::collections::HashSet;
use std::error::Error;

const REVIEW_DETAILS_QUERY: &str = "query($owner:String!,$name:String!,$number:Int!){repository(owner:$owner,name:$name){pullRequest(number:$number){reviewDecision reviewThreads(first:50){nodes {isResolved isOutdated comments(first:1){nodes {author {login} body url createdAt}}}}}}}";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GithubCollection {
    pub identity: Option<Identity>,
    pub pull_requests: Vec<PullRequest>,
    pub warnings: Vec<Warning>,
}

#[derive(Clone)]
struct Repository {
    key: RepoKey,
    owner: String,
}

struct DetailResult {
    pull_request: PullRequest,
    warning: Option<Warning>,
}

#[derive(Default)]
enum RequiredNullable<T> {
    #[default]
    Missing,
    Present(Option<T>),
}

impl<'de, T> Deserialize<'de> for RequiredNullable<T>
where
    T: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Option::deserialize(deserializer).map(Self::Present)
    }
}

impl<T> RequiredNullable<T> {
    fn into_option(self, field: &str) -> Result<Option<T>, Box<dyn Error>> {
        match self {
            Self::Missing => {
                Err(format!("GitHub response is missing required field {field}").into())
            }
            Self::Present(value) => Ok(value),
        }
    }
}

#[derive(Deserialize)]
struct GithubIdentity {
    id: u64,
    login: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct GithubTeam {
    organization: String,
    slug: String,
}

impl GithubTeam {
    fn full_name(&self) -> String {
        format!("{}/{}", self.organization, self.slug)
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct AuthoredPullRequest {
    number: u64,
    title: String,
    url: String,
    head_ref_name: String,
    #[serde(default)]
    body: RequiredNullable<String>,
    updated_at: String,
    created_at: String,
    #[serde(default)]
    merged_at: RequiredNullable<String>,
    is_draft: bool,
    state: GithubPullRequestState,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ReviewPullRequest {
    number: u64,
    title: String,
    url: String,
    head_ref_name: String,
    updated_at: String,
    created_at: String,
    is_draft: bool,
    #[serde(default)]
    author: RequiredNullable<GithubAuthor>,
}

#[derive(Deserialize)]
struct GithubAuthor {
    login: String,
}

#[derive(Clone, Copy, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
enum GithubPullRequestState {
    Open,
    Merged,
}

#[derive(Deserialize)]
struct DetailResponse {
    #[serde(default)]
    errors: Option<Vec<serde_json::Value>>,
    data: DetailData,
}

#[derive(Deserialize)]
struct DetailData {
    repository: DetailRepository,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct DetailRepository {
    pull_request: DetailPullRequest,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct DetailPullRequest {
    #[serde(default)]
    review_decision: RequiredNullable<ReviewDecision>,
    review_threads: ReviewThreads,
}

#[derive(Clone, Copy, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum ReviewDecision {
    Approved,
    ChangesRequested,
    ReviewRequired,
}

#[derive(Deserialize)]
struct ReviewThreads {
    nodes: Vec<ReviewThread>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ReviewThread {
    is_resolved: bool,
    is_outdated: bool,
    comments: ReviewComments,
}

#[derive(Deserialize)]
struct ReviewComments {
    nodes: Vec<ReviewComment>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ReviewComment {
    #[serde(default)]
    author: RequiredNullable<GithubAuthor>,
    body: String,
    url: String,
    created_at: String,
}

pub fn collect(config: &Config) -> GithubCollection {
    collect_with(config, |program, args| {
        command::run(program, args).map_err(|error| error.to_string())
    })
}

fn collect_with<R>(config: &Config, run: R) -> GithubCollection
where
    R: Fn(&str, &[String]) -> Result<String, String> + Sync,
{
    let repositories = configured_repositories(config);
    if repositories.is_empty() {
        return GithubCollection {
            identity: None,
            pull_requests: Vec::new(),
            warnings: Vec::new(),
        };
    }

    let mut warnings = Vec::new();
    let identity_args = strings(&["api", "user", "--jq", "{login,id}"]);
    let identity = match invoke_json::<GithubIdentity, _>(&run, &identity_args) {
        Ok(identity) => Some(Identity {
            id: identity.id.to_string(),
            name: identity.login,
            email: None,
        }),
        Err(error) => {
            warnings.push(warning(
                &[Category::Retour],
                format!(
                    "GitHub identity collection failed; review-thread comments were omitted: {error}"
                ),
            ));
            None
        }
    };

    let team_args = strings(&[
        "api",
        "user/teams",
        "--jq",
        r#".[] | "\(.organization.login)/\(.slug)""#,
    ]);
    let teams = match run("gh", &team_args)
        .map_err(|error| format!("command failed: {error}"))
        .and_then(|source| parse_teams(&source).map_err(|error| error.to_string()))
    {
        Ok(teams) => teams,
        Err(error) => {
            warnings.push(warning(
                &[Category::Review],
                format!("GitHub team collection failed: {error}"),
            ));
            Vec::new()
        }
    };

    let mut authored_open = Vec::new();
    let mut authored_merged = Vec::new();
    let mut review_candidates = Vec::new();

    for repository in &repositories {
        collect_authored_list(
            &run,
            repository,
            AuthoredListRequest::Open,
            &mut authored_open,
            &mut warnings,
        );
        collect_authored_list(
            &run,
            repository,
            AuthoredListRequest::Merged,
            &mut authored_merged,
            &mut warnings,
        );
        collect_review_list(
            &run,
            repository,
            "review-requested:@me".to_owned(),
            "personal review-requested PR list",
            &mut review_candidates,
            &mut warnings,
        );
        for team in matching_teams(&teams, &repository.owner) {
            collect_review_list(
                &run,
                repository,
                format!("team-review-requested:{team}"),
                "team review-requested PR list",
                &mut review_candidates,
                &mut warnings,
            );
        }
    }

    deduplicate(&mut authored_open);
    let mut authored_keys = authored_open
        .iter()
        .map(|pull_request| pull_request.key.clone())
        .collect::<HashSet<_>>();
    authored_merged.retain(|pull_request| authored_keys.insert(pull_request.key.clone()));
    review_candidates
        .retain(|pull_request| !pull_request.draft && !authored_keys.contains(&pull_request.key));
    deduplicate(&mut review_candidates);

    let login = identity.as_ref().map(|identity| identity.name.as_str());
    let detailed = parallel::bounded_map(&authored_open, |pull_request| {
        collect_details(&run, pull_request, login)
    });
    let mut pull_requests = Vec::new();
    for result in detailed {
        pull_requests.push(result.pull_request);
        warnings.extend(result.warning);
    }
    pull_requests.extend(authored_merged);
    pull_requests.extend(review_candidates);

    GithubCollection {
        identity,
        pull_requests,
        warnings,
    }
}

fn configured_repositories(config: &Config) -> Vec<Repository> {
    config
        .unique_repos()
        .into_iter()
        .filter(|repo| repo.provider == Provider::Github)
        .filter_map(|repo| {
            repository_parts(&repo.path)
                .ok()
                .map(|(owner, _name)| Repository {
                    key: RepoKey {
                        provider: repo.provider,
                        path: repo.path.clone(),
                    },
                    owner: owner.to_owned(),
                })
        })
        .collect()
}

#[derive(Clone, Copy)]
enum AuthoredListRequest {
    Open,
    Merged,
}

impl AuthoredListRequest {
    fn args(self, repository: &Repository) -> Vec<String> {
        let (state, fields) = match self {
            Self::Open => (
                "open",
                "number,title,url,headRefName,body,updatedAt,createdAt,isDraft,state",
            ),
            Self::Merged => (
                "merged",
                "number,title,url,headRefName,body,updatedAt,createdAt,mergedAt,isDraft,state",
            ),
        };
        strings(&[
            "pr",
            "list",
            "--repo",
            &repository.key.path,
            "--author",
            "@me",
            "--state",
            state,
            "--limit",
            "50",
            "--json",
            fields,
        ])
    }

    const fn expected_state(self) -> PullRequestState {
        match self {
            Self::Open => PullRequestState::Open,
            Self::Merged => PullRequestState::Merged,
        }
    }

    const fn categories(self) -> &'static [Category] {
        match self {
            Self::Open => &[Category::Retour, Category::Linear],
            Self::Merged => &[Category::Linear],
        }
    }

    const fn label(self) -> &'static str {
        match self {
            Self::Open => "open authored PR list",
            Self::Merged => "merged authored PR list",
        }
    }
}

fn collect_authored_list<R>(
    run: &R,
    repository: &Repository,
    request: AuthoredListRequest,
    destination: &mut Vec<PullRequest>,
    warnings: &mut Vec<Warning>,
) where
    R: Fn(&str, &[String]) -> Result<String, String> + Sync,
{
    let result = run("gh", &request.args(repository))
        .map_err(|error| format!("command failed: {error}"))
        .and_then(|source| {
            parse_authored_list(&source, &repository.key)
                .map_err(|error| format!("invalid JSON: {error}"))
        })
        .and_then(|pull_requests| {
            if pull_requests
                .iter()
                .all(|pull_request| pull_request.state == request.expected_state())
            {
                Ok(pull_requests)
            } else {
                Err(format!(
                    "response contains a PR outside the requested {} state",
                    match request.expected_state() {
                        PullRequestState::Open => "OPEN",
                        PullRequestState::Merged => "MERGED",
                    }
                ))
            }
        });

    match result {
        Ok(pull_requests) => destination.extend(pull_requests),
        Err(error) => warnings.push(warning(
            request.categories(),
            format!(
                "GitHub {} failed for {}: {error}",
                request.label(),
                repository.key.path
            ),
        )),
    }
}

fn collect_review_list<R>(
    run: &R,
    repository: &Repository,
    search: String,
    label: &str,
    destination: &mut Vec<PullRequest>,
    warnings: &mut Vec<Warning>,
) where
    R: Fn(&str, &[String]) -> Result<String, String> + Sync,
{
    let args = strings(&[
        "pr",
        "list",
        "--repo",
        &repository.key.path,
        "--search",
        &search,
        "--state",
        "open",
        "--limit",
        "50",
        "--json",
        "number,title,url,headRefName,updatedAt,createdAt,isDraft,author",
    ]);
    match run("gh", &args)
        .map_err(|error| format!("command failed: {error}"))
        .and_then(|source| {
            parse_review_list(&source, &repository.key)
                .map_err(|error| format!("invalid JSON: {error}"))
        }) {
        Ok(pull_requests) => destination.extend(pull_requests),
        Err(error) => warnings.push(warning(
            &[Category::Review],
            format!("GitHub {label} failed for {}: {error}", repository.key.path),
        )),
    }
}

fn collect_details<R>(run: &R, pull_request: &PullRequest, login: Option<&str>) -> DetailResult
where
    R: Fn(&str, &[String]) -> Result<String, String> + Sync,
{
    let (owner, name) = repository_parts(&pull_request.key.repo.path)
        .expect("validated configuration always has a GitHub owner and repository name");
    let number = pull_request.key.number.to_string();
    let args = strings(&[
        "api",
        "graphql",
        "-f",
        &format!("query={REVIEW_DETAILS_QUERY}"),
        "-F",
        &format!("owner={owner}"),
        "-F",
        &format!("name={name}"),
        "-F",
        &format!("number={number}"),
    ]);
    let result = run("gh", &args)
        .map_err(|error| format!("command failed: {error}"))
        .and_then(|source| {
            parse_details(&source, login, &pull_request.updated_at)
                .map_err(|error| format!("invalid JSON: {error}"))
        });

    let mut pull_request = pull_request.clone();
    match result {
        Ok(feedback) => {
            pull_request.feedback = feedback;
            DetailResult {
                pull_request,
                warning: None,
            }
        }
        Err(error) => DetailResult {
            warning: Some(warning(
                &[Category::Retour],
                format!(
                    "GitHub review details failed for {}#{}: {error}",
                    pull_request.key.repo.path, pull_request.key.number
                ),
            )),
            pull_request,
        },
    }
}

fn parse_authored_list(source: &str, repo: &RepoKey) -> Result<Vec<PullRequest>, Box<dyn Error>> {
    let response: Vec<AuthoredPullRequest> = serde_json::from_str(source)?;
    response
        .into_iter()
        .map(|pull_request| {
            let body = pull_request
                .body
                .into_option("[].body")?
                .unwrap_or_default();
            let state = PullRequestState::from(pull_request.state);
            let updated_at = match state {
                PullRequestState::Open => pull_request.updated_at,
                PullRequestState::Merged => pull_request
                    .merged_at
                    .into_option("[].mergedAt")?
                    .ok_or("GitHub merged pull request has a null mergedAt")?,
            };
            Ok(PullRequest {
                key: PullRequestKey {
                    repo: repo.clone(),
                    number: pull_request.number,
                },
                title: pull_request.title,
                body,
                branch: pull_request.head_ref_name,
                destination: String::new(),
                url: pull_request.url,
                draft: pull_request.is_draft,
                state,
                created_at: pull_request.created_at,
                updated_at,
                awaiting_review: false,
                feedback: Vec::new(),
            })
        })
        .collect()
}

fn parse_review_list(source: &str, repo: &RepoKey) -> Result<Vec<PullRequest>, Box<dyn Error>> {
    let response: Vec<ReviewPullRequest> = serde_json::from_str(source)?;
    response
        .into_iter()
        .map(|pull_request| {
            let author = pull_request.author.into_option("[].author")?;
            let _author_login = author.map(|author| author.login);
            Ok(PullRequest {
                key: PullRequestKey {
                    repo: repo.clone(),
                    number: pull_request.number,
                },
                title: pull_request.title,
                body: String::new(),
                branch: pull_request.head_ref_name,
                destination: String::new(),
                url: pull_request.url,
                draft: pull_request.is_draft,
                state: PullRequestState::Open,
                created_at: pull_request.created_at,
                updated_at: pull_request.updated_at,
                awaiting_review: true,
                feedback: Vec::new(),
            })
        })
        .collect()
}

fn parse_teams(source: &str) -> Result<Vec<GithubTeam>, Box<dyn Error>> {
    source
        .lines()
        .filter(|line| !line.is_empty())
        .map(|line| {
            let Some((organization, slug)) = line.split_once('/') else {
                return Err(
                    format!("invalid GitHub team {line:?}, expected organization/slug").into(),
                );
            };
            if organization.trim().is_empty()
                || slug.trim().is_empty()
                || organization != organization.trim()
                || slug != slug.trim()
                || slug.contains('/')
            {
                return Err(
                    format!("invalid GitHub team {line:?}, expected organization/slug").into(),
                );
            }
            Ok(GithubTeam {
                organization: organization.to_owned(),
                slug: slug.to_owned(),
            })
        })
        .collect()
}

fn matching_teams(teams: &[GithubTeam], owner: &str) -> Vec<String> {
    teams
        .iter()
        .filter(|team| team.organization.eq_ignore_ascii_case(owner))
        .map(GithubTeam::full_name)
        .collect()
}

fn parse_details(
    source: &str,
    login: Option<&str>,
    fallback_at: &str,
) -> Result<Vec<Feedback>, Box<dyn Error>> {
    let response: DetailResponse = serde_json::from_str(source)?;
    if let Some(errors) = response.errors
        && !errors.is_empty()
    {
        return Err(format!("GitHub GraphQL errors: {errors:?}").into());
    }
    let pull_request = response.data.repository.pull_request;
    let decision = pull_request
        .review_decision
        .into_option("data.repository.pullRequest.reviewDecision")?;
    let mut feedback = Vec::new();
    if decision == Some(ReviewDecision::ChangesRequested) {
        feedback.push(Feedback {
            created_at: fallback_at.to_owned(),
            kind: FeedbackKind::ChangesRequested,
        });
    }

    for thread in pull_request.review_threads.nodes {
        let _is_outdated = thread.is_outdated;
        if thread.is_resolved {
            continue;
        }
        let Some(comment) = thread.comments.nodes.into_iter().next() else {
            continue;
        };
        let author = comment.author.into_option(
            "data.repository.pullRequest.reviewThreads.nodes[].comments.nodes[].author",
        )?;
        let _body = comment.body;
        let _url = comment.url;
        if let Some(login) = login
            && author
                .as_ref()
                .is_some_and(|author| author.login.eq_ignore_ascii_case(login))
        {
            continue;
        }
        if login.is_some() {
            feedback.push(Feedback {
                created_at: comment.created_at,
                kind: FeedbackKind::Comment,
            });
        }
    }

    Ok(feedback)
}

fn invoke_json<T, R>(run: &R, args: &[String]) -> Result<T, String>
where
    T: DeserializeOwned,
    R: Fn(&str, &[String]) -> Result<String, String> + Sync,
{
    let source = run("gh", args).map_err(|error| format!("command failed: {error}"))?;
    serde_json::from_str(&source).map_err(|error| format!("invalid JSON: {error}"))
}

fn deduplicate(pull_requests: &mut Vec<PullRequest>) {
    let mut seen = HashSet::new();
    pull_requests.retain(|pull_request| seen.insert(pull_request.key.clone()));
}

fn repository_parts(path: &str) -> Result<(&str, &str), Box<dyn Error>> {
    let Some((owner, name)) = path.split_once('/') else {
        return Err(format!("invalid GitHub repository path {path:?}").into());
    };
    if owner.is_empty() || name.is_empty() || name.contains('/') {
        return Err(format!("invalid GitHub repository path {path:?}").into());
    }
    Ok((owner, name))
}

fn warning(categories: &[Category], message: String) -> Warning {
    Warning {
        categories: categories.to_vec(),
        message,
    }
}

fn strings(values: &[&str]) -> Vec<String> {
    values.iter().map(|value| (*value).to_owned()).collect()
}

impl From<GithubPullRequestState> for PullRequestState {
    fn from(state: GithubPullRequestState) -> Self {
        match state {
            GithubPullRequestState::Open => Self::Open,
            GithubPullRequestState::Merged => Self::Merged,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{Config, Provider, RepoKey};
    use crate::model::{Category, FeedbackKind, PullRequestState};
    use std::sync::Mutex;

    const VIEWER_RESPONSE: &str = include_str!("fixtures/github-viewer.json");
    const TEAM_RESPONSE: &str = include_str!("fixtures/github-teams.txt");
    const OPEN_RESPONSE: &str = include_str!("fixtures/github-open.json");
    const MERGED_RESPONSE: &str = include_str!("fixtures/github-merged.json");
    const REVIEW_RESPONSE: &str = include_str!("fixtures/github-review.json");
    const DETAIL_RESPONSE: &str = include_str!("fixtures/github-review-details.json");

    fn repo() -> RepoKey {
        RepoKey {
            provider: Provider::Github,
            path: "ExampleOrg/shared-app".to_owned(),
        }
    }

    fn config() -> Config {
        Config::parse(
            r#"
                stale_days = 7
                next_count = 3

                [[tracks]]
                name = "Application"
                teams = ["APP"]

                  [[tracks.repos]]
                  provider = "github"
                  path = "ExampleOrg/shared-app"

                [[tracks]]
                name = "Operations"
                teams = ["OPS"]

                  [[tracks.repos]]
                  provider = "github"
                  path = "ExampleOrg/shared-app"

                  [[tracks.repos]]
                  provider = "bitbucket"
                  path = "ExampleOrg/standalone"
            "#,
        )
        .unwrap()
    }

    #[test]
    fn parses_authored_lists_with_nullable_body_and_merged_event_time() {
        let open = parse_authored_list(OPEN_RESPONSE, &repo()).unwrap();
        let merged = parse_authored_list(MERGED_RESPONSE, &repo()).unwrap();

        assert_eq!(open.len(), 2);
        assert_eq!(open[0].state, PullRequestState::Open);
        assert_eq!(open[0].body, "APP-201 tracks validation.");
        assert_eq!(open[1].body, "");
        assert!(open[1].draft);
        assert_eq!(merged.len(), 1);
        assert_eq!(merged[0].state, PullRequestState::Merged);
        assert_eq!(merged[0].updated_at, "2026-08-08T08:00:00Z");
    }

    #[test]
    fn rejects_missing_required_authored_fields() {
        let mut response: serde_json::Value = serde_json::from_str(OPEN_RESPONSE).unwrap();
        response[0].as_object_mut().unwrap().remove("body");

        assert!(parse_authored_list(&response.to_string(), &repo()).is_err());
    }

    #[test]
    fn review_lists_accept_null_authors_and_keep_draft_information() {
        let pull_requests = parse_review_list(REVIEW_RESPONSE, &repo()).unwrap();

        assert_eq!(pull_requests.len(), 2);
        assert_eq!(pull_requests[0].key.number, 301);
        assert!(pull_requests[0].awaiting_review);
        assert!(!pull_requests[0].draft);
        assert!(pull_requests[1].draft);
    }

    #[test]
    fn filters_teams_by_owner_organization_case_insensitively() {
        let teams = parse_teams(TEAM_RESPONSE).unwrap();

        assert_eq!(
            matching_teams(&teams, "ExampleOrg"),
            ["ExampleOrg/reviewers", "exampleorg/platform"]
        );
        assert_eq!(
            matching_teams(&teams, "UnconfiguredOrg"),
            Vec::<String>::new()
        );
    }

    #[test]
    fn retains_unresolved_external_threads_including_outdated_and_null_authors() {
        let feedback = parse_details(
            DETAIL_RESPONSE,
            Some("example-user"),
            "2026-08-10T09:00:00Z",
        )
        .unwrap();

        assert_eq!(feedback.len(), 4);
        assert_eq!(feedback[0].kind, FeedbackKind::ChangesRequested);
        assert_eq!(feedback[0].created_at, "2026-08-10T09:00:00Z");
        assert_eq!(feedback[1].created_at, "2026-08-05T08:00:00Z");
        assert_eq!(feedback[2].created_at, "2026-08-08T08:00:00Z");
        assert_eq!(feedback[3].created_at, "2026-08-09T08:00:00Z");
        assert!(
            feedback[1..]
                .iter()
                .all(|entry| entry.kind == FeedbackKind::Comment)
        );
    }

    #[test]
    fn changes_requested_remains_available_when_identity_collection_fails() {
        let feedback = parse_details(DETAIL_RESPONSE, None, "2026-08-10T09:00:00Z").unwrap();

        assert_eq!(feedback.len(), 1);
        assert_eq!(feedback[0].kind, FeedbackKind::ChangesRequested);
    }

    #[test]
    fn rejects_partial_graphql_data_when_errors_are_present() {
        let mut response: serde_json::Value = serde_json::from_str(DETAIL_RESPONSE).unwrap();
        response["errors"] = serde_json::json!([{
            "message": "One review thread could not be loaded"
        }]);

        assert!(
            parse_details(
                &response.to_string(),
                Some("example-user"),
                "2026-08-10T09:00:00Z"
            )
            .is_err()
        );
    }

    #[test]
    fn accepts_explicitly_empty_graphql_errors() {
        let mut response: serde_json::Value = serde_json::from_str(DETAIL_RESPONSE).unwrap();
        response["errors"] = serde_json::json!([]);

        let feedback = parse_details(
            &response.to_string(),
            Some("example-user"),
            "2026-08-10T09:00:00Z",
        )
        .unwrap();

        assert_eq!(feedback.len(), 4);
    }

    #[test]
    fn collection_resolves_identity_and_teams_once_and_deduplicates_review_requests() {
        let calls = Mutex::new(Vec::new());
        let collection = collect_with(&config(), |program, args| {
            calls
                .lock()
                .unwrap()
                .push((program.to_owned(), args.to_vec()));

            if args == strings(&["api", "user", "--jq", "{login,id}"]) {
                return Ok(VIEWER_RESPONSE.to_owned());
            }
            if args
                == strings(&[
                    "api",
                    "user/teams",
                    "--jq",
                    r#".[] | "\(.organization.login)/\(.slug)""#,
                ])
            {
                return Ok(TEAM_RESPONSE.to_owned());
            }
            if args.first().map(String::as_str) == Some("api") {
                return Ok(DETAIL_RESPONSE.to_owned());
            }
            if args.iter().any(|arg| arg == "--author") && args.iter().any(|arg| arg == "open") {
                return Ok(OPEN_RESPONSE.to_owned());
            }
            if args.iter().any(|arg| arg == "--author") && args.iter().any(|arg| arg == "merged") {
                return Ok("[]".to_owned());
            }
            if args.iter().any(|arg| arg == "review-requested:@me") {
                return Ok(REVIEW_RESPONSE.to_owned());
            }
            if args
                .iter()
                .any(|arg| arg == "team-review-requested:ExampleOrg/reviewers")
            {
                return Ok(REVIEW_RESPONSE.to_owned());
            }
            if args
                .iter()
                .any(|arg| arg == "team-review-requested:exampleorg/platform")
            {
                return Ok("[]".to_owned());
            }
            Err(format!("unexpected command: {program} {args:?}"))
        });

        assert_eq!(collection.identity.as_ref().unwrap().id, "424242");
        assert!(collection.warnings.is_empty());
        assert_eq!(collection.pull_requests.len(), 3);
        assert_eq!(
            collection
                .pull_requests
                .iter()
                .filter(|pull_request| pull_request.awaiting_review)
                .map(|pull_request| pull_request.key.number)
                .collect::<Vec<_>>(),
            [301]
        );

        let calls = calls.into_inner().unwrap();
        assert_eq!(
            calls
                .iter()
                .filter(|(_, args)| args == &strings(&["api", "user", "--jq", "{login,id}"]))
                .count(),
            1
        );
        assert_eq!(
            calls
                .iter()
                .filter(|(_, args)| args.get(1).map(String::as_str) == Some("user/teams"))
                .count(),
            1
        );
        assert!(!calls.iter().any(|(_, args)| {
            args.iter()
                .any(|arg| arg.contains("team-review-requested:OtherOrg/unrelated"))
        }));
        assert!(calls.iter().any(|(_, args)| {
            args == &strings(&[
                "pr",
                "list",
                "--repo",
                "ExampleOrg/shared-app",
                "--author",
                "@me",
                "--state",
                "open",
                "--limit",
                "50",
                "--json",
                "number,title,url,headRefName,body,updatedAt,createdAt,isDraft,state",
            ])
        }));
        assert!(calls.iter().any(|(_, args)| {
            args.first().map(String::as_str) == Some("api")
                && args.get(1).map(String::as_str) == Some("graphql")
                && args.contains(&format!("query={REVIEW_DETAILS_QUERY}"))
                && args.contains(&"owner=ExampleOrg".to_owned())
                && args.contains(&"name=shared-app".to_owned())
                && args.contains(&"number=201".to_owned())
        }));
    }

    #[test]
    fn collection_skips_every_gh_call_without_a_configured_github_repo() {
        let config = Config::parse(
            r#"
                stale_days = 7
                next_count = 3

                [[tracks]]
                name = "Application"
                teams = ["APP"]

                  [[tracks.repos]]
                  provider = "bitbucket"
                  path = "ExampleOrg/shared-app"
            "#,
        )
        .unwrap();

        let collection = collect_with(&config, |_program, _args| {
            panic!("GitHub must not be called outside configured GitHub repositories")
        });

        assert_eq!(
            collection,
            GithubCollection {
                identity: None,
                pull_requests: Vec::new(),
                warnings: Vec::new(),
            }
        );
    }

    #[test]
    fn command_failures_only_degrade_their_consuming_categories() {
        let collection = collect_with(&config(), |_program, args| {
            if args == strings(&["api", "user", "--jq", "{login,id}"]) {
                return Err("identity unavailable".to_owned());
            }
            if args.get(1).map(String::as_str) == Some("user/teams") {
                return Err("teams unavailable".to_owned());
            }
            if args.iter().any(|arg| arg == "--author") && args.iter().any(|arg| arg == "open") {
                return Err("open list unavailable".to_owned());
            }
            if args.iter().any(|arg| arg == "--author") && args.iter().any(|arg| arg == "merged") {
                return Err("merged list unavailable".to_owned());
            }
            Err("review list unavailable".to_owned())
        });

        assert!(collection.pull_requests.is_empty());
        assert_eq!(collection.warnings.len(), 5);
        assert!(collection.warnings.iter().any(|warning| {
            warning.categories == [Category::Retour]
                && warning.message.contains("identity unavailable")
        }));
        assert!(collection.warnings.iter().any(|warning| {
            warning.categories == [Category::Review]
                && warning.message.contains("teams unavailable")
        }));
        assert!(collection.warnings.iter().any(|warning| {
            warning.categories == [Category::Retour, Category::Linear]
                && warning.message.contains("open list unavailable")
        }));
        assert!(collection.warnings.iter().any(|warning| {
            warning.categories == [Category::Linear]
                && warning.message.contains("merged list unavailable")
        }));
        assert!(collection.warnings.iter().any(|warning| {
            warning.categories == [Category::Review]
                && warning.message.contains("review list unavailable")
        }));
    }

    #[test]
    fn detail_failure_warns_retour_and_preserves_the_open_pull_request() {
        let collection = collect_with(&config(), |_program, args| {
            if args == strings(&["api", "user", "--jq", "{login,id}"]) {
                return Ok(VIEWER_RESPONSE.to_owned());
            }
            if args.get(1).map(String::as_str) == Some("user/teams") {
                return Ok("".to_owned());
            }
            if args.first().map(String::as_str) == Some("api") {
                return Err("detail unavailable".to_owned());
            }
            if args.iter().any(|arg| arg == "--author") && args.iter().any(|arg| arg == "open") {
                return Ok(OPEN_RESPONSE.to_owned());
            }
            Ok("[]".to_owned())
        });

        assert_eq!(collection.pull_requests.len(), 2);
        assert!(collection.pull_requests.iter().all(|pull_request| {
            pull_request.state == PullRequestState::Open && pull_request.feedback.is_empty()
        }));
        assert_eq!(collection.warnings.len(), 2);
        assert!(collection.warnings.iter().all(|warning| {
            warning.categories == [Category::Retour]
                && warning.message.contains("detail unavailable")
        }));
    }

    #[test]
    fn partial_graphql_response_preserves_pr_without_false_feedback() {
        let mut open_response: serde_json::Value = serde_json::from_str(OPEN_RESPONSE).unwrap();
        open_response.as_array_mut().unwrap().truncate(1);
        let open_response = open_response.to_string();
        let mut detail_response: serde_json::Value = serde_json::from_str(DETAIL_RESPONSE).unwrap();
        detail_response["errors"] = serde_json::json!([{
            "message": "One review thread could not be loaded"
        }]);
        let detail_response = detail_response.to_string();

        let collection = collect_with(&config(), |_program, args| {
            if args == strings(&["api", "user", "--jq", "{login,id}"]) {
                return Ok(VIEWER_RESPONSE.to_owned());
            }
            if args.get(1).map(String::as_str) == Some("user/teams") {
                return Ok(String::new());
            }
            if args.first().map(String::as_str) == Some("api") {
                return Ok(detail_response.clone());
            }
            if args.iter().any(|arg| arg == "--author") && args.iter().any(|arg| arg == "open") {
                return Ok(open_response.clone());
            }
            Ok("[]".to_owned())
        });

        assert_eq!(collection.pull_requests.len(), 1);
        assert!(collection.pull_requests[0].feedback.is_empty());
        assert_eq!(collection.warnings.len(), 1);
        assert_eq!(collection.warnings[0].categories, [Category::Retour]);
        assert!(collection.warnings[0].message.contains("GraphQL errors"));
    }
}
