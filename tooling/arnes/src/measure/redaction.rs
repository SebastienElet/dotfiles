use serde_json::Value;

const MARKER: &str = "[REDACTED]";

pub fn redact(value: &mut Value) {
    match value {
        Value::Object(fields) => {
            for (key, value) in fields {
                if sensitive_key(key) {
                    *value = Value::String(MARKER.to_owned());
                } else {
                    redact(value);
                }
            }
        }
        Value::Array(values) => values.iter_mut().for_each(redact),
        Value::String(value) => *value = redact_string(value),
        _ => {}
    }
}

pub fn redact_string(value: &str) -> String {
    if contains_private_key(value) {
        return MARKER.to_owned();
    }
    let mut result = value.to_owned();
    for prefix in [
        "bearer ",
        "sk-",
        "ghp_",
        "github_pat_",
        "glpat-",
        "akia",
        "aiza",
        "npm_",
        "xoxa-",
        "xoxb-",
        "xoxp-",
        "xoxr-",
        "xoxs-",
    ] {
        result = redact_prefix(&result, prefix);
    }
    result
}

fn sensitive_key(key: &str) -> bool {
    let normalized: String = key
        .chars()
        .filter(|character| character.is_ascii_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect();
    matches!(
        normalized.as_str(),
        "apikey"
            | "apitoken"
            | "accesskey"
            | "accesstoken"
            | "refreshtoken"
            | "idtoken"
            | "authtoken"
            | "authorization"
            | "cookie"
            | "password"
            | "passwd"
            | "privatekey"
            | "secret"
            | "token"
            | "thought"
            | "reasoning"
            | "chainofthought"
            | "thoughts"
            | "thinking"
    ) || normalized.contains("secret")
        || normalized.starts_with("reasoning")
        || normalized == "analysis"
        || normalized.ends_with("credentials")
        || ["password", "passwd", "token", "privatekey", "accesskey"]
            .iter()
            .any(|suffix| normalized.ends_with(suffix))
}

fn contains_private_key(value: &str) -> bool {
    let uppercase = value.to_ascii_uppercase();
    uppercase.contains("-----BEGIN") && uppercase.contains("PRIVATE KEY-----")
}

fn redact_prefix(value: &str, prefix: &str) -> String {
    let mut output = String::new();
    let mut remaining = value;
    while let Some(start) = remaining.to_ascii_lowercase().find(prefix) {
        output.push_str(&remaining[..start]);
        let token_start = start + prefix.len();
        let token_len = remaining[token_start..]
            .chars()
            .take_while(|character| token_character(*character))
            .map(char::len_utf8)
            .sum::<usize>();
        if token_len == 0 {
            output.push_str(&remaining[start..token_start]);
            remaining = &remaining[token_start..];
        } else {
            output.push_str(MARKER);
            remaining = &remaining[token_start + token_len..];
        }
    }
    output.push_str(remaining);
    output
}

fn token_character(character: char) -> bool {
    character.is_ascii_alphanumeric() || matches!(character, '.' | '_' | '-' | '/' | '+')
}
