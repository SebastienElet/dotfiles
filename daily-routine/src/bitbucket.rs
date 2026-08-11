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

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BitbucketCollection {
    pub identity: Option<Identity>,
    pub pull_requests: Vec<PullRequest>,
    pub warnings: Vec<Warning>,
}

#[derive(Clone)]
struct Repository {
    key: RepoKey,
    workspace: String,
    short_name: String,
}

struct DetailResult {
    pull_request: PullRequest,
    warnings: Vec<Warning>,
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
                Err(format!("Bitbucket response is missing required field {field}").into())
            }
            Self::Present(value) => Ok(value),
        }
    }
}

#[derive(Deserialize)]
struct PullRequestListResponse {
    pull_requests: Vec<BitbucketPullRequest>,
}

#[derive(Deserialize)]
struct BitbucketPullRequest {
    id: u64,
    title: String,
    #[serde(default)]
    description: RequiredNullable<String>,
    state: BitbucketPullRequestState,
    draft: bool,
    created_on: String,
    updated_on: String,
    author: BitbucketUser,
    source: BitbucketSource,
    destination: BitbucketDestination,
    links: BitbucketLinks,
    #[serde(default)]
    reviewers: RequiredNullable<serde_json::Value>,
}

#[derive(Deserialize)]
#[serde(rename_all = "UPPERCASE")]
enum BitbucketPullRequestState {
    Open,
    Merged,
}

#[derive(Deserialize)]
struct BitbucketUser {
    account_id: String,
    display_name: String,
}

#[derive(Deserialize)]
struct BitbucketSource {
    branch: BitbucketBranch,
    repository: BitbucketRepository,
}

#[derive(Deserialize)]
struct BitbucketRepository {
    full_name: String,
}

#[derive(Deserialize)]
struct BitbucketDestination {
    branch: BitbucketBranch,
}

#[derive(Deserialize)]
struct BitbucketBranch {
    name: String,
}

#[derive(Deserialize)]
struct BitbucketLinks {
    html: BitbucketHtmlLink,
}

#[derive(Deserialize)]
struct BitbucketHtmlLink {
    href: String,
}

#[derive(Deserialize)]
struct PullRequestViewResponse {
    pull_request: BitbucketPullRequestView,
}

#[derive(Deserialize)]
struct BitbucketPullRequestView {
    #[serde(rename = "source")]
    _source: BitbucketSource,
    participants: Vec<BitbucketParticipant>,
}

#[derive(Deserialize)]
struct BitbucketParticipant {
    user: BitbucketUser,
    role: String,
    approved: bool,
}

#[derive(Deserialize)]
struct CommentListResponse {
    comments: Vec<BitbucketComment>,
}

#[derive(Deserialize)]
struct BitbucketComment {
    id: u64,
    content: BitbucketContent,
    user: BitbucketUser,
    created_on: String,
    updated_on: String,
    deleted: bool,
    #[serde(default)]
    resolution: RequiredNullable<serde_json::Value>,
    #[serde(default)]
    parent: RequiredNullable<serde_json::Value>,
}

#[derive(Deserialize)]
struct BitbucketContent {
    raw: String,
}

#[derive(Deserialize)]
struct TaskListResponse {
    repo: String,
    tasks: Vec<BitbucketTask>,
    workspace: String,
}

#[derive(Deserialize)]
struct BitbucketTask {
    #[serde(default)]
    state: Option<String>,
    #[serde(default)]
    created_on: Option<String>,
}

#[derive(Deserialize)]
struct BitbucketIdentity {
    account_id: String,
    display_name: String,
}

pub fn collect(config: &Config) -> BitbucketCollection {
    collect_with(config, |program, args| {
        command::run(program, args).map_err(|error| error.to_string())
    })
}

