use crate::command;
use crate::config::Config;
use crate::model::{Category, Identity, Issue, IssueState, Warning};
use serde::Deserialize;
use std::collections::HashMap;
use std::error::Error;

const QUERY: &str = r#"
query {
  viewer {
    id
    name
    email
    assignedIssues(first: 200) {
      nodes {
        identifier
        title
        url
        priority
        priorityLabel
        updatedAt
        branchName
        state {
          name
          type
        }
        team {
          key
        }
        project {
          name
        }
        labels {
          nodes {
            name
          }
        }
      }
    }
  }
}
"#;

// Blocking relations need their own query: merged into the one above, Linear rejects the request
// for exceeding its complexity ceiling of 10000. The API offers no filter on inverseRelations
// either, so `blocks` is selected client-side and both page sizes are bounded by that same
// ceiling — hence the truncation warnings rather than silent under-reporting. Keep these two
// constants in step with the page sizes embedded in the query below.
const ISSUE_PAGE_SIZE: usize = 100;
const RELATION_PAGE_SIZE: usize = 20;

const RELATIONS_QUERY: &str = r#"
query {
  viewer {
    assignedIssues(first: 100, filter: { state: { type: { nin: ["completed", "canceled"] } } }) {
      pageInfo {
        hasNextPage
      }
      nodes {
        identifier
        inverseRelations(first: 20) {
          pageInfo {
            hasNextPage
          }
          nodes {
            type
            issue {
              identifier
              state {
                type
              }
            }
          }
        }
      }
    }
  }
}
"#;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LinearCollection {
    pub identity: Option<Identity>,
    pub issues: Vec<Issue>,
    pub warnings: Vec<Warning>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct LinearResponse {
    data: LinearData,
    #[serde(default)]
    errors: Option<Vec<serde_json::Value>>,
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
                Err(format!("Linear response is missing required field {field}").into())
            }
            Self::Present(value) => Ok(value),
        }
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct LinearData {
    viewer: LinearViewer,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct LinearViewer {
    id: String,
    name: String,
    #[serde(default)]
    email: RequiredNullable<String>,
    assigned_issues: LinearIssues,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct LinearIssues {
    nodes: Vec<LinearIssue>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct LinearIssue {
    identifier: String,
    title: String,
    url: String,
    priority: u8,
    #[serde(rename = "priorityLabel")]
    _priority_label: String,
    updated_at: String,
    branch_name: String,
    state: LinearState,
    team: LinearTeam,
    #[serde(default)]
    project: RequiredNullable<LinearProject>,
    labels: LinearLabels,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct LinearState {
    #[serde(rename = "name")]
    _name: String,
    #[serde(rename = "type")]
    kind: LinearStateType,
}

#[derive(Clone, Copy, Deserialize)]
#[serde(rename_all = "lowercase")]
enum LinearStateType {
    Triage,
    Backlog,
    Unstarted,
    Started,
    Completed,
    Canceled,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct LinearTeam {
    key: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct LinearProject {
    name: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct LinearLabels {
    nodes: Vec<LinearLabel>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct LinearLabel {
    name: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RelationsResponse {
    data: RelationsData,
    #[serde(default)]
    errors: Option<Vec<serde_json::Value>>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RelationsData {
    viewer: RelationsViewer,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RelationsViewer {
    assigned_issues: RelationsIssues,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RelationsIssues {
    page_info: PageInfo,
    nodes: Vec<RelationsIssue>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct PageInfo {
    has_next_page: bool,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RelationsIssue {
    identifier: String,
    inverse_relations: InverseRelations,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct InverseRelations {
    page_info: PageInfo,
    nodes: Vec<InverseRelation>,
}

// An inverse relation is stored on the target: a `blocks` entry names an issue blocking this one.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct InverseRelation {
    #[serde(rename = "type")]
    kind: LinearRelationType,
    issue: RelatedIssue,
}

#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
enum LinearRelationType {
    Blocks,
    Duplicate,
    Related,
    Similar,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RelatedIssue {
    identifier: String,
    state: RelatedState,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RelatedState {
    #[serde(rename = "type")]
    kind: LinearStateType,
}

#[derive(Debug, Default)]
struct BlockerIndex {
    blockers: HashMap<String, Vec<String>>,
    truncated_relations: Vec<String>,
    truncated_issues: bool,
}

impl BlockerIndex {
    fn warnings(&self) -> Vec<Warning> {
        let categories = vec![Category::Linear, Category::Suivant];
        let mut warnings = Vec::new();
        if self.truncated_issues {
            warnings.push(Warning {
                categories: categories.clone(),
                message: format!(
                    "blockers were read for the first {ISSUE_PAGE_SIZE} unresolved issues only: blocked issues beyond that page stay in the report"
                ),
            });
        }
        if !self.truncated_relations.is_empty() {
            warnings.push(Warning {
                categories,
                message: format!(
                    "blockers were read for the first {RELATION_PAGE_SIZE} relations of {} only: further blockers stay unseen",
                    self.truncated_relations.join(", ")
                ),
            });
        }
        warnings
    }
}

pub fn collect(config: &Config) -> LinearCollection {
    collect_with(
        config,
        |program, args| {
            command::run_json(program, args).map_err(|error| -> Box<dyn Error> { Box::new(error) })
        },
        |program, args| {
            command::run_json(program, args).map_err(|error| -> Box<dyn Error> { Box::new(error) })
        },
    )
}

fn collect_with<I, R>(config: &Config, run_issues: I, run_relations: R) -> LinearCollection
where
    I: FnOnce(&str, &[String]) -> Result<LinearResponse, Box<dyn Error>>,
    R: FnOnce(&str, &[String]) -> Result<RelationsResponse, Box<dyn Error>>,
{
    let team_keys = config.team_keys();
    if team_keys.is_empty() {
        return LinearCollection {
            identity: None,
            issues: Vec::new(),
            warnings: Vec::new(),
        };
    }

    let args = vec!["api".to_owned(), QUERY.to_owned()];
    let (identity, mut issues) =
        match run_issues("linear", &args).and_then(|response| map_response(response, &team_keys)) {
            Ok(collected) => collected,
            Err(error) => {
                return LinearCollection {
                    identity: None,
                    issues: Vec::new(),
                    warnings: vec![Warning {
                        categories: vec![Category::Linear, Category::Suivant],
                        message: format!("linear collection failed: {error}"),
                    }],
                };
            }
        };
    if issues.is_empty() {
        return LinearCollection {
            identity: Some(identity),
            issues,
            warnings: Vec::new(),
        };
    }

    // A blocker lookup that fails only costs precision, so it degrades with a warning instead of
    // discarding issues the report would otherwise still render correctly.
    let args = vec!["api".to_owned(), RELATIONS_QUERY.to_owned()];
    let warnings = match run_relations("linear", &args).and_then(map_relations) {
        Ok(mut index) => {
            let warnings = index.warnings();
            for issue in &mut issues {
                if let Some(blockers) = index
                    .blockers
                    .remove(&issue.identifier.to_ascii_uppercase())
                {
                    issue.blockers = blockers;
                }
            }
            warnings
        }
        Err(error) => vec![Warning {
            categories: vec![Category::Linear, Category::Suivant],
            message: format!(
                "linear blocker collection failed, blocked issues stay in the report: {error}"
            ),
        }],
    };

    LinearCollection {
        identity: Some(identity),
        issues,
        warnings,
    }
}

fn map_relations(response: RelationsResponse) -> Result<BlockerIndex, Box<dyn Error>> {
    if response
        .errors
        .as_ref()
        .is_some_and(|errors| !errors.is_empty())
    {
        return Err("Linear GraphQL response contains errors".into());
    }

    let assigned = response.data.viewer.assigned_issues;
    let mut index = BlockerIndex {
        truncated_issues: assigned.page_info.has_next_page,
        ..BlockerIndex::default()
    };
    for issue in assigned.nodes {
        if issue.inverse_relations.page_info.has_next_page {
            index.truncated_relations.push(issue.identifier.clone());
        }
        let mut blockers = issue
            .inverse_relations
            .nodes
            .into_iter()
            .filter(|relation| matches!(relation.kind, LinearRelationType::Blocks))
            .filter(|relation| !IssueState::from(relation.issue.state.kind).is_resolved())
            .map(|relation| relation.issue.identifier)
            .collect::<Vec<_>>();
        if !blockers.is_empty() {
            blockers.sort();
            index
                .blockers
                .insert(issue.identifier.to_ascii_uppercase(), blockers);
        }
    }
    index.truncated_relations.sort();

    Ok(index)
}

#[cfg(test)]
fn parse_response(
    source: &str,
    team_keys: &[String],
) -> Result<(Identity, Vec<Issue>), Box<dyn Error>> {
    let response = serde_json::from_str(source)?;
    map_response(response, team_keys)
}

#[cfg(test)]
fn parse_relations(source: &str) -> Result<BlockerIndex, Box<dyn Error>> {
    map_relations(serde_json::from_str(source)?)
}

fn map_response(
    response: LinearResponse,
    team_keys: &[String],
) -> Result<(Identity, Vec<Issue>), Box<dyn Error>> {
    if response
        .errors
        .as_ref()
        .is_some_and(|errors| !errors.is_empty())
    {
        return Err("Linear GraphQL response contains errors".into());
    }

    let viewer = response.data.viewer;
    let identity = Identity {
        id: viewer.id,
        name: viewer.name,
        email: viewer.email.into_option("data.viewer.email")?,
    };
    let issues =
        viewer
            .assigned_issues
            .nodes
            .into_iter()
            .try_fold(Vec::new(), |mut issues, issue| {
                let project = issue
                    .project
                    .into_option("data.viewer.assignedIssues.nodes.project")?;
                if team_keys
                    .iter()
                    .any(|team_key| team_key.eq_ignore_ascii_case(&issue.team.key))
                {
                    issues.push(Issue {
                        identifier: issue.identifier,
                        title: issue.title,
                        url: issue.url,
                        priority: issue.priority,
                        updated_at: issue.updated_at,
                        branch_name: issue.branch_name,
                        state_type: issue.state.kind.into(),
                        team_key: issue.team.key,
                        project: project.map(|project| project.name),
                        labels: issue
                            .labels
                            .nodes
                            .into_iter()
                            .map(|label| label.name)
                            .collect(),
                        blockers: Vec::new(),
                    });
                }
                Ok::<_, Box<dyn Error>>(issues)
            })?;

    Ok((identity, issues))
}

impl From<LinearStateType> for IssueState {
    fn from(state: LinearStateType) -> Self {
        match state {
            LinearStateType::Triage => Self::Triage,
            LinearStateType::Backlog => Self::Backlog,
            LinearStateType::Unstarted => Self::Unstarted,
            LinearStateType::Started => Self::Started,
            LinearStateType::Completed => Self::Completed,
            LinearStateType::Canceled => Self::Canceled,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{Config, Track};
    use crate::model::{Category, IssueState};
    use std::error::Error;

    const RESPONSE: &str = include_str!("fixtures/linear-response.json");
    const RELATIONS: &str = include_str!("fixtures/linear-relations.json");

    fn relations_response() -> RelationsResponse {
        serde_json::from_str(RELATIONS).unwrap()
    }

    #[test]
    fn parses_identity_and_every_issue_field() {
        let (identity, issues) = parse_response(RESPONSE, &["ops".to_owned()]).unwrap();

        assert_eq!(identity.id, "user-42");
        assert_eq!(identity.name, "Example User");
        assert_eq!(identity.email.as_deref(), Some("ada@example.test"));
        assert_eq!(issues.len(), 1);

        let issue = &issues[0];
        assert_eq!(issue.identifier, "OPS-168");
        assert_eq!(issue.title, "Collect assigned Linear issues");
        assert_eq!(issue.url, "https://linear.example.test/issue/OPS-168");
        assert_eq!(issue.priority, 2);
        assert_eq!(issue.updated_at, "2026-08-11T08:42:00.000Z");
        assert_eq!(
            issue.branch_name,
            "example-user/ops-168-collect-linear-issues"
        );
        assert_eq!(issue.state_type, IssueState::Started);
        assert_eq!(issue.team_key, "OPS");
        assert_eq!(issue.project, None);
        assert_eq!(issue.labels, ["daily-routine", "backend"]);
        assert!(
            issue.blockers.is_empty(),
            "the issue query carries no relation, blockers come from the second query"
        );
    }

    #[test]
    fn keeps_only_unresolved_blocking_relations() {
        let index = parse_relations(RELATIONS).unwrap();

        assert_eq!(
            index.blockers.get("OPS-168"),
            Some(&vec!["OPS-89".to_owned()])
        );
        assert_eq!(
            index.blockers.keys().collect::<Vec<_>>(),
            ["OPS-168"],
            "completed and canceled blockers, and non-blocking relations, must not block"
        );
        assert!(index.warnings().is_empty());
    }

    #[test]
    fn sorts_blockers_so_the_report_stays_stable() {
        let mut response: serde_json::Value = serde_json::from_str(RELATIONS).unwrap();
        response["data"]["viewer"]["assignedIssues"]["nodes"][0]["inverseRelations"]["nodes"][0]
            ["issue"] = serde_json::json!({
            "identifier": "OPS-93",
            "state": { "type": "started" }
        });

        let index = parse_relations(&response.to_string()).unwrap();

        assert_eq!(
            index.blockers.get("OPS-168"),
            Some(&vec!["OPS-89".to_owned(), "OPS-93".to_owned()])
        );
    }

    #[test]
    fn warns_instead_of_under_reporting_truncated_relations() {
        let mut response: serde_json::Value = serde_json::from_str(RELATIONS).unwrap();
        response["data"]["viewer"]["assignedIssues"]["pageInfo"]["hasNextPage"] =
            serde_json::Value::Bool(true);
        response["data"]["viewer"]["assignedIssues"]["nodes"][2]["inverseRelations"]["pageInfo"]
            ["hasNextPage"] = serde_json::Value::Bool(true);
        response["data"]["viewer"]["assignedIssues"]["nodes"][0]["inverseRelations"]["pageInfo"]
            ["hasNextPage"] = serde_json::Value::Bool(true);

        let warnings = parse_relations(&response.to_string()).unwrap().warnings();

        assert_eq!(warnings.len(), 2);
        assert!(
            warnings
                .iter()
                .all(|warning| warning.categories == [Category::Linear, Category::Suivant])
        );
        assert!(warnings[0].message.contains("first 100 unresolved issues"));
        assert!(
            warnings[1]
                .message
                .contains("first 20 relations of EXT-7, OPS-168"),
            "truncated issues must be listed in a stable order: {}",
            warnings[1].message
        );
    }

    #[test]
    fn rejects_unknown_relation_types() {
        let response = RELATIONS.replacen("\"type\": \"blocks\"", "\"type\": \"clones\"", 1);

        assert!(parse_relations(&response).is_err());
    }

    #[test]
    fn rejects_relation_payloads_with_graphql_errors() {
        let mut response: serde_json::Value = serde_json::from_str(RELATIONS).unwrap();
        response["errors"] = serde_json::json!([{ "message": "permission denied" }]);

        let error = parse_relations(&response.to_string()).unwrap_err();

        assert!(error.to_string().contains("GraphQL"));
    }

    #[test]
    fn relations_query_declares_the_page_sizes_the_warnings_report() {
        let query = RELATIONS_QUERY
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ");

        assert!(query.contains(&format!("assignedIssues(first: {ISSUE_PAGE_SIZE},")));
        assert!(query.contains(&format!("inverseRelations(first: {RELATION_PAGE_SIZE})")));
        assert!(
            query.contains(r#"nin: ["completed", "canceled"]"#),
            "a resolved issue is never blocked, and the complexity ceiling needs the smaller page"
        );
    }

    #[test]
    fn accepts_explicitly_null_email_and_project() {
        let mut response: serde_json::Value = serde_json::from_str(RESPONSE).unwrap();
        response["data"]["viewer"]["email"] = serde_json::Value::Null;
        response["data"]["viewer"]["assignedIssues"]["nodes"][0]["project"] =
            serde_json::Value::Null;

        let (identity, issues) =
            parse_response(&response.to_string(), &["OPS".to_owned()]).unwrap();

        assert_eq!(identity.email, None);
        assert_eq!(issues[0].project, None);
    }

    #[test]
    fn rejects_responses_without_viewer_email() {
        let mut response: serde_json::Value = serde_json::from_str(RESPONSE).unwrap();
        response["data"]["viewer"]
            .as_object_mut()
            .unwrap()
            .remove("email");

        assert!(parse_response(&response.to_string(), &["OPS".to_owned()]).is_err());
    }

    #[test]
    fn rejects_responses_without_issue_project() {
        let mut response: serde_json::Value = serde_json::from_str(RESPONSE).unwrap();
        response["data"]["viewer"]["assignedIssues"]["nodes"][0]
            .as_object_mut()
            .unwrap()
            .remove("project");

        assert!(parse_response(&response.to_string(), &["OPS".to_owned()]).is_err());
    }

    #[test]
    fn rejects_missing_projects_before_filtering_teams() {
        let mut response: serde_json::Value = serde_json::from_str(RESPONSE).unwrap();
        response["data"]["viewer"]["assignedIssues"]["nodes"][2]
            .as_object_mut()
            .unwrap()
            .remove("project");

        assert!(parse_response(&response.to_string(), &["OPS".to_owned()]).is_err());
    }

    #[test]
    fn filters_issues_to_configured_team_keys_case_insensitively() {
        let (_, issues) = parse_response(RESPONSE, &["oPs".to_owned(), "app".to_owned()]).unwrap();

        assert_eq!(
            issues
                .iter()
                .map(|issue| issue.identifier.as_str())
                .collect::<Vec<_>>(),
            ["OPS-168", "APP-12"]
        );
        assert_eq!(issues[0].project, None);
        assert_eq!(issues[1].project.as_deref(), Some("Application"));
        assert_eq!(issues[0].labels, ["daily-routine", "backend"]);
        assert!(issues[1].labels.is_empty());
    }

    #[test]
    fn maps_all_six_linear_state_types() {
        for (source, expected) in [
            ("triage", IssueState::Triage),
            ("backlog", IssueState::Backlog),
            ("unstarted", IssueState::Unstarted),
            ("started", IssueState::Started),
            ("completed", IssueState::Completed),
            ("canceled", IssueState::Canceled),
        ] {
            let response = RESPONSE.replacen(
                "\"type\": \"started\"",
                &format!("\"type\": \"{source}\""),
                1,
            );

            let (_, issues) = parse_response(&response, &["OPS".to_owned()]).unwrap();

            assert_eq!(issues[0].state_type, expected, "state type {source}");
        }
    }

    #[test]
    fn rejects_malformed_and_incomplete_responses() {
        assert!(parse_response("{", &["OPS".to_owned()]).is_err());

        let mut incomplete: serde_json::Value = serde_json::from_str(RESPONSE).unwrap();
        incomplete["data"]["viewer"]["assignedIssues"]["nodes"][0]
            .as_object_mut()
            .unwrap()
            .remove("title");

        assert!(
            parse_response(&incomplete.to_string(), &["OPS".to_owned()]).is_err(),
            "a missing critical issue field must fail schema parsing"
        );

        let mut incomplete_state: serde_json::Value = serde_json::from_str(RESPONSE).unwrap();
        incomplete_state["data"]["viewer"]["assignedIssues"]["nodes"][0]["state"]
            .as_object_mut()
            .unwrap()
            .remove("name");

        assert!(
            parse_response(&incomplete_state.to_string(), &["OPS".to_owned()]).is_err(),
            "a missing state name must fail schema parsing"
        );
    }

    #[test]
    fn rejects_responses_without_priority_labels() {
        let mut response: serde_json::Value = serde_json::from_str(RESPONSE).unwrap();
        response["data"]["viewer"]["assignedIssues"]["nodes"][0]
            .as_object_mut()
            .unwrap()
            .remove("priorityLabel");

        assert!(
            parse_response(&response.to_string(), &["OPS".to_owned()]).is_err(),
            "a missing priority label must fail schema parsing"
        );
    }

    #[test]
    fn rejects_unknown_linear_state_types() {
        let response = RESPONSE.replacen("\"type\": \"started\"", "\"type\": \"paused\"", 1);

        assert!(parse_response(&response, &["OPS".to_owned()]).is_err());
    }

    #[test]
    fn rejects_partial_data_with_graphql_errors() {
        let mut response: serde_json::Value = serde_json::from_str(RESPONSE).unwrap();
        response["errors"] = serde_json::json!([{ "message": "permission denied" }]);

        let error = parse_response(&response.to_string(), &["OPS".to_owned()]).unwrap_err();

        assert!(error.to_string().contains("GraphQL"));
    }

    #[test]
    fn collect_uses_two_linear_api_queries_with_every_critical_field() {
        let config = config(&["OPS"]);
        let mut issue_calls = 0;
        let mut relation_calls = 0;

        let collection = collect_with(
            &config,
            |program, args| {
                issue_calls += 1;
                assert_eq!(program, "linear");
                assert_eq!(args.len(), 2);
                assert_eq!(args[0], "api");
                assert!(!args.join(" ").contains("issue mine"));

                let query = &args[1];
                let normalized_query = query.split_whitespace().collect::<Vec<_>>().join(" ");
                assert_eq!(
                    normalized_query,
                    "query { viewer { id name email assignedIssues(first: 200) { nodes { identifier title url priority priorityLabel updatedAt branchName state { name type } team { key } project { name } labels { nodes { name } } } } } }"
                );

                Ok::<_, Box<dyn Error>>(serde_json::from_str(RESPONSE).unwrap())
            },
            |program, args| {
                relation_calls += 1;
                assert_eq!(program, "linear");
                assert_eq!(args[0], "api");

                let normalized_query = args[1].split_whitespace().collect::<Vec<_>>().join(" ");
                assert_eq!(
                    normalized_query,
                    r#"query { viewer { assignedIssues(first: 100, filter: { state: { type: { nin: ["completed", "canceled"] } } }) { pageInfo { hasNextPage } nodes { identifier inverseRelations(first: 20) { pageInfo { hasNextPage } nodes { type issue { identifier state { type } } } } } } } }"#
                );

                Ok::<_, Box<dyn Error>>(relations_response())
            },
        );

        assert_eq!((issue_calls, relation_calls), (1, 1));
        assert_eq!(collection.identity.unwrap().id, "user-42");
        assert_eq!(collection.issues.len(), 1);
        assert_eq!(collection.issues[0].blockers, ["OPS-89"]);
        assert!(collection.warnings.is_empty());
    }

    #[test]
    fn collect_skips_linear_when_no_team_is_configured() {
        let config = config(&[]);

        let collection = collect_with(
            &config,
            |_, _| -> Result<LinearResponse, Box<dyn Error>> {
                panic!("the Linear CLI must not run without configured teams")
            },
            |_, _| -> Result<RelationsResponse, Box<dyn Error>> {
                panic!("the Linear CLI must not run without configured teams")
            },
        );

        assert_eq!(collection.identity, None);
        assert!(collection.issues.is_empty());
        assert!(collection.warnings.is_empty());
    }

    #[test]
    fn collect_skips_the_relation_query_without_issues_in_scope() {
        let config = config(&["ZZZ"]);

        let collection = collect_with(
            &config,
            |_, _| Ok::<_, Box<dyn Error>>(serde_json::from_str(RESPONSE).unwrap()),
            |_, _| -> Result<RelationsResponse, Box<dyn Error>> {
                panic!("blockers must not be queried when no issue is in scope")
            },
        );

        assert_eq!(collection.identity.unwrap().id, "user-42");
        assert!(collection.issues.is_empty());
        assert!(collection.warnings.is_empty());
    }

    #[test]
    fn collect_turns_any_failure_into_one_contextual_warning() {
        let config = config(&["OPS"]);

        let collection = collect_with(
            &config,
            |_, _| Err::<LinearResponse, Box<dyn Error>>("simulated schema failure".into()),
            |_, _| -> Result<RelationsResponse, Box<dyn Error>> {
                panic!("blockers must not be queried once the issue query failed")
            },
        );

        assert_eq!(collection.identity, None);
        assert!(collection.issues.is_empty());
        assert_eq!(collection.warnings.len(), 1);
        assert_eq!(
            collection.warnings[0].categories,
            [Category::Linear, Category::Suivant]
        );
        assert!(collection.warnings[0].message.contains("linear"));
        assert!(
            collection.warnings[0]
                .message
                .contains("simulated schema failure")
        );
    }

    #[test]
    fn collect_keeps_the_report_when_only_the_blocker_query_fails() {
        let config = config(&["OPS"]);

        let collection = collect_with(
            &config,
            |_, _| Ok::<_, Box<dyn Error>>(serde_json::from_str(RESPONSE).unwrap()),
            |_, _| Err::<RelationsResponse, Box<dyn Error>>("simulated relation failure".into()),
        );

        assert_eq!(collection.issues.len(), 1);
        assert!(
            collection.issues[0].blockers.is_empty(),
            "unknown blockers must leave the issue in the report rather than hide it"
        );
        assert_eq!(collection.warnings.len(), 1);
        assert_eq!(
            collection.warnings[0].categories,
            [Category::Linear, Category::Suivant]
        );
        assert!(
            collection.warnings[0]
                .message
                .contains("simulated relation failure")
        );
        assert!(
            collection.warnings[0]
                .message
                .contains("blocked issues stay in the report")
        );
    }

    #[test]
    fn collect_degrades_partial_data_with_graphql_errors() {
        let config = config(&["OPS"]);
        let mut response: serde_json::Value = serde_json::from_str(RESPONSE).unwrap();
        response["errors"] = serde_json::json!([{ "message": "permission denied" }]);

        let collection = collect_with(
            &config,
            |_, _| Ok::<_, Box<dyn Error>>(serde_json::from_value(response).unwrap()),
            |_, _| -> Result<RelationsResponse, Box<dyn Error>> {
                panic!("blockers must not be queried once the issue query failed")
            },
        );

        assert_eq!(collection.identity, None);
        assert!(collection.issues.is_empty());
        assert_eq!(collection.warnings.len(), 1);
        assert_eq!(
            collection.warnings[0].categories,
            [Category::Linear, Category::Suivant]
        );
        assert!(collection.warnings[0].message.contains("linear"));
        assert!(collection.warnings[0].message.contains("GraphQL"));
    }

    fn config(teams: &[&str]) -> Config {
        Config {
            stale_days: 7,
            next_count: 3,
            tracks: vec![Track {
                name: "Platform".to_owned(),
                teams: teams.iter().map(|team| (*team).to_owned()).collect(),
                repos: Vec::new(),
            }],
        }
    }
}
