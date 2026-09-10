use serde_json::json;
use std::{
    env,
    fs::{self, OpenOptions},
    io::{self, Write},
    path::Path,
};

/// # Errors
/// Returns errors resolving the fixture workspace or appending the tool observation log.
pub fn invoke_shim(tool: &str, args: &[String]) -> Result<i32, String> {
    let workspace =
        env::var_os("HARNESS_EVAL_WORKSPACE").ok_or("Missing fixture instrumentation")?;
    let log = env::var_os("HARNESS_EVAL_OBSERVATIONS").ok_or("Missing fixture instrumentation")?;
    let cwd = env::current_dir().map_err(|error| error.to_string())?;
    let arguments = public_arguments(tool, args, Path::new(&workspace), &cwd);
    let (output, exit_code) = invoke(tool, args, &cwd);
    let observation = json!({ "tool": tool, "args": arguments, "exitCode": exit_code });
    let line = format!("{observation}\n");
    OpenOptions::new()
        .create(true)
        .append(true)
        .open(log)
        .and_then(|mut file| file.write_all(line.as_bytes()))
        .map_err(|error| error.to_string())?;
    io::stdout()
        .write_all(output.as_bytes())
        .map_err(|error| error.to_string())?;
    Ok(exit_code)
}

fn public_arguments(tool: &str, args: &[String], workspace: &Path, cwd: &Path) -> Vec<String> {
    args.iter()
        .map(|argument| {
            let normalized = if tool == "cat" {
                normalized_target(argument, workspace, cwd)
            } else {
                argument.clone()
            };
            if [
                ".agents/skills/code-search/SKILL.md",
                "src/auth/session.ts",
                "FEATURE_FLAG_DISABLED",
                "--files",
            ]
            .contains(&normalized.as_str())
            {
                normalized
            } else {
                "<other>".into()
            }
        })
        .collect()
}

fn normalized_target(argument: &str, workspace: &Path, cwd: &Path) -> String {
    let target = fs::canonicalize(cwd.join(argument));
    let root = fs::canonicalize(workspace);
    match (target, root) {
        (Ok(target), Ok(root)) => target
            .strip_prefix(root)
            .ok()
            .and_then(Path::to_str)
            .unwrap_or("<other>")
            .into(),
        _ => "<unresolved>".into(),
    }
}

fn invoke(tool: &str, args: &[String], cwd: &Path) -> (String, i32) {
    if tool == "cat" {
        let content: io::Result<Vec<_>> =
            args.iter().map(|path| fs::read(cwd.join(path))).collect();
        return content.map_or_else(
            |_| (String::new(), 1),
            |files| {
                (
                    files
                        .iter()
                        .map(|bytes| String::from_utf8_lossy(bytes))
                        .collect(),
                    0,
                )
            },
        );
    }
    if tool == "rg"
        && args
            .iter()
            .any(|argument| argument == "FEATURE_FLAG_DISABLED")
    {
        return (
            "src/flags.ts:1:export const FEATURE_FLAG_DISABLED = false;\n".into(),
            0,
        );
    }
    if tool == "fd" || (tool == "rg" && args.iter().any(|argument| argument == "--files")) {
        return ("packages/app/package.json\npackages/auth/package.json\nsrc/auth/session.ts\nsrc/flags.ts\n".into(), 0);
    }
    if tool == "colgrep-search" && args.iter().any(|argument| !argument.starts_with('-')) {
        let output = json!({ "results": [
            { "path": "packages/app/package.json", "content": "{\"name\":\"app\",\"dependencies\":{\"auth\":\"workspace:*\"}}" },
            { "path": "packages/auth/package.json", "content": "{\"name\":\"auth\"}" }
        ] });
        return (format!("{output}\n"), 0);
    }
    ("Unsupported synthetic tool invocation\n".into(), 64)
}

#[cfg(test)]
mod tests;
