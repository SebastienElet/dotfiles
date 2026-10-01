use crate::{
    model::{Config, Fallible, Issue, RepositoryConfig, Team, present},
    process,
};
use serde::{Deserialize, de::DeserializeOwned};
use serde_json::{Value, json};
use std::time::Duration;

pub struct Providers<'a> {
    pub config: &'a Config,
}

pub fn parse<T: DeserializeOwned>(value: Value) -> Fallible<T> {
    serde_json::from_value(value).map_err(|_| "provider response malformed")
}

#[derive(Deserialize)]
struct Envelope {
    data: Value,
    #[serde(default, deserialize_with = "present")]
    errors: Option<Vec<Value>>,
}

#[derive(Deserialize)]
struct Context {
    name: String,
    host: String,
}
#[derive(Deserialize)]
struct Contexts {
    contexts: Vec<Context>,
}

impl Providers<'_> {
    pub fn bitbucket(&self, path: &str) -> Fallible<Value> {
        process::command(
            "bkt",
            &[
                "api".into(),
                path.into(),
                "--context".into(),
                self.config.bkt_context.clone(),
                "--json".into(),
            ],
            Duration::from_secs(30),
        )
    }

    pub fn graphql(&self, query: &str, variables: &Value) -> Fallible<Value> {
        let envelope: Envelope = parse(process::command(
            "linear",
            &[
                "api".into(),
                query.into(),
                "--workspace".into(),
                self.config.linear_workspace.clone(),
                "--variables-json".into(),
                variables.to_string(),
            ],
            Duration::from_secs(30),
        )?)?;
        if envelope.data.is_null() || envelope.errors.is_some_and(|errors| !errors.is_empty()) {
            return Err("Linear returned errors or missing data");
        }
        Ok(envelope.data)
    }

    pub fn identity(&self) -> Fallible<Value> {
        let contexts: Contexts = parse(process::command(
            "bkt",
            &["context".into(), "list".into(), "--json".into()],
            Duration::from_secs(30),
        )?)?;
        let matching: Vec<_> = contexts
            .contexts
            .iter()
            .filter(|c| c.name == self.config.bkt_context)
            .collect();
        if matching.len() != 1
            || matching
                .first()
                .is_none_or(|c| c.host != "api.bitbucket.org")
        {
            return Err("configured Bitbucket context is not uniquely Cloud");
        }
        let user: User = parse(self.bitbucket("/2.0/user")?)?;
        super::bitbucket::validate_uuid(&user.uuid)?;
        let identity: Identity = parse(self.graphql(
            "query Identity { viewer { id } organization { id } }",
            &json!({}),
        )?)?;
        Ok(
            json!({"bitbucketUserId":user.uuid, "linearUserId":identity.viewer.id,"linearOrganizationId":identity.organization.id}),
        )
    }

    pub fn team(&self, repo: &RepositoryConfig) -> Fallible<Team> {
        super::linear::read_team(self, repo)
    }
    pub fn issue(&self, id: &str) -> Fallible<Issue> {
        super::linear::read_issue(self, id)
    }
}

#[derive(Deserialize)]
struct User {
    uuid: String,
}
#[derive(Deserialize)]
struct Identity {
    viewer: Entity,
    organization: Entity,
}
#[derive(Deserialize)]
struct Entity {
    id: uuid::Uuid,
}
