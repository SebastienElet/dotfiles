use crate::command;
use crate::config::Config;
use crate::model::{Category, Identity, Issue, IssueState, Warning};
use serde::Deserialize;
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
    name: String,
    #[serde(rename = "type")]
    kind: LinearStateType,
}

#[derive(Deserialize)]
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

pub fn collect(config: &Config) -> LinearCollection {
    collect_with(config, |program, args| {
        command::run_json(program, args).map_err(|error| -> Box<dyn Error> { Box::new(error) })
    })
}

fn collect_with<F>(config: &Config, run_json: F) -> LinearCollection
where
    F: FnOnce(&str, &[String]) -> Result<LinearResponse, Box<dyn Error>>,
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
    match run_json("linear", &args).and_then(|response| map_response(response, &team_keys)) {
        Ok((identity, issues)) => LinearCollection {
            identity: Some(identity),
            issues,
            warnings: Vec::new(),
        },
        Err(error) => LinearCollection {
            identity: None,
            issues: Vec::new(),
            warnings: vec![Warning {
                categories: vec![Category::Linear, Category::Suivant],
                message: format!("linear collection failed: {error}"),
            }],
        },
    }
}

fn parse_response(
    source: &str,
    team_keys: &[String],
) -> Result<(Identity, Vec<Issue>), Box<dyn Error>> {
    let response = serde_json::from_str(source)?;
    map_response(response, team_keys)
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
    fn collect_uses_one_linear_api_query_with_every_critical_field() {
        let config = config(&["OPS"]);
        let mut calls = 0;

        let collection = collect_with(&config, |program, args| {
            calls += 1;
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
        });

        assert_eq!(calls, 1);
        assert_eq!(collection.identity.unwrap().id, "user-42");
        assert_eq!(collection.issues.len(), 1);
        assert!(collection.warnings.is_empty());
    }

    #[test]
    fn collect_skips_linear_when_no_team_is_configured() {
        let config = config(&[]);

        let collection = collect_with(&config, |_, _| -> Result<LinearResponse, Box<dyn Error>> {
            panic!("the Linear CLI must not run without configured teams")
        });

        assert_eq!(collection.identity, None);
        assert!(collection.issues.is_empty());
        assert!(collection.warnings.is_empty());
    }

    #[test]
    fn collect_turns_any_failure_into_one_contextual_warning() {
        let config = config(&["OPS"]);

        let collection = collect_with(&config, |_, _| {
            Err::<LinearResponse, Box<dyn Error>>("simulated schema failure".into())
        });

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
    fn collect_degrades_partial_data_with_graphql_errors() {
        let config = config(&["OPS"]);
        let mut response: serde_json::Value = serde_json::from_str(RESPONSE).unwrap();
        response["errors"] = serde_json::json!([{ "message": "permission denied" }]);

        let collection = collect_with(&config, |_, _| {
            Ok::<_, Box<dyn Error>>(serde_json::from_value(response).unwrap())
        });

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
