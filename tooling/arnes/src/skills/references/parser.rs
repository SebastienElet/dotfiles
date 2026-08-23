use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

pub fn local_references(contents: &str) -> BTreeSet<PathBuf> {
    let visible = without_fenced_code(contents);
    let mut references = markdown_targets(&visible)
        .into_iter()
        .filter_map(markdown_candidate)
        .collect::<BTreeSet<_>>();
    references.extend(resource_tokens(&visible).filter_map(token_candidate));
    references
}

fn without_fenced_code(contents: &str) -> String {
    let mut visible = String::new();
    let mut fence = None;
    for line in contents.lines() {
        if let Some(marker) = fence_marker(line) {
            match fence {
                None => fence = Some(marker),
                Some((character, length)) if marker.0 == character && marker.1 >= length => {
                    fence = None;
                }
                Some(_) => {}
            }
        } else if fence.is_none() {
            visible.push_str(line);
            visible.push('\n');
        }
    }
    visible
}

fn fence_marker(line: &str) -> Option<(char, usize)> {
    let mut characters = line.trim_start().chars();
    let character = characters.next()?;
    if !matches!(character, '`' | '~') {
        return None;
    }
    let length = 1 + characters
        .take_while(|candidate| *candidate == character)
        .count();
    (length >= 3).then_some((character, length))
}

fn markdown_targets(contents: &str) -> Vec<String> {
    let mut targets = Vec::new();
    let mut search = 0;
    while let Some(offset) = contents[search..].find("](") {
        let start = search + offset + 2;
        let Some((end, target)) = balanced_target(contents, start) else {
            break;
        };
        targets.push(target);
        search = end + 1;
    }
    targets
}

fn balanced_target(contents: &str, start: usize) -> Option<(usize, String)> {
    let mut depth = 1;
    let mut escaped = false;
    for (offset, character) in contents[start..].char_indices() {
        if escaped {
            escaped = false;
            continue;
        }
        match character {
            '\\' => escaped = true,
            '(' => depth += 1,
            ')' if depth == 1 => {
                let end = start + offset;
                return Some((end, contents[start..end].to_owned()));
            }
            ')' => depth -= 1,
            _ => {}
        }
    }
    None
}

fn markdown_candidate(value: String) -> Option<PathBuf> {
    let value = value.trim();
    let value = if let Some(value) = value.strip_prefix('<') {
        value.split_once('>')?.0
    } else {
        value.split_whitespace().next().unwrap_or(value)
    };
    relative_candidate(value).map(PathBuf::from)
}

fn resource_tokens(contents: &str) -> impl Iterator<Item = &str> {
    contents
        .split_whitespace()
        .filter(|token| !token.contains("]("))
        .map(trim_token)
}

fn trim_token(token: &str) -> &str {
    token
        .trim_matches(|character: char| {
            matches!(
                character,
                '`' | '\'' | '"' | '(' | ')' | '[' | ']' | '{' | '}' | '<' | '>' | ',' | ';'
            )
        })
        .trim_end_matches(['.', ':'])
}

fn token_candidate(value: &str) -> Option<PathBuf> {
    let value = relative_candidate(value)?;
    let explicit = value.starts_with("./") || value.starts_with("../");
    let resource = ["agents/", "assets/", "evals/", "references/", "scripts/"]
        .iter()
        .any(|prefix| value.starts_with(prefix));
    (explicit || resource).then(|| Path::new(value).to_owned())
}

fn relative_candidate(value: &str) -> Option<&str> {
    let value = value
        .split('#')
        .next()?
        .trim_end_matches(|character: char| {
            matches!(
                character,
                '`' | '\'' | '"' | ')' | ']' | '}' | '>' | ',' | ';' | '.' | ':'
            )
        });
    if value.is_empty()
        || value.starts_with(['/', '~', '#'])
        || value.contains("://")
        || value.starts_with("mailto:")
    {
        None
    } else {
        Some(value)
    }
}

#[cfg(test)]
mod tests {
    use super::local_references;
    use std::path::PathBuf;

    #[test]
    fn extracts_relative_links_and_resource_tokens() {
        let contents = "[root](guide.md) [nested](references/a(b).md) `scripts/run.sh`";

        assert_eq!(
            local_references(contents).into_iter().collect::<Vec<_>>(),
            vec![
                PathBuf::from("guide.md"),
                PathBuf::from("references/a(b).md"),
                PathBuf::from("scripts/run.sh")
            ]
        );
    }

    #[test]
    fn ignores_external_and_fenced_examples() {
        let contents = "[web](https://example.com/a.md) [anchor](#a)\n```sh\nscripts/no.sh\n```";

        assert!(local_references(contents).is_empty());
    }
}
