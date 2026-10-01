use crate::{
    model::{Fallible, PullRequest, PullRequestState, RepositoryConfig, present},
    providers::{Providers, parse},
};
use serde::Deserialize;
use std::collections::BTreeSet;
use url::Url;
use uuid::Uuid;

#[derive(Deserialize)]
struct Repository {
    uuid: String,
    full_name: String,
}
#[derive(Deserialize)]
struct Branch {
    branch: BranchName,
    repository: Repository,
}
#[derive(Deserialize)]
struct BranchName {
    name: String,
}
#[derive(Deserialize)]
struct Author {
    uuid: String,
}
#[derive(Deserialize)]
struct Link {
    href: String,
}
#[derive(Deserialize)]
struct Links {
    html: Link,
}
#[derive(Deserialize)]
struct RawPullRequest {
    id: u64,
    title: String,
    description: String,
    state: PullRequestState,
    author: Author,
    source: Branch,
    destination: Branch,
    links: Links,
}
#[derive(Deserialize)]
struct Page {
    values: Vec<RawPullRequest>,
    #[serde(default, deserialize_with = "present")]
    next: Option<String>,
    #[serde(default, deserialize_with = "present")]
    size: Option<usize>,
    #[serde(default, deserialize_with = "present")]
    #[serde(rename = "page")]
    number: Option<u64>,
    #[serde(default, deserialize_with = "present")]
    pagelen: Option<usize>,
}

pub fn validate_uuid(value: &str) -> Fallible<()> {
    let raw = value
        .strip_prefix('{')
        .and_then(|v| v.strip_suffix('}'))
        .ok_or("Bitbucket UUID malformed")?;
    if raw.len() != 36 || Uuid::parse_str(raw).is_err() {
        return Err("Bitbucket UUID malformed");
    }
    Ok(())
}

fn validate_repository(repo: &Repository) -> Fallible<()> {
    validate_uuid(&repo.uuid)?;
    let mut parts = repo.full_name.split('/');
    let valid = |s: &str| {
        !s.is_empty()
            && s.bytes()
                .all(|c| c.is_ascii_alphanumeric() || b"_.-".contains(&c))
    };
    if !parts.next().is_some_and(valid)
        || !parts.next().is_some_and(valid)
        || parts.next().is_some()
    {
        return Err("Bitbucket repository name malformed");
    }
    Ok(())
}

fn normalize(
    pr: RawPullRequest,
    config: &RepositoryConfig,
    repository: &Repository,
) -> Fallible<PullRequest> {
    validate_uuid(&pr.author.uuid)?;
    validate_repository(&pr.source.repository)?;
    validate_repository(&pr.destination.repository)?;
    let url = format!(
        "https://bitbucket.org/{}/pull-requests/{}",
        config.identity(),
        pr.id
    );
    if pr.id == 0
        || pr.title.is_empty()
        || pr.source.branch.name.is_empty()
        || pr.destination.branch.name.is_empty()
        || pr.destination.repository.full_name != config.identity()
        || pr.destination.repository.uuid != repository.uuid
        || pr.links.html.href != url
    {
        return Err("pull request identity or required field invalid");
    }
    Ok(PullRequest {
        id: pr.id,
        title: pr.title,
        description: pr.description,
        branch: pr.source.branch.name,
        state: pr.state,
        url,
        repository: config.identity(),
        source_repository: pr.source.repository.full_name,
        source_repository_id: pr.source.repository.uuid,
        destination_repository_id: pr.destination.repository.uuid,
        author_id: pr.author.uuid,
        team_key: config.team_key.clone(),
    })
}

fn next_path(value: &str, root: &str) -> Fallible<String> {
    let next = Url::parse(value).map_err(|_| "Bitbucket next URL malformed")?;
    if next.origin().ascii_serialization() != "https://api.bitbucket.org"
        || !next.username().is_empty()
        || next.password().is_some()
        || next.fragment().is_some()
        || next.path() != format!("{root}/pullrequests")
    {
        return Err("Bitbucket pagination escaped repository");
    }
    Ok(format!(
        "{}{}",
        next.path(),
        next.query().map_or_else(String::new, |q| format!("?{q}"))
    ))
}

pub fn read_pull_requests(
    providers: &Providers<'_>,
    config: &RepositoryConfig,
) -> Fallible<Vec<PullRequest>> {
    let root = format!("/2.0/repositories/{}", config.identity());
    let repo: Repository = parse(providers.bitbucket(&root)?)?;
    validate_repository(&repo)?;
    if repo.full_name != config.identity() {
        return Err("repository identity mismatch");
    }
    let mut path = Some(format!(
        "{root}/pullrequests?state=OPEN&state=MERGED&state=DECLINED&pagelen=100"
    ));
    let mut visited = BTreeSet::new();
    let mut sizes = Vec::new();
    let mut results = Vec::new();
    while let Some(current) = path {
        if !visited.insert(current.clone()) {
            return Err("Bitbucket pagination cycle");
        }
        let page: Page = parse(providers.bitbucket(&current)?)?;
        if page.number == Some(0)
            || page.pagelen == Some(0)
            || page
                .pagelen
                .is_some_and(|length| page.values.len() > length)
        {
            return Err("Bitbucket pagination metadata invalid");
        }
        if let Some(size) = page.size {
            sizes.push(size);
        }
        for pr in page.values {
            results.push(normalize(pr, config, &repo)?);
        }
        path = page
            .next
            .as_deref()
            .map(|value| next_path(value, &root))
            .transpose()?;
    }
    if results
        .iter()
        .map(|pr| &pr.url)
        .collect::<BTreeSet<_>>()
        .len()
        != results.len()
        || sizes.iter().any(|size| *size != results.len())
    {
        return Err("duplicate PR or inventory contradicts advertised total");
    }
    Ok(results)
}
