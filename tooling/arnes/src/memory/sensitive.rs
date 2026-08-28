pub(crate) fn contains_sensitive(value: &str) -> bool {
    let lowercase = value.to_ascii_lowercase();
    has_private_pem(&lowercase)
        || has_url_userinfo(&lowercase)
        || has_authorization_header(&lowercase)
        || has_secret_assignment(&lowercase)
        || has_credential_prefix(&lowercase)
        || has_system_prompt_marker(&lowercase)
        || has_transcript_block(&lowercase)
}

fn has_private_pem(value: &str) -> bool {
    value.match_indices("-----begin ").any(|(index, _)| {
        value[index..]
            .lines()
            .next()
            .is_some_and(|line| line.contains("private key-----"))
    })
}

fn has_url_userinfo(value: &str) -> bool {
    value.match_indices("://").any(|(index, _)| {
        value[index + 3..]
            .split(|character: char| {
                character == '/'
                    || character == '?'
                    || character == '#'
                    || character.is_whitespace()
            })
            .next()
            .is_some_and(|authority| authority.contains('@'))
    })
}

fn has_authorization_header(value: &str) -> bool {
    value.match_indices("authorization").any(|(index, _)| {
        has_left_boundary(value, index)
            && has_right_boundary(&value[index..], 13)
            && value[index + 13..].trim_start().starts_with(':')
    })
}

fn has_secret_assignment(value: &str) -> bool {
    value.lines().any(|line| {
        ["password", "secret", "token"]
            .into_iter()
            .any(|key| has_assignment(line, key))
            || has_api_key_assignment(line)
    })
}

fn has_assignment(line: &str, key: &str) -> bool {
    line.match_indices(key).any(|(index, _)| {
        has_word_boundary(line, index, key.len())
            && line[index + key.len()..]
                .trim_start()
                .starts_with(['=', ':'])
    })
}

fn has_api_key_assignment(line: &str) -> bool {
    line.match_indices("api").any(|(index, _)| {
        if !has_left_boundary(line, index) {
            return false;
        }
        let separator = &line[index + 3..];
        let separator_length = separator
            .chars()
            .take_while(|character| {
                matches!(character, '_' | '-' | '.') || character.is_whitespace()
            })
            .map(char::len_utf8)
            .sum::<usize>();
        if separator_length == 0 {
            return false;
        }
        let key = &separator[separator_length..];
        key.get(..3).is_some_and(|value| value == "key")
            && has_right_boundary(key, 3)
            && key[3..].trim_start().starts_with(['=', ':'])
    })
}

fn has_word_boundary(value: &str, index: usize, length: usize) -> bool {
    has_left_boundary(value, index) && has_right_boundary(&value[index..], length)
}

fn has_left_boundary(value: &str, index: usize) -> bool {
    index == 0
        || value[..index]
            .chars()
            .next_back()
            .is_some_and(|character| !character.is_ascii_alphanumeric() && character != '_')
}

fn has_right_boundary(value: &str, index: usize) -> bool {
    value[index..]
        .chars()
        .next()
        .is_none_or(|character| !character.is_ascii_alphanumeric() && character != '_')
}

fn has_credential_prefix(value: &str) -> bool {
    [
        "sk-",
        "ghp_",
        "github_pat_",
        "xoxb-",
        "xoxa-",
        "xoxp-",
        "xoxr-",
        "xoxs-",
    ]
    .into_iter()
    .any(|prefix| {
        value
            .match_indices(prefix)
            .any(|(index, _)| has_left_boundary(value, index))
    })
}

fn has_system_prompt_marker(value: &str) -> bool {
    [
        "system prompt:",
        "<|system|>",
        "[system]",
        "begin system prompt",
    ]
    .into_iter()
    .any(|marker| value.contains(marker))
}

fn has_transcript_block(value: &str) -> bool {
    value
        .lines()
        .filter(|line| has_role_prefix(line))
        .take(2)
        .count()
        == 2
}

fn has_role_prefix(line: &str) -> bool {
    let line = line.trim_start_matches(|character: char| {
        character.is_whitespace() || matches!(character, '#' | '>' | '-' | '*')
    });
    ["user", "assistant", "system"].into_iter().any(|role| {
        line.strip_prefix(role)
            .is_some_and(|rest| rest.trim_start().starts_with(':'))
    })
}
