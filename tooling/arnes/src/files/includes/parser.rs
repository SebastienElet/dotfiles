pub fn imports(contents: &str) -> Vec<String> {
    let mut fence = None;
    contents
        .lines()
        .flat_map(|line| {
            if let Some((marker, length)) = fence_marker(line) {
                match fence {
                    Some((open, minimum)) if marker == open && length >= minimum => fence = None,
                    None => fence = Some((marker, length)),
                    _ => {}
                }
                return Vec::new();
            }
            if fence.is_some() {
                Vec::new()
            } else {
                imports_in_line(line)
            }
        })
        .collect()
}

pub fn leading_imports(contents: &str) -> Vec<String> {
    let mut fence = None;
    contents
        .lines()
        .filter(|line| leading_import(line, &mut fence))
        .flat_map(imports_in_line)
        .collect()
}

pub fn without_leading_imports(contents: &str) -> String {
    let mut fence = None;
    contents
        .split_inclusive('\n')
        .filter(|line| !leading_import(line.trim_end_matches('\n'), &mut fence))
        .collect()
}

fn leading_import(line: &str, fence: &mut Option<(u8, usize)>) -> bool {
    if let Some((marker, length)) = fence_marker(line) {
        match *fence {
            Some((open, minimum)) if marker == open && length >= minimum => *fence = None,
            None => *fence = Some((marker, length)),
            _ => {}
        }
        return false;
    }
    fence.is_none() && line.starts_with('@') && !imports_in_line(line).is_empty()
}

fn imports_in_line(line: &str) -> Vec<String> {
    let bytes = line.as_bytes();
    let mut imports = Vec::new();
    let mut index = 0;
    let mut code_ticks = None;

    while index < bytes.len() {
        if bytes[index] == b'`' {
            let length = bytes[index..]
                .iter()
                .take_while(|byte| **byte == b'`')
                .count();
            match code_ticks {
                Some(open) if open == length => code_ticks = None,
                None => code_ticks = Some(length),
                _ => {}
            }
            index += length;
            continue;
        }
        if bytes[index] != b'@'
            || code_ticks.is_some()
            || index > 0 && bytes[index - 1].is_ascii_alphanumeric()
        {
            index += 1;
            continue;
        }
        let start = index + 1;
        let mut end = start;
        while end < bytes.len()
            && (bytes[end].is_ascii_alphanumeric() || b"./_~-".contains(&bytes[end]))
        {
            end += 1;
        }
        if end > start {
            imports.push(line[start..end].to_owned());
        }
        index = end.max(index + 1);
    }
    imports
}

fn fence_marker(line: &str) -> Option<(u8, usize)> {
    let bytes = line.trim_start().as_bytes();
    let marker = *bytes.first()?;
    if !matches!(marker, b'`' | b'~') {
        return None;
    }
    let length = bytes.iter().take_while(|byte| **byte == marker).count();
    (length >= 3).then_some((marker, length))
}
