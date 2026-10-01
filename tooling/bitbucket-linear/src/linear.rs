use crate::{
    model::{
        Attachment, Fallible, Issue, IssueId, RepositoryConfig, State, Team, TeamId, nullable,
        team_key,
    },
    providers::{Providers, parse},
};
use serde::Deserialize;
use serde_json::json;
use std::collections::BTreeSet;
use time::{OffsetDateTime, format_description::well_known::Rfc3339};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct PageInfo {
    has_next_page: bool,
    #[serde(deserialize_with = "nullable")]
    end_cursor: Option<String>,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Connection<T> {
    nodes: Vec<T>,
    page_info: PageInfo,
}
#[derive(Deserialize)]
struct RawTeam {
    id: TeamId,
    key: String,
    states: Connection<State>,
}
#[derive(Deserialize)]
struct TeamData {
    team: RawTeam,
}
#[derive(Deserialize)]
struct TeamReference {
    id: TeamId,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawIssue {
    id: IssueId,
    identifier: String,
    team: TeamReference,
    state: State,
    #[serde(deserialize_with = "nullable")]
    archived_at: Option<String>,
    attachments: Connection<Attachment>,
}
#[derive(Deserialize)]
struct IssueData {
    issue: RawIssue,
}

fn advance(info: PageInfo, visited: &mut BTreeSet<String>) -> Fallible<Option<String>> {
    if info.end_cursor.as_ref().is_some_and(String::is_empty) {
        return Err("empty Linear cursor");
    }
    if !info.has_next_page {
        return Ok(None);
    }
    let cursor = info.end_cursor.ok_or("Linear cursor missing")?;
    if !visited.insert(cursor.clone()) {
        return Err("Linear pagination cycle");
    }
    Ok(Some(cursor))
}

pub const fn validate_state(state: &State) -> Fallible<()> {
    if state.name.is_empty() {
        return Err("Linear state name empty");
    }
    Ok(())
}

pub fn validate_attachment(value: &Attachment) -> Fallible<()> {
    if value.id.is_empty() || url::Url::parse(&value.url).is_err() {
        return Err("attachment identity invalid");
    }
    Ok(())
}

fn identifier(value: &str) -> bool {
    value.split_once('-').is_some_and(|(key, number)| {
        team_key(key) && !number.is_empty() && number.bytes().all(|c| c.is_ascii_digit())
    })
}

pub fn read_team(providers: &Providers<'_>, repo: &RepositoryConfig) -> Fallible<Team> {
    let mut after: Option<String> = None;
    let mut visited = BTreeSet::new();
    let mut states = Vec::new();
    loop {
        let data: TeamData = parse(providers.graphql("query Team($id:String!,$after:String) { team(id:$id) { id key states(first:100,after:$after) { nodes { id name type } pageInfo { hasNextPage endCursor } } } }", &json!({"id":repo.team_id,"after":after}))?)?;
        if data.team.id != repo.team_id || data.team.key != repo.team_key {
            return Err("Linear team identity mismatch");
        }
        for state in &data.team.states.nodes {
            validate_state(state)?;
        }
        states.extend(data.team.states.nodes);
        after = advance(data.team.states.page_info, &mut visited)?;
        if after.is_none() {
            break;
        }
    }
    if states.iter().map(|s| s.id).collect::<BTreeSet<_>>().len() != states.len() {
        return Err("duplicate team state");
    }
    Ok(Team {
        id: repo.team_id,
        key: repo.team_key.clone(),
        states,
        completed_state_id: repo.completed_state_id,
    })
}

pub fn read_issue(providers: &Providers<'_>, requested: &str) -> Fallible<Issue> {
    let mut after: Option<String> = None;
    let mut visited = BTreeSet::new();
    let mut result: Option<Issue> = None;
    let mut attachments = Vec::new();
    loop {
        let data: IssueData = parse(providers.graphql("query Issue($id:String!,$after:String) { issue(id:$id) { id identifier archivedAt team { id } state { id name type } attachments(first:100,after:$after) { nodes { id url title subtitle } pageInfo { hasNextPage endCursor } } } }", &json!({"id":requested,"after":after}))?)?;
        let raw = data.issue;
        validate_state(&raw.state)?;
        if !identifier(&raw.identifier)
            || (raw.id.to_string() != requested && raw.identifier != requested)
            || raw
                .archived_at
                .as_ref()
                .is_some_and(|date| OffsetDateTime::parse(date, &Rfc3339).is_err())
        {
            return Err("issue identity or archive timestamp invalid");
        }
        let issue = Issue {
            id: raw.id,
            identifier: raw.identifier,
            team_id: raw.team.id,
            state: raw.state,
            archived_at: raw.archived_at,
            attachments: Vec::new(),
        };
        if result
            .as_ref()
            .is_some_and(|previous| !super::core::same_issue(previous, &issue))
        {
            return Err("issue changed during pagination");
        }
        for attachment in &raw.attachments.nodes {
            validate_attachment(attachment)?;
        }
        attachments.extend(raw.attachments.nodes);
        after = advance(raw.attachments.page_info, &mut visited)?;
        result = Some(issue);
        if after.is_none() {
            break;
        }
    }
    if attachments
        .iter()
        .map(|a| &a.id)
        .collect::<BTreeSet<_>>()
        .len()
        != attachments.len()
    {
        return Err("duplicate attachment identity");
    }
    let mut issue = result.ok_or("issue absent")?;
    issue.attachments = attachments;
    Ok(issue)
}
