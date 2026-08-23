pub fn references(contents: &str) -> Result<Vec<String>, ()> {
    let mut references = Vec::new();
    let mut index = 0;
    while index < contents.len() {
        let (reference, next) = reference_at(contents, index);
        references.extend(reference?);
        index = next;
    }
    Ok(references)
}

fn reference_at(contents: &str, index: usize) -> (Result<Option<String>, ()>, usize) {
    let bytes = contents.as_bytes();
    if bytes[index] != b'$' {
        return (Ok(None), index + 1);
    }
    if bytes.get(index + 1) == Some(&b'$') {
        return (Ok(None), index + 2);
    }
    if bytes.get(index + 1).is_some_and(u8::is_ascii_digit) {
        return (Ok(None), positional_end(bytes, index + 2));
    }
    if bytes.get(index + 1) == Some(&b'{') {
        return braced(contents, index);
    }
    named(contents, index)
}

fn positional_end(bytes: &[u8], mut end: usize) -> usize {
    while bytes.get(end).is_some_and(u8::is_ascii_digit) {
        end += 1;
    }
    end
}

fn braced(contents: &str, index: usize) -> (Result<Option<String>, ()>, usize) {
    let bytes = contents.as_bytes();
    let start = index + 2;
    let mut end = start;
    while bytes.get(end).is_some_and(identifier_byte) {
        end += 1;
    }
    if end > start && bytes.get(end) == Some(&b'}') && identifier_start(bytes[start]) {
        (Ok(Some(contents[start..end].to_owned())), end + 1)
    } else {
        (Err(()), contents.len())
    }
}

fn named(contents: &str, index: usize) -> (Result<Option<String>, ()>, usize) {
    let bytes = contents.as_bytes();
    let Some(first) = bytes.get(index + 1) else {
        return (Ok(None), index + 1);
    };
    if !identifier_start(*first) {
        return (Ok(None), index + 1);
    }
    let start = index + 1;
    let mut end = start + 1;
    while bytes.get(end).is_some_and(identifier_byte) {
        end += 1;
    }
    let name = &contents[start..end];
    (Ok((name != "ARGUMENTS").then(|| name.to_owned())), end)
}

fn identifier_start(byte: u8) -> bool {
    byte.is_ascii_alphabetic() || byte == b'_'
}

fn identifier_byte(byte: &u8) -> bool {
    byte.is_ascii_alphanumeric() || *byte == b'_'
}

#[cfg(test)]
mod tests {
    use super::references;

    #[test]
    fn named_variables_ignore_literals_and_claude_arguments() {
        assert_eq!(
            references("$NAME ${OTHER} $$LITERAL $ARGUMENTS $ARGUMENTS[0] $0 $12").unwrap(),
            ["NAME", "OTHER"]
        );
    }

    #[test]
    fn malformed_braced_variables_fail_closed() {
        assert!(references("${NAME:-fallback}").is_err());
        assert!(references("${NAME").is_err());
    }
}