fn collect_with<R>(config: &Config, run: R) -> BitbucketCollection
where
    R: Fn(&str, &[String]) -> Result<String, String> + Sync,
{
    let repositories = configured_repositories(config);
    if repositories.is_empty() {
        return BitbucketCollection {
            identity: None,
            pull_requests: Vec::new(),
            warnings: Vec::new(),
        };
    }

    let mut warnings = Vec::new();
    let identity_args = strings(&["api", "/user", "--json"]);
    let identity = match invoke_json::<BitbucketIdentity, _>(&run, &identity_args) {
        Ok(identity) => Some(Identity {
            id: identity.account_id,
            name: identity.display_name,
            email: None,
        }),
        Err(error) => {
            warnings.push(warning(
                &[Category::Review, Category::Retour],
                format!("Bitbucket identity collection failed: {error}"),
            ));
            None
        }
    };

    let mut authored_open = Vec::new();
    let mut authored_merged = Vec::new();
    let mut review_candidates = Vec::new();

    for repository in &repositories {
        collect_list(
            &run,
            repository,
            ListRequest::AuthoredOpen,
            &mut authored_open,
            &mut warnings,
        );
        collect_list(
            &run,
            repository,
            ListRequest::AuthoredMerged,
            &mut authored_merged,
            &mut warnings,
        );
        collect_list(
            &run,
            repository,
            ListRequest::Review,
            &mut review_candidates,
            &mut warnings,
        );
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

    let detailed_authored = parallel::bounded_map(&authored_open, |pull_request| {
        collect_authored_details(&run, pull_request, identity.as_ref())
    });
    let mut pull_requests = Vec::new();
    for result in detailed_authored {
        pull_requests.push(result.pull_request);
        warnings.extend(result.warnings);
    }
    pull_requests.extend(authored_merged);

    if let Some(identity) = &identity {
        let review_results = parallel::bounded_map(&review_candidates, |pull_request| {
            collect_review_detail(&run, pull_request, &identity.id)
        });
        for result in review_results {
            match result {
                Ok(Some(pull_request)) => pull_requests.push(pull_request),
                Ok(None) => {}
                Err(warning) => warnings.push(warning),
            }
        }
    }

    BitbucketCollection {
        identity,
        pull_requests,
        warnings,
    }
}

fn configured_repositories(config: &Config) -> Vec<Repository> {
    config
        .unique_repos()
        .into_iter()
        .filter(|repo| repo.provider == Provider::Bitbucket)
        .filter_map(|repo| {
            repository_parts(&repo.path)
                .ok()
                .map(|(workspace, short_name)| Repository {
                    key: RepoKey {
                        provider: repo.provider,
                        path: repo.path.clone(),
                    },
                    workspace: workspace.to_owned(),
                    short_name: short_name.to_owned(),
                })
        })
        .collect()
}

#[derive(Clone, Copy)]
enum ListRequest {
    AuthoredOpen,
    AuthoredMerged,
    Review,
}

impl ListRequest {
    fn args(self, repository: &Repository) -> Vec<String> {
        match self {
            Self::AuthoredOpen => strings(&[
                "pr",
                "list",
                "--mine",
                "--repo",
                &repository.short_name,
                "--workspace",
                &repository.workspace,
                "--state",
                "OPEN",
                "--limit",
                "50",
                "--json",
            ]),
            Self::AuthoredMerged => strings(&[
                "pr",
                "list",
                "--mine",
                "--repo",
                &repository.short_name,
                "--workspace",
                &repository.workspace,
                "--state",
                "MERGED",
                "--limit",
                "50",
                "--json",
            ]),
            Self::Review => strings(&[
                "pr",
                "list",
                "--reviewer",
                "--repo",
                &repository.short_name,
                "--workspace",
                &repository.workspace,
                "--state",
                "OPEN",
                "--json",
            ]),
        }
    }

    const fn expected_state(self) -> PullRequestState {
        match self {
            Self::AuthoredOpen | Self::Review => PullRequestState::Open,
            Self::AuthoredMerged => PullRequestState::Merged,
        }
    }

    const fn categories(self) -> &'static [Category] {
        match self {
            Self::AuthoredOpen => &[Category::Retour, Category::Linear],
            Self::AuthoredMerged => &[Category::Linear],
            Self::Review => &[Category::Review],
        }
    }

    const fn label(self) -> &'static str {
        match self {
            Self::AuthoredOpen => "open authored PR list",
            Self::AuthoredMerged => "merged authored PR list",
            Self::Review => "review-requested PR list",
        }
    }
}

fn collect_list<R>(
    run: &R,
    repository: &Repository,
    request: ListRequest,
    destination: &mut Vec<PullRequest>,
    warnings: &mut Vec<Warning>,
) where
    R: Fn(&str, &[String]) -> Result<String, String> + Sync,
{
    let args = request.args(repository);
    let result = run("bkt", &args)
        .map_err(|error| format!("command failed: {error}"))
        .and_then(|source| {
            parse_list(&source, &repository.key).map_err(|error| format!("invalid JSON: {error}"))
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
                "Bitbucket {} failed for {}: {error}",
                request.label(),
                repository.key.path
            ),
        )),
    }
}

