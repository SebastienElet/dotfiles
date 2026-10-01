use serde_json::{Value, json};
pub mod json;

use std::{
    error::Error,
    fs,
    path::Path,
    process::{Command, Output},
};

pub const TEAM: &str = "00000000-0000-4000-8000-000000000001";
pub const ISSUE: &str = "00000000-0000-4000-8000-000000000003";

pub fn config() -> Value {
    json!({"bktContext":"work", "linearWorkspace":"work", "repositories":[{"workspace":"acme", "repository":"app", "teamKey":"ENG", "teamId":TEAM}]})
}

pub fn invoke(args: &[&str], directory: Option<&Path>) -> Result<Output, Box<dyn Error>> {
    let mut command = Command::new(env!("CARGO_BIN_EXE_bitbucket-linear-sync"));
    command.args(args).env(
        "PATH",
        directory.map_or_else(String::new, |d| d.to_string_lossy().into_owned()),
    );
    if let Some(directory) = directory {
        command.env("FIXTURE_ROOT", directory);
    }
    Ok(command.output()?)
}

fn provider() -> Result<std::path::PathBuf, Box<dyn Error>> {
    let executable = std::env::current_exe()?;
    let profile = executable
        .parent()
        .and_then(Path::parent)
        .ok_or("test profile directory")?;
    let provider = profile.join("examples/provider");
    if !provider.is_file() {
        return Err("Cargo fixture example unavailable".into());
    }
    Ok(provider)
}

pub fn pull_request(id: u64, identifier: &str, state: &str) -> Value {
    let repo = json!({"uuid":"{00000000-0000-4000-8000-000000000011}","full_name":"acme/app"});
    json!({"id":id,"title":identifier,"description":"", "state":state,
        "author":{"uuid":"{00000000-0000-4000-8000-000000000012}"},
        "source":{"branch":{"name":format!("{identifier}-fix")},"repository":repo},
        "destination":{"branch":{"name":"main"},"repository":repo},
        "links":{"html":{"href":format!("https://bitbucket.org/acme/app/pull-requests/{id}")}}})
}

pub fn issue(id: &str, identifier: &str) -> Value {
    json!({"id":id,"identifier":identifier,"team":{"id":TEAM},
        "state":{"id":"00000000-0000-4000-8000-000000000004","name":"Todo","type":"unstarted"},
        "archivedAt":null,"attachments":{"nodes":[],"pageInfo":{"hasNextPage":false,"endCursor":null}}})
}

pub fn store() -> Value {
    let done =
        json!({"id":"00000000-0000-4000-8000-000000000002","name":"Done","type":"completed"});
    json!({"contexts":{"contexts":[{"name":"work","host":"api.bitbucket.org"}]},
        "user":{"uuid":"{00000000-0000-4000-8000-000000000012}"},
        "identity":{"viewer":{"id":TEAM},"organization":{"id":TEAM}},
        "repository":{"uuid":"{00000000-0000-4000-8000-000000000011}","full_name":"acme/app"},
        "pages":[{"values":[pull_request(1,"ENG-1","MERGED")]}],
        "teamPages":[{"id":TEAM,"key":"ENG","states":{"nodes":[done],"pageInfo":{"hasNextPage":false,"endCursor":null}}}],
        "issues":[issue(ISSUE,"ENG-1"),issue("00000000-0000-4000-8000-000000000005","ENG-2")],"done":done,"overrides":{},"failMutationIssue":null,"graphqlErrors":false,"inventoryFailure":false,"loseAttachment":false,"removeOnRead":null,"issueReadCount":0,"attachmentPages":null})
}

pub struct Fixture {
    pub dir: tempfile::TempDir,
}

impl Fixture {
    pub fn new(store: &Value) -> Result<Self, Box<dyn Error>> {
        let dir = tempfile::tempdir()?;
        for name in ["bkt", "linear"] {
            fs::copy(provider()?, dir.path().join(name))?;
        }
        fs::write(dir.path().join("store.json"), store.to_string())?;
        fs::write(dir.path().join("config.json"), config().to_string())?;
        Ok(Self { dir })
    }
    pub fn run(&self, apply: bool) -> Result<Output, Box<dyn Error>> {
        let config = self.dir.path().join("config.json");
        let mut args = vec!["--config", config.to_str().ok_or("config path")?, "--json"];
        if apply {
            args.push("--apply");
        }
        invoke(&args, Some(self.dir.path()))
    }
    pub fn read(&self) -> Result<Value, Box<dyn Error>> {
        Ok(serde_json::from_slice(&fs::read(
            self.dir.path().join("store.json"),
        )?)?)
    }
    pub fn write(&self, value: &Value) -> Result<(), Box<dyn Error>> {
        Ok(fs::write(
            self.dir.path().join("store.json"),
            value.to_string(),
        )?)
    }
    pub fn calls(&self) -> Result<String, Box<dyn Error>> {
        Ok(fs::read_to_string(self.dir.path().join("calls"))?)
    }
}

pub fn output_value(output: &Output) -> Result<(Value, Value), Box<dyn Error>> {
    let text = std::str::from_utf8(&output.stdout)?;
    let mut lines = text.lines();
    let plan = serde_json::from_str(lines.next().ok_or("plan absent")?)?;
    let result = serde_json::from_str(lines.next().ok_or("result absent")?)?;
    assert!(lines.next().is_none());
    Ok((plan, result))
}
