use super::*;
#[test]
fn documented_fields_and_unknown_extensions_are_preserved()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    assert_codex_fields_preserved()?;
    assert_claude_fields_preserved()?;
    assert_cursor_fields_preserved()?;
    Ok(())
}
fn assert_codex_fields_preserved() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let harness = Harness::new()?;
    harness.write("codex", &codex_documented_fields())?;
    assert_success(&harness.install("codex")?, "codex SessionEnd command valid");
    let codex: Value = serde_json::from_slice(&fs::read(harness.config("codex")?)?)?;
    let hooks = (*(*(*(codex).get("hooks").ok_or("missing fixture index hooks")?)
        .get("SessionEnd")
        .ok_or("missing fixture index SessionEnd")?)
    .get(0)
    .ok_or("missing fixture index 0")?)
    .get("hooks")
    .ok_or("missing fixture index hooks")?;
    assert_eq!(
        *(*(*(hooks).get(0).ok_or("missing fixture index 0")?)
            .get("futureField")
            .ok_or("missing fixture index futureField")?)
        .get("x")
        .ok_or("missing fixture index x")?,
        1
    );
    assert_eq!(
        *(*(hooks).get(1).ok_or("missing fixture index 1")?)
            .get("command_windows")
            .ok_or("missing fixture index command_windows")?,
        "other.exe"
    );
    Ok(())
}
fn assert_claude_fields_preserved() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let harness = Harness::new()?;
    harness.write("claude-code", &claude_documented_fields())?;
    assert_success(
        &harness.install("claude-code")?,
        "claude Stop all variants valid",
    );
    let claude: Value = serde_json::from_slice(&fs::read(harness.config("claude-code")?)?)?;
    let hooks = (*(*(*(*(claude).get("hooks").ok_or("missing fixture index hooks")?)
        .get("Stop")
        .ok_or("missing fixture index Stop")?)
    .get(0)
    .ok_or("missing fixture index 0")?)
    .get("hooks")
    .ok_or("missing fixture index hooks")?)
    .as_array()
    .ok_or("expected JSON array")?;
    assert_eq!(
        *(*(*(hooks).first().ok_or("missing fixture index 0")?)
            .get("futureField")
            .ok_or("missing fixture index futureField")?)
        .get("x")
        .ok_or("missing fixture index x")?,
        1
    );
    assert_eq!(hooks.len(), 5);
    Ok(())
}
fn assert_cursor_fields_preserved() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let harness = Harness::new()?;
    harness.write("cursor", &cursor_documented_fields())?;
    assert_success(
        &harness.install("cursor")?,
        "cursor stop command valid object matcher",
    );
    let cursor: Value = serde_json::from_slice(&fs::read(harness.config("cursor")?)?)?;
    assert_eq!(
        *(*(*(*(*(cursor).get("hooks").ok_or("missing fixture index hooks")?)
            .get("stop")
            .ok_or("missing fixture index stop")?)
        .get(0)
        .ok_or("missing fixture index 0")?)
        .get("futureField")
        .ok_or("missing fixture index futureField")?)
        .get("x")
        .ok_or("missing fixture index x")?,
        1
    );
    assert_eq!(
        *(cursor)
            .get("version")
            .ok_or("missing fixture index version")?,
        1.5
    );
    Ok(())
}
fn codex_documented_fields() -> Value {
    json ! ({ "hooks" : { "SessionEnd" : [{ "hooks" : [{ "type" : "command" , "command" : "third-party" , "timeout" : 3 , "statusMessage" : "done" , "additionalContextLimit" : 0 , "commandWindows" : "third-party.exe" , "async" : false , "futureField" : { "x" : 1 } } , { "type" : "command" , "command" : "other" , "command_windows" : "other.exe" }] }] } })
}
fn claude_documented_fields() -> Value {
    json ! ({ "hooks" : { "Stop" : [{ "hooks" : [{ "type" : "command" , "command" : "third-party" , "args" : ["a"] , "async" : true , "asyncRewake" : true , "shell" : "bash" , "timeout" : 1 , "statusMessage" : "running" , "once" : false , "if" : "Bash(*)" , "futureField" : { "x" : 1 } } , { "type" : "http" , "url" : "https://example.test" , "headers" : { "x" : "$TOKEN" } , "allowedEnvVars" : ["TOKEN"] } , { "type" : "mcp_tool" , "server" : "s" , "tool" : "t" , "input" : ["opaque"] } , { "type" : "prompt" , "prompt" : "review" , "model" : "fast" , "continueOnBlock" : true } , { "type" : "agent" , "prompt" : "investigate" , "model" : "fast" }] }] } })
}
fn cursor_documented_fields() -> Value {
    json ! ({ "version" : 1.5 , "hooks" : { "stop" : [{ "command" : "third-party" , "timeout" : 1 , "loop_limit" : null , "failClosed" : true , "matcher" : { "type" : "Shell" } , "futureField" : { "x" : 1 } } , { "type" : "prompt" , "prompt" : "review" , "model" : "fast" , "timeout" : 1 , "loop_limit" : 5 , "failClosed" : false , "matcher" : "Stop" }] } })
}
#[test]
fn future_event_and_variant_extensions_remain_opaque()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let _: () = for (agent, event, value) in codex_future_extensions()
        .into_iter()
        .chain(other_future_extensions())
    {
        assert_future_extension_preserved(agent, event, &value)?;
    };
    Ok(())
}
fn codex_future_extensions() -> Vec<(&'static str, &'static str, Value)> {
    vec![
        (
            "codex",
            "FutureEvent",
            nested(
                "FutureEvent",
                &json ! ({ "type" : "http" , "timeout" : "future" , "future" : true }),
            ),
        ),
        (
            "codex",
            "Stop",
            nested(
                "Stop",
                &json ! ({ "type" : "http" , "timeout" : "future" , "future" : true }),
            ),
        ),
        (
            "codex",
            "Stop",
            nested(
                "Stop",
                &json ! ({ "type" : "agent" , "timeout" : "future" , "future" : true }),
            ),
        ),
        (
            "codex",
            "Stop",
            nested(
                "Stop",
                &json ! ({ "type" : "prompt" , "command" : "future" , "timeout" : "future" , "future" : true }),
            ),
        ),
    ]
}
fn other_future_extensions() -> Vec<(&'static str, &'static str, Value)> {
    vec![
        (
            "claude-code",
            "FutureEvent",
            nested(
                "FutureEvent",
                &json ! ({ "type" : "future" , "timeout" : "future" , "future" : true }),
            ),
        ),
        (
            "claude-code",
            "Stop",
            nested(
                "Stop",
                &json ! ({ "type" : "future" , "timeout" : "future" , "future" : true }),
            ),
        ),
        (
            "cursor",
            "futureEvent",
            direct(
                "futureEvent",
                &json ! ({ "type" : "future" , "timeout" : "future" , "future" : true }),
            ),
        ),
        (
            "cursor",
            "stop",
            direct(
                "stop",
                &json ! ({ "type" : "future" , "timeout" : "future" , "future" : true }),
            ),
        ),
    ]
}
fn assert_future_extension_preserved(
    agent: &str,
    event: &str,
    value: &Value,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let harness = Harness::new()?;
    harness.write(agent, value)?;
    assert_success(&harness.install(agent)?, agent);
    let installed: Value = serde_json::from_slice(&fs::read(harness.config(agent)?)?)?;
    let handler = match agent {
        "cursor" => (*(*(installed)
            .get("hooks")
            .ok_or("missing fixture index hooks")?)
        .get(event)
        .ok_or("missing fixture index event")?)
        .get(0)
        .ok_or("missing fixture index 0")?,
        _ => (*(*(*(*(installed)
            .get("hooks")
            .ok_or("missing fixture index hooks")?)
        .get(event)
        .ok_or("missing fixture index event")?)
        .get(0)
        .ok_or("missing fixture index 0")?)
        .get("hooks")
        .ok_or("missing fixture index hooks")?)
        .get(0)
        .ok_or("missing fixture index 0")?,
    };
    assert_eq!(
        *(handler)
            .get("timeout")
            .ok_or("missing fixture index timeout")?,
        "future"
    );
    assert_eq!(
        *(handler)
            .get("future")
            .ok_or("missing fixture index future")?,
        true
    );
    Ok(())
}
