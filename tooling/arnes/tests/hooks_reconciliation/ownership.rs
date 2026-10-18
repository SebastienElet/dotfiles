use super::*;
#[test]
fn preserves_thought_hooks_and_similar_measurement_commands()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let harness = Harness::new()?;
    let similar = format!("{} --extra", harness.command("cursor")?);
    let excluded = harness.command("cursor")?;
    harness . write_config ("cursor" , & serde_json :: json ! ({ "version" : 1 , "hooks" : { "afterAgentThought" : [{ "command" : "third-party-thought" } , { "type" : "prompt" , "prompt" : excluded } , { "command" : excluded }] , "futureEvent" : [{ "command" : excluded } , { "command" : "third-party-future" }] , "stop" : [{ "command" : similar }] } }) ,) ? ;
    assert_success(&harness.install("cursor")?);
    let config = read_json(harness.config("cursor")?)?;
    assert_eq!(
        *(*(*(*(config).get("hooks").ok_or("missing fixture index hooks")?)
            .get("afterAgentThought")
            .ok_or("missing fixture index afterAgentThought")?)
        .get(0)
        .ok_or("missing fixture index 0")?)
        .get("command")
        .ok_or("missing fixture index command")?,
        "third-party-thought"
    );
    assert_eq!(
        *(*(*(*(config).get("hooks").ok_or("missing fixture index hooks")?)
            .get("afterAgentThought")
            .ok_or("missing fixture index afterAgentThought")?)
        .get(1)
        .ok_or("missing fixture index 1")?)
        .get("type")
        .ok_or("missing fixture index type")?,
        "prompt"
    );
    assert_eq!(
        (*(*(config).get("hooks").ok_or("missing fixture index hooks")?)
            .get("afterAgentThought")
            .ok_or("missing fixture index afterAgentThought")?)
        .as_array()
        .ok_or("expected JSON array")?
        .len(),
        2
    );
    assert_eq!(
        *(*(*(*(config).get("hooks").ok_or("missing fixture index hooks")?)
            .get("futureEvent")
            .ok_or("missing fixture index futureEvent")?)
        .get(0)
        .ok_or("missing fixture index 0")?)
        .get("command")
        .ok_or("missing fixture index command")?,
        "third-party-future"
    );
    assert_eq!(
        (*(*(config).get("hooks").ok_or("missing fixture index hooks")?)
            .get("futureEvent")
            .ok_or("missing fixture index futureEvent")?)
        .as_array()
        .ok_or("expected JSON array")?
        .len(),
        1
    );
    assert_eq!(
        *(*(*(*(config).get("hooks").ok_or("missing fixture index hooks")?)
            .get("stop")
            .ok_or("missing fixture index stop")?)
        .get(0)
        .ok_or("missing fixture index 0")?)
        .get("command")
        .ok_or("missing fixture index command")?,
        similar
    );
    assert_eq!(
        *(*(*(*(config).get("hooks").ok_or("missing fixture index hooks")?)
            .get("stop")
            .ok_or("missing fixture index stop")?)
        .get(1)
        .ok_or("missing fixture index 1")?)
        .get("command")
        .ok_or("missing fixture index command")?,
        harness.command("cursor")?
    );
    Ok(())
}
#[test]
fn preserves_prompt_hooks_on_touched_events() -> Result<(), Box<dyn std::error::Error + Send + Sync>>
{
    let harness = Harness::new()?;
    harness . write_config ("claude-code" , & serde_json :: json ! ({ "hooks" : { "Stop" : [{ "hooks" : [{ "type" : "prompt" , "prompt" : "is the task complete?" }] }] } }) ,) ? ;
    assert_success(&harness.install("claude-code")?);
    let claude = read_json(harness.config("claude-code")?)?;
    assert_eq!(
        *(*(*(*(*(*(claude).get("hooks").ok_or("missing fixture index hooks")?)
            .get("Stop")
            .ok_or("missing fixture index Stop")?)
        .get(0)
        .ok_or("missing fixture index 0")?)
        .get("hooks")
        .ok_or("missing fixture index hooks")?)
        .get(0)
        .ok_or("missing fixture index 0")?)
        .get("type")
        .ok_or("missing fixture index type")?,
        "prompt"
    );
    let harness = Harness::new()?;
    harness . write_config ("cursor" , & serde_json :: json ! ({ "version" : 1 , "hooks" : { "stop" : [{ "type" : "prompt" , "prompt" : "is the task complete?" }] } }) ,) ? ;
    assert_success(&harness.install("cursor")?);
    let cursor = read_json(harness.config("cursor")?)?;
    assert_eq!(
        *(*(*(*(cursor).get("hooks").ok_or("missing fixture index hooks")?)
            .get("stop")
            .ok_or("missing fixture index stop")?)
        .get(0)
        .ok_or("missing fixture index 0")?)
        .get("type")
        .ok_or("missing fixture index type")?,
        "prompt"
    );
    assert_eq!(
        *(*(*(*(cursor).get("hooks").ok_or("missing fixture index hooks")?)
            .get("stop")
            .ok_or("missing fixture index stop")?)
        .get(1)
        .ok_or("missing fixture index 1")?)
        .get("command")
        .ok_or("missing fixture index command")?,
        harness.command("cursor")?
    );
    Ok(())
}