fn collect_authored_details<R>(
    run: &R,
    pull_request: &PullRequest,
    identity: Option<&Identity>,
) -> DetailResult
where
    R: Fn(&str, &[String]) -> Result<String, String> + Sync,
{
    let mut pull_request = pull_request.clone();
    let mut warnings = Vec::new();
    let (workspace, repository) = repository_parts(&pull_request.key.repo.path)
        .expect("validated configuration always has a workspace and repository short name");
    let number = pull_request.key.number.to_string();

    if let Some(identity) = identity {
        let args = strings(&[
            "pr",
            "comments",
            &number,
            "--repo",
            repository,
            "--workspace",
            workspace,
            "--state",
            "unresolved",
            "--json",
        ]);
        match run("bkt", &args)
            .map_err(|error| format!("command failed: {error}"))
            .and_then(|source| {
                parse_comments(&source, &identity.id)
                    .map_err(|error| format!("invalid JSON: {error}"))
            }) {
            Ok(feedback) => pull_request.feedback.extend(feedback),
            Err(error) => warnings.push(detail_warning(&pull_request, "comments", error)),
        }
    }

    let args = strings(&[
        "pr",
        "task",
        "list",
        &number,
        "--repo",
        repository,
        "--workspace",
        workspace,
        "--json",
    ]);
    match run("bkt", &args)
        .map_err(|error| format!("command failed: {error}"))
        .and_then(|source| {
            parse_tasks(&source, &pull_request.updated_at, repository, workspace)
                .map_err(|error| format!("invalid JSON: {error}"))
        }) {
        Ok(feedback) => pull_request.feedback.extend(feedback),
        Err(error) => warnings.push(detail_warning(&pull_request, "tasks", error)),
    }

    DetailResult {
        pull_request,
        warnings,
    }
}

fn collect_review_detail<R>(
    run: &R,
    pull_request: &PullRequest,
    account_id: &str,
) -> Result<Option<PullRequest>, Warning>
where
    R: Fn(&str, &[String]) -> Result<String, String> + Sync,
{
    let (workspace, repository) = repository_parts(&pull_request.key.repo.path)
        .expect("validated configuration always has a workspace and repository short name");
    let number = pull_request.key.number.to_string();
    let args = strings(&[
        "pr",
        "view",
        &number,
        "--repo",
        repository,
        "--workspace",
        workspace,
        "--json",
    ]);
    let view = run("bkt", &args)
        .map_err(|error| format!("command failed: {error}"))
        .and_then(|source| parse_view(&source).map_err(|error| format!("invalid JSON: {error}")))
        .map_err(|error| {
            warning(
                &[Category::Review],
                format!(
                    "Bitbucket review details failed for {}#{}: {error}",
                    pull_request.key.repo.path, pull_request.key.number
                ),
            )
        })?;

    if awaits_review(&view, account_id) {
        let mut pull_request = pull_request.clone();
        pull_request.awaiting_review = true;
        Ok(Some(pull_request))
    } else {
        Ok(None)
    }
}

fn invoke_json<T, R>(run: &R, args: &[String]) -> Result<T, String>
where
    T: DeserializeOwned,
    R: Fn(&str, &[String]) -> Result<String, String> + Sync,
{
    let source = run("bkt", args).map_err(|error| format!("command failed: {error}"))?;
    serde_json::from_str(&source).map_err(|error| format!("invalid JSON: {error}"))
}

fn deduplicate(pull_requests: &mut Vec<PullRequest>) {
    let mut seen = HashSet::new();
    pull_requests.retain(|pull_request| seen.insert(pull_request.key.clone()));
}

fn warning(categories: &[Category], message: String) -> Warning {
    Warning {
        categories: categories.to_vec(),
        message,
    }
}

fn detail_warning(pull_request: &PullRequest, detail: &str, error: String) -> Warning {
    warning(
        &[Category::Retour],
        format!(
            "Bitbucket {detail} failed for {}#{}: {error}",
            pull_request.key.repo.path, pull_request.key.number
        ),
    )
}

fn strings(values: &[&str]) -> Vec<String> {
    values.iter().map(|value| (*value).to_owned()).collect()
}

fn short_name(path: &str) -> Result<&str, Box<dyn Error>> {
    repository_parts(path).map(|(_, name)| name)
}

fn repository_parts(path: &str) -> Result<(&str, &str), Box<dyn Error>> {
    let Some((workspace, name)) = path.split_once('/') else {
        return Err(format!("invalid Bitbucket repository path {path:?}").into());
    };
    if workspace.is_empty() || name.is_empty() || name.contains('/') {
        return Err(format!("invalid Bitbucket repository path {path:?}").into());
    }
    Ok((workspace, name))
}

