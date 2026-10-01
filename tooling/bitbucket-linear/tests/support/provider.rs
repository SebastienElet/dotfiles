mod json;
use json::{at, at_mut, put};
use serde_json::{Value, json};
use std::{
    env,
    error::Error,
    fs,
    io::{self, Write},
    path::PathBuf,
};

fn main() -> Result<(), Box<dyn Error>> {
    let root = PathBuf::from(env::var("FIXTURE_ROOT")?);
    let path = root.join("store.json");
    let mut store: Value = serde_json::from_slice(&fs::read(&path)?)?;
    let args: Vec<String> = env::args().skip(1).collect();
    let query = args.get(1).ok_or("missing request")?;
    let name = env::current_exe()?
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or("name")?
        .to_owned();
    let context = if name == "bkt" {
        "--context"
    } else {
        "--workspace"
    };
    if query != "list"
        && !args.windows(2).any(|a| {
            a.first().is_some_and(|v| v == context) && a.get(1).is_some_and(|v| v == "work")
        })
    {
        return Err("wrong explicit context".into());
    }
    let mut calls = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(root.join("calls"))?;
    writeln!(calls, "{name} {query}")?;
    let override_value = at(&store, "/overrides")?
        .as_object()
        .and_then(|rows| {
            rows.iter()
                .find(|(prefix, _)| query.starts_with(prefix.as_str()))
        })
        .map(|(_, response)| response.clone());
    let result = if let Some(response) = override_value {
        response
    } else if name == "bkt" {
        bitbucket(&store, query)?
    } else {
        linear(&mut store, query, &args)?
    };
    fs::write(path, serde_json::to_vec(&store)?)?;
    writeln!(io::stdout().lock(), "{result}")?;
    Ok(())
}

fn bitbucket(store: &Value, path: &str) -> Result<Value, Box<dyn Error>> {
    if path == "list" {
        return Ok(at(store, "/contexts")?.clone());
    }
    if path == "/2.0/user" {
        return Ok(at(store, "/user")?.clone());
    }
    if path.contains("pullrequests") {
        if at(store, "/inventoryFailure")?.as_bool() == Some(true) || path.contains("other") {
            return Err("inventory failed".into());
        }
        let index = usize::from(path.contains("cursor="));
        return at(store, "/pages")?
            .get(index)
            .cloned()
            .ok_or_else(|| "page unavailable".into());
    }
    Ok(at(store, "/repository")?.clone())
}

fn linear(store: &mut Value, query: &str, args: &[String]) -> Result<Value, Box<dyn Error>> {
    if at(store, "/graphqlErrors")?.as_bool() == Some(true) {
        return Ok(json!({"data": {}, "errors": [{"message":"partial"}]}));
    }
    if query.starts_with("query Identity") {
        return Ok(json!({"data": at(store, "/identity")?}));
    }
    let variables: Value = serde_json::from_str(args.last().ok_or("variables")?)?;
    if query.starts_with("query Team") {
        let index = usize::from(!at(&variables, "/after")?.is_null());
        return Ok(
            json!({"data": {"team": at(store, "/teamPages")?.get(index).ok_or("team page")?}}),
        );
    }
    let requested = if query.starts_with("mutation Attach") {
        &at(&variables, "/input/issueId")?
    } else {
        &at(&variables, "/id")?
    };
    let issues = at(store, "/issues")?.as_array().ok_or("issues")?;
    let index = issues
        .iter()
        .position(|i| i.get("id") == Some(requested) || i.get("identifier") == Some(requested))
        .ok_or("issue missing")?;
    if query.starts_with("query Issue") {
        if let Some(threshold) = at(store, "/removeOnRead")?.as_u64() {
            let count = at(store, "/issueReadCount")?.as_u64().unwrap_or(0) + 1;
            put(json!(count), store, "/issueReadCount")?;
            if count >= threshold {
                put(
                    json!([]),
                    store,
                    &format!("/issues/{index}/attachments/nodes"),
                )?;
            }
        }
        let mut issue = at(store, "/issues")?.get(index).ok_or("issue")?.clone();
        if let Some(pages) = at(store, "/attachmentPages")?.as_array() {
            let page = usize::from(!at(&variables, "/after")?.is_null());
            put(
                pages.get(page).ok_or("attachment page")?.clone(),
                &mut issue,
                "/attachments",
            )?;
        }
        return Ok(json!({"data": {"issue": issue}}));
    }
    if at(store, "/failMutationIssue")? == *requested {
        return Err("mutation failed".into());
    }
    if query.starts_with("mutation Attach") {
        let input = &at(&variables, "/input")?;
        let attachment = json!({"id":format!("link-{}", at(input, "/url")?), "url":at(input, "/url")?, "title":at(input, "/title")?, "subtitle":at(input, "/subtitle")?});
        if at(store, "/loseAttachment")?.as_bool() != Some(true) {
            let nodes = at_mut(store, &format!("/issues/{index}/attachments/nodes"))?
                .as_array_mut()
                .ok_or("nodes")?;
            nodes.retain(|node| node.get("url") != input.get("url"));
            nodes.push(attachment.clone());
        }
        return Ok(json!({"data":{"attachmentCreate":{"success":true,"attachment":attachment}}}));
    }
    if query.starts_with("mutation Complete") {
        let mut completed = at(store, "/done")?.clone();
        put(
            at(&variables, "/input/stateId")?.clone(),
            &mut completed,
            "/id",
        )?;
        put(completed, store, &format!("/issues/{index}/state"))?;
        return Ok(
            json!({"data":{"issueUpdate":{"success":true,"issue":at(store, "/issues")?.get(index).ok_or("issue")?}}}),
        );
    }
    Err("unexpected request".into())
}
