pub(crate) fn contains_shell_command(value: &str) -> bool {
    value.contains("$(")
        || value.lines().any(|line| {
            has_shell_shebang(line)
                || has_shell_fence(line)
                || has_shell_prompt(line)
                || has_interpreter_invocation(line)
                || has_pipe_to_shell(line)
        })
}

fn has_shell_shebang(line: &str) -> bool {
    let Some(invocation) = line.trim_start().strip_prefix("#!") else {
        return false;
    };
    let mut tokens = invocation.split_ascii_whitespace();
    let Some(executable) = tokens.next() else {
        return false;
    };
    shell_executable(executable)
        || executable_name(executable) == "env" && tokens.any(shell_executable)
}

fn has_shell_fence(line: &str) -> bool {
    let line = line.trim_start();
    ["```", "~~~"].into_iter().any(|fence| {
        line.strip_prefix(fence)
            .and_then(|suffix| suffix.split_ascii_whitespace().next())
            .is_some_and(shell_language)
    })
}

fn has_shell_prompt(line: &str) -> bool {
    let line = line.trim_start();
    line.starts_with("$ ") || line.starts_with("% ")
}

fn has_interpreter_invocation(line: &str) -> bool {
    let mut tokens = line.trim_start().split_ascii_whitespace();
    let Some(executable) = tokens.next() else {
        return false;
    };
    shell_executable(executable) && tokens.next() == Some("-c") && tokens.next().is_some()
}

fn has_pipe_to_shell(line: &str) -> bool {
    line.split('|').skip(1).any(|suffix| {
        suffix
            .split_ascii_whitespace()
            .next()
            .is_some_and(shell_executable)
    })
}

fn shell_language(value: &str) -> bool {
    matches!(
        value.to_ascii_lowercase().as_str(),
        "sh" | "bash" | "zsh" | "fish" | "shell"
    )
}

fn shell_executable(value: &str) -> bool {
    matches!(executable_name(value), "sh" | "bash" | "zsh" | "fish")
}

fn executable_name(value: &str) -> &str {
    value.rsplit('/').next().unwrap_or(value)
}