fn parse_list(source: &str, repo: &RepoKey) -> Result<Vec<PullRequest>, Box<dyn Error>> {
    let response: PullRequestListResponse = serde_json::from_str(source)?;
    response
        .pull_requests
        .into_iter()
        .map(|pull_request| {
            let body = pull_request
                .description
                .into_option("pull_requests[].description")?
                .unwrap_or_default();
            pull_request
                .reviewers
                .into_option("pull_requests[].reviewers")?;

            let _author_account_id = pull_request.author.account_id;
            let _author_display_name = pull_request.author.display_name;
            let _source_repository = pull_request.source.repository.full_name;
            Ok(PullRequest {
                key: PullRequestKey {
                    repo: repo.clone(),
                    number: pull_request.id,
                },
                title: pull_request.title,
                body,
                branch: pull_request.source.branch.name,
                destination: pull_request.destination.branch.name,
                url: pull_request.links.html.href,
                draft: pull_request.draft,
                state: pull_request.state.into(),
                created_at: pull_request.created_on,
                updated_at: pull_request.updated_on,
                awaiting_review: false,
                feedback: Vec::new(),
            })
        })
        .collect()
}

fn parse_view(source: &str) -> Result<BitbucketPullRequestView, Box<dyn Error>> {
    let response: PullRequestViewResponse = serde_json::from_str(source)?;
    Ok(response.pull_request)
}

fn awaits_review(view: &BitbucketPullRequestView, account_id: &str) -> bool {
    view.participants.iter().any(|participant| {
        participant.role == "REVIEWER"
            && participant.user.account_id == account_id
            && !participant.approved
    })
}

fn parse_comments(source: &str, account_id: &str) -> Result<Vec<Feedback>, Box<dyn Error>> {
    let response: CommentListResponse = serde_json::from_str(source)?;
    response
        .comments
        .into_iter()
        .map(|comment| {
            comment.resolution.into_option("comments[].resolution")?;
            comment.parent.into_option("comments[].parent")?;
            let _id = comment.id;
            let _raw = comment.content.raw;
            let _display_name = comment.user.display_name;
            let _updated_on = comment.updated_on;
            Ok((comment.created_on, comment.deleted, comment.user.account_id))
        })
        .filter_map(|result| match result {
            Ok((_, true, _)) => None,
            Ok((_, false, author)) if author == account_id => None,
            Ok((created_at, false, _)) => Some(Ok(Feedback {
                created_at,
                kind: FeedbackKind::Comment,
            })),
            Err(error) => Some(Err(error)),
        })
        .collect()
}

fn parse_tasks(
    source: &str,
    fallback_at: &str,
    expected_repo: &str,
    expected_workspace: &str,
) -> Result<Vec<Feedback>, Box<dyn Error>> {
    let response: TaskListResponse = serde_json::from_str(source)?;
    if !response.repo.eq_ignore_ascii_case(expected_repo)
        || !response.workspace.eq_ignore_ascii_case(expected_workspace)
    {
        return Err(format!(
            "Bitbucket task response belongs to {}/{}, expected {expected_workspace}/{expected_repo}",
            response.workspace, response.repo
        )
        .into());
    }

    Ok(response
        .tasks
        .into_iter()
        .filter(|task| {
            !task
                .state
                .as_deref()
                .is_some_and(|state| state.eq_ignore_ascii_case("resolved"))
        })
        .map(|task| Feedback {
            created_at: task.created_on.unwrap_or_else(|| fallback_at.to_owned()),
            kind: FeedbackKind::Task,
        })
        .collect())
}

impl From<BitbucketPullRequestState> for PullRequestState {
    fn from(state: BitbucketPullRequestState) -> Self {
        match state {
            BitbucketPullRequestState::Open => Self::Open,
            BitbucketPullRequestState::Merged => Self::Merged,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{Config, Provider, RepoKey};
    use crate::model::{Category, FeedbackKind, PullRequestState};
    use std::sync::Mutex;

    const LIST_RESPONSE: &str = include_str!("fixtures/bitbucket-list.json");
    const VIEW_RESPONSE: &str = include_str!("fixtures/bitbucket-view.json");
    const COMMENT_RESPONSE: &str = include_str!("fixtures/bitbucket-comments.json");
    const TASK_RESPONSE: &str = include_str!("fixtures/bitbucket-tasks.json");

    fn repo() -> RepoKey {
        RepoKey {
            provider: Provider::Bitbucket,
            path: "ExampleOrg/shared-app".to_owned(),
        }
    }

    fn config() -> Config {
        Config::parse(
            r#"
                stale_days = 7
                next_count = 3

                [[tracks]]
                name = "First"
                teams = ["OPS"]

                  [[tracks.repos]]
                  provider = "bitbucket"
                  path = "ExampleOrg/shared-app"

                [[tracks]]
                name = "Second"
                teams = ["APP"]

                  [[tracks.repos]]
                  provider = "bitbucket"
                  path = "ExampleOrg/shared-app"

                  [[tracks.repos]]
                  provider = "github"
                  path = "ExampleOrg/standalone"
            "#,
        )
        .unwrap()
    }

    fn config_with_colliding_short_names() -> Config {
        Config::parse(
            r#"
                stale_days = 7
                next_count = 3

                [[tracks]]
                name = "Application"
                teams = ["APP"]

                  [[tracks.repos]]
                  provider = "bitbucket"
                  path = "ExampleOrg/shared-app"

                  [[tracks.repos]]
                  provider = "bitbucket"
                  path = "OtherOrg/shared-app"
            "#,
        )
        .unwrap()
    }

    fn list_response(numbers: &[(u64, &str, bool)]) -> String {
        list_response_for("ExampleOrg/shared-app", numbers)
    }

    fn list_response_for(repository_path: &str, numbers: &[(u64, &str, bool)]) -> String {
        let pull_requests = numbers
            .iter()
            .map(|(number, state, draft)| {
                serde_json::json!({
                    "id": number,
                    "title": format!("Pull request {number}"),
                    "description": format!("OPS-{number}"),
                    "state": state,
                    "draft": draft,
                    "created_on": "2026-08-01T08:00:00Z",
                    "updated_on": "2026-08-10T08:00:00Z",
                    "author": {
                        "display_name": "Example User",
                        "account_id": "account-me"
                    },
                    "source": {
                        "branch": { "name": format!("example-user/ops-{number}") },
                        "repository": { "full_name": repository_path }
                    },
                    "destination": { "branch": { "name": "develop" } },
                    "links": {
                        "html": { "href": format!("https://bitbucket.example.test/pr/{number}") }
                    },
                    "reviewers": null
                })
            })
            .collect::<Vec<_>>();
        serde_json::json!({ "pull_requests": pull_requests }).to_string()
    }

    #[test]
    fn extracts_repository_short_name() {
        assert_eq!(short_name("ExampleOrg/shared-app").unwrap(), "shared-app");
    }

    #[test]
    fn parses_lists_and_preserves_stacked_destinations() {
        let pull_requests = parse_list(LIST_RESPONSE, &repo()).unwrap();

        assert_eq!(pull_requests.len(), 2);
        assert_eq!(pull_requests[0].key.number, 101);
        assert_eq!(pull_requests[0].state, PullRequestState::Open);
        assert_eq!(pull_requests[0].destination, "feature/stack-base");
        assert_eq!(pull_requests[0].body, "OPS-101 tracks validation.");
        assert_eq!(pull_requests[1].body, "");
        assert!(pull_requests[1].draft);
    }

    #[test]
    fn rejects_missing_fields_but_accepts_nullable_descriptions_and_reviewers() {
        let mut response: serde_json::Value = serde_json::from_str(LIST_RESPONSE).unwrap();
        response["pull_requests"][0]
            .as_object_mut()
            .unwrap()
            .remove("description");

        assert!(parse_list(&response.to_string(), &repo()).is_err());
        assert!(parse_list(LIST_RESPONSE, &repo()).is_ok());
    }

    #[test]
    fn accepts_pull_requests_from_forks() {
        let source = list_response_for("OtherOrg/shared-app", &[(101, "OPEN", false)]);

        let pull_requests = parse_list(&source, &repo()).unwrap();

        assert_eq!(pull_requests.len(), 1);
        assert_eq!(pull_requests[0].key.repo, repo());
    }

    #[test]
    fn current_reviewer_must_be_present_and_not_approved() {
        let view = parse_view(VIEW_RESPONSE).unwrap();

        assert!(awaits_review(&view, "account-me"));
        assert!(!awaits_review(&view, "account-approved"));
        assert!(!awaits_review(&view, "account-other"));
        assert!(!awaits_review(&view, "account-missing"));
    }

    #[test]
    fn accepts_view_details_from_forks() {
        let mut response: serde_json::Value = serde_json::from_str(VIEW_RESPONSE).unwrap();
        response["pull_request"]["source"]["repository"]["full_name"] =
            serde_json::json!("OtherOrg/shared-app");

        let view = parse_view(&response.to_string()).unwrap();

        assert!(awaits_review(&view, "account-me"));
    }

    #[test]
    fn retains_only_external_non_deleted_unresolved_comments() {
        let feedback = parse_comments(COMMENT_RESPONSE, "account-me").unwrap();

        assert_eq!(feedback.len(), 1);
        assert_eq!(feedback[0].kind, FeedbackKind::Comment);
        assert_eq!(feedback[0].created_at, "2026-08-09T07:00:00+00:00");
    }

    #[test]
    fn parses_empty_tasks() {
        assert!(
            parse_tasks(TASK_RESPONSE, "fallback", "shared-app", "ExampleOrg")
                .unwrap()
                .is_empty()
        );
    }

    #[test]
    fn rejects_missing_or_null_task_scope() {
        let mut missing_repo: serde_json::Value = serde_json::from_str(TASK_RESPONSE).unwrap();
        missing_repo.as_object_mut().unwrap().remove("repo");
        let mut null_workspace: serde_json::Value = serde_json::from_str(TASK_RESPONSE).unwrap();
        null_workspace["workspace"] = serde_json::Value::Null;

        for response in [missing_repo, null_workspace] {
            assert!(
                parse_tasks(
                    &response.to_string(),
                    "fallback",
                    "shared-app",
                    "ExampleOrg"
                )
                .is_err()
            );
        }
    }

    #[test]
    fn treats_tasks_as_open_unless_explicitly_resolved() {
        let response = r#"{
            "repo": "shared-app",
            "workspace": "ExampleOrg",
            "tasks": [
                {"state":"OPEN","created_on":"2026-08-07T08:00:00Z"},
                {"state":"RESOLVED","created_on":"2026-08-08T08:00:00Z"},
                {},
                {"state":null,"created_on":null}
            ]
        }"#;

        let feedback =
            parse_tasks(response, "2026-08-10T00:00:00Z", "shared-app", "ExampleOrg").unwrap();

        assert_eq!(feedback.len(), 3);
        assert!(
            feedback
                .iter()
                .all(|entry| entry.kind == FeedbackKind::Task)
        );
        assert_eq!(feedback[0].created_at, "2026-08-07T08:00:00Z");
        assert_eq!(feedback[1].created_at, "2026-08-10T00:00:00Z");
        assert_eq!(feedback[2].created_at, "2026-08-10T00:00:00Z");
    }

    #[test]
    fn collection_resolves_identity_once_and_lists_each_unique_scoped_repo_sequentially() {
        let calls = Mutex::new(Vec::new());
        let collection = collect_with(&config(), |program, args| {
            calls
                .lock()
                .unwrap()
                .push((program.to_owned(), args.to_vec()));
            if args.first().map(String::as_str) == Some("api") {
                Ok(r#"{"account_id":"account-me","display_name":"Example User"}"#.to_owned())
            } else {
                Ok(r#"{"pull_requests":[]}"#.to_owned())
            }
        });

        assert_eq!(collection.identity.as_ref().unwrap().id, "account-me");
        assert!(collection.pull_requests.is_empty());
        assert!(collection.warnings.is_empty());
        assert_eq!(
            calls.into_inner().unwrap(),
            [
                (
                    "bkt".to_owned(),
                    vec!["api".to_owned(), "/user".to_owned(), "--json".to_owned()]
                ),
                (
                    "bkt".to_owned(),
                    vec![
                        "pr".to_owned(),
                        "list".to_owned(),
                        "--mine".to_owned(),
                        "--repo".to_owned(),
                        "shared-app".to_owned(),
                        "--workspace".to_owned(),
                        "ExampleOrg".to_owned(),
                        "--state".to_owned(),
                        "OPEN".to_owned(),
                        "--limit".to_owned(),
                        "50".to_owned(),
                        "--json".to_owned(),
                    ]
                ),
                (
                    "bkt".to_owned(),
                    vec![
                        "pr".to_owned(),
                        "list".to_owned(),
                        "--mine".to_owned(),
                        "--repo".to_owned(),
                        "shared-app".to_owned(),
                        "--workspace".to_owned(),
                        "ExampleOrg".to_owned(),
                        "--state".to_owned(),
                        "MERGED".to_owned(),
                        "--limit".to_owned(),
                        "50".to_owned(),
                        "--json".to_owned(),
                    ]
                ),
                (
                    "bkt".to_owned(),
                    vec![
                        "pr".to_owned(),
                        "list".to_owned(),
                        "--reviewer".to_owned(),
                        "--repo".to_owned(),
                        "shared-app".to_owned(),
                        "--workspace".to_owned(),
                        "ExampleOrg".to_owned(),
                        "--state".to_owned(),
                        "OPEN".to_owned(),
                        "--json".to_owned(),
                    ]
                ),
            ]
        );
    }

    #[test]
    fn collects_review_and_external_feedback_while_preserving_partial_details() {
        let calls = Mutex::new(Vec::new());
        let collection = collect_with(&config(), |_, args| {
            calls.lock().unwrap().push(args.to_vec());
            let command = args.join(" ");
            match command.as_str() {
                "api /user --json" => {
                    Ok(r#"{"account_id":"account-me","display_name":"Example User"}"#.to_owned())
                }
                "pr list --mine --repo shared-app --workspace ExampleOrg --state OPEN --limit 50 --json" => {
                    Ok(list_response(&[(101, "OPEN", false)]))
                }
                "pr list --mine --repo shared-app --workspace ExampleOrg --state MERGED --limit 50 --json" => {
                    Ok(list_response(&[(400, "MERGED", false)]))
                }
                "pr list --reviewer --repo shared-app --workspace ExampleOrg --state OPEN --json" => {
                    Ok(list_response(&[
                        (101, "OPEN", false),
                        (600, "OPEN", false),
                        (601, "OPEN", true),
                    ]))
                }
                "pr view 600 --repo shared-app --workspace ExampleOrg --json" => {
                    Ok(VIEW_RESPONSE.to_owned())
                }
                "pr comments 101 --repo shared-app --workspace ExampleOrg --state unresolved --json" => {
                    Ok(COMMENT_RESPONSE.to_owned())
                }
                "pr task list 101 --repo shared-app --workspace ExampleOrg --json" => {
                    Err("task endpoint unavailable".to_owned())
                }
                other => panic!("unexpected bkt command: {other}"),
            }
        });

        assert_eq!(collection.pull_requests.len(), 3);
        let authored = collection
            .pull_requests
            .iter()
            .find(|pull_request| pull_request.key.number == 101)
            .unwrap();
        assert!(
            !authored.awaiting_review,
            "authored work wins a duplicate review result"
        );
        assert_eq!(authored.feedback.len(), 1);
        assert_eq!(authored.feedback[0].kind, FeedbackKind::Comment);
        assert!(
            collection
                .pull_requests
                .iter()
                .find(|pull_request| pull_request.key.number == 600)
                .unwrap()
                .awaiting_review
        );
        assert!(
            !collection
                .pull_requests
                .iter()
                .any(|pull_request| pull_request.key.number == 601)
        );
        assert_eq!(collection.warnings.len(), 1);
        assert_eq!(collection.warnings[0].categories, [Category::Retour]);

        let calls = calls.into_inner().unwrap();
        assert!(
            calls.iter().any(|args| args.join(" ")
                == "pr view 600 --repo shared-app --workspace ExampleOrg --json")
        );
        assert!(
            !calls
                .iter()
                .any(|args| args.join(" ").starts_with("pr view 101 "))
        );
        assert!(
            !calls
                .iter()
                .any(|args| args.join(" ").starts_with("pr view 601 "))
        );
    }

    #[test]
    fn identity_failure_keeps_linear_work_and_task_feedback() {
        let task_response = r#"{
            "repo":"shared-app",
            "workspace":"ExampleOrg",
            "tasks":[{"state":"OPEN","created_on":"2026-08-07T08:00:00Z"}]
        }"#;
        let calls = Mutex::new(Vec::new());
        let collection = collect_with(&config(), |_, args| {
            calls.lock().unwrap().push(args.to_vec());
            let command = args.join(" ");
            match command.as_str() {
                "api /user --json" => Err("bkt missing".to_owned()),
                "pr list --mine --repo shared-app --workspace ExampleOrg --state OPEN --limit 50 --json" => {
                    Ok(list_response(&[(101, "OPEN", false)]))
                }
                "pr list --mine --repo shared-app --workspace ExampleOrg --state MERGED --limit 50 --json" => {
                    Ok(list_response(&[(400, "MERGED", false)]))
                }
                "pr list --reviewer --repo shared-app --workspace ExampleOrg --state OPEN --json" => {
                    Ok(list_response(&[(600, "OPEN", false)]))
                }
                "pr task list 101 --repo shared-app --workspace ExampleOrg --json" => {
                    Ok(task_response.to_owned())
                }
                other => panic!("unexpected bkt command: {other}"),
            }
        });

        assert_eq!(collection.identity, None);
        assert_eq!(collection.pull_requests.len(), 2);
        assert_eq!(collection.pull_requests[0].feedback.len(), 1);
        assert_eq!(
            collection.pull_requests[0].feedback[0].kind,
            FeedbackKind::Task
        );
        assert_eq!(
            collection.warnings[0].categories,
            [Category::Review, Category::Retour]
        );
        let calls = calls.into_inner().unwrap();
        assert!(
            !calls
                .iter()
                .any(|args| args.get(1).map(String::as_str) == Some("comments"))
        );
        assert!(
            !calls
                .iter()
                .any(|args| args.get(1).map(String::as_str) == Some("view"))
        );
    }

    #[test]
    fn list_failures_only_degrade_their_consuming_categories() {
        let collection = collect_with(&config(), |_, args| match args.join(" ").as_str() {
            "api /user --json" => {
                Ok(r#"{"account_id":"account-me","display_name":"Example User"}"#.to_owned())
            }
            command if command.contains("--mine") && command.contains("--state OPEN") => {
                Err("open list failed".to_owned())
            }
            command if command.contains("--mine") && command.contains("--state MERGED") => {
                Err("merged list failed".to_owned())
            }
            command if command.contains("--reviewer") => Err("review list failed".to_owned()),
            other => panic!("unexpected bkt command: {other}"),
        });

        assert_eq!(collection.warnings.len(), 3);
        assert_eq!(
            collection.warnings[0].categories,
            [Category::Retour, Category::Linear]
        );
        assert_eq!(collection.warnings[1].categories, [Category::Linear]);
        assert_eq!(collection.warnings[2].categories, [Category::Review]);
    }

    #[test]
    fn scopes_same_named_repositories_to_distinct_workspaces() {
        let collection = collect_with(&config_with_colliding_short_names(), |_, args| {
            let command = args.join(" ");
            match command.as_str() {
                "api /user --json" => {
                    Ok(r#"{"account_id":"account-me","display_name":"Example User"}"#.to_owned())
                }
                "pr list --mine --repo shared-app --workspace ExampleOrg --state MERGED --limit 50 --json" => {
                    Ok(list_response_for(
                        "ExampleOrg/shared-app",
                        &[(200, "MERGED", false)],
                    ))
                }
                "pr list --mine --repo shared-app --workspace OtherOrg --state MERGED --limit 50 --json" => {
                    Ok(list_response_for(
                        "OtherOrg/shared-app",
                        &[(200, "MERGED", false)],
                    ))
                }
                command if command.starts_with("pr list ") => {
                    Ok(r#"{"pull_requests":[]}"#.to_owned())
                }
                other => panic!("unexpected bkt command: {other}"),
            }
        });

        assert!(collection.warnings.is_empty());
        assert_eq!(collection.pull_requests.len(), 2);
        assert_eq!(
            collection.pull_requests[0].key.repo.path,
            "ExampleOrg/shared-app"
        );
        assert_eq!(
            collection.pull_requests[1].key.repo.path,
            "OtherOrg/shared-app"
        );
    }

    #[test]
    fn attaches_fork_pull_requests_to_the_command_target() {
        let collection = collect_with(&config(), |_, args| match args.join(" ").as_str() {
            "api /user --json" => {
                Ok(r#"{"account_id":"account-me","display_name":"Example User"}"#.to_owned())
            }
            "pr list --mine --repo shared-app --workspace ExampleOrg --state MERGED --limit 50 --json" => {
                Ok(list_response_for(
                    "OtherOrg/shared-app",
                    &[(200, "MERGED", false)],
                ))
            }
            command if command.starts_with("pr list ") => Ok(r#"{"pull_requests":[]}"#.to_owned()),
            other => panic!("unexpected bkt command: {other}"),
        });

        assert!(collection.warnings.is_empty());
        assert_eq!(collection.pull_requests.len(), 1);
        assert_eq!(
            collection.pull_requests[0].key.repo.path,
            "ExampleOrg/shared-app"
        );
    }

    #[test]
    fn mismatched_task_envelope_warns_and_drops_tasks() {
        let collection = collect_with(&config(), |_, args| match args.join(" ").as_str() {
            "api /user --json" => {
                Ok(r#"{"account_id":"account-me","display_name":"Example User"}"#.to_owned())
            }
            "pr list --mine --repo shared-app --workspace ExampleOrg --state OPEN --limit 50 --json" => {
                Ok(list_response(&[(101, "OPEN", false)]))
            }
            command if command.starts_with("pr list ") => Ok(r#"{"pull_requests":[]}"#.to_owned()),
            "pr comments 101 --repo shared-app --workspace ExampleOrg --state unresolved --json" => {
                Ok(r#"{"comments":[]}"#.to_owned())
            }
            "pr task list 101 --repo shared-app --workspace ExampleOrg --json" => Ok(
                r#"{"repo":"shared-app","workspace":"OtherOrg","tasks":[{"state":"OPEN"}]}"#
                    .to_owned(),
            ),
            other => panic!("unexpected bkt command: {other}"),
        });

        assert!(collection.pull_requests[0].feedback.is_empty());
        assert_eq!(collection.warnings.len(), 1);
        assert_eq!(collection.warnings[0].categories, [Category::Retour]);
        assert!(collection.warnings[0].message.contains("OtherOrg"));
    }
}
