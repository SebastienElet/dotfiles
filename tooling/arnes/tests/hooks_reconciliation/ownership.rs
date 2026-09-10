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
#[test]
fn memory_reconciliation_preserves_third_party_order_and_replaces_owned_handlers()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let _: () = for agent in ["codex", "claude-code"] {
        let harness = Harness::new()?;
        harness.write_memory_manifest(agent)?;
        harness.executable_named("agent-memory")?;
        let command = harness.memory_command(agent)?;
        harness . write_config (agent , & serde_json :: json ! ({ "hooks" : { "UserPromptSubmit" : [{ "matcher" : "first" , "hooks" : [{ "type" : "command" , "command" : "third-before" }] } , { "hooks" : [{ "type" : "command" , "command" : command , "timeout" : 7 } , { "type" : "command" , "command" : "third-after" }] } , { "hooks" : [{ "type" : "command" , "command" : command }] }] } }) ,) ? ;
        assert_success(&harness.install(agent)?);
        let config = read_json(harness.config(agent)?)?;
        let commands: Vec<&str> =
            (*(*(config).get("hooks").ok_or("missing fixture index hooks")?)
                .get("UserPromptSubmit")
                .ok_or("missing fixture index UserPromptSubmit")?)
            .as_array()
            .ok_or("expected JSON array")?
            .iter()
            .map(
                |group| -> Result<_, Box<dyn std::error::Error + Send + Sync>> {
                    Ok(
                        (*(group).get("hooks").ok_or("missing fixture index hooks")?)
                            .as_array()
                            .ok_or("expected JSON array")?,
                    )
                },
            )
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .flatten()
            .map(
                |handler| -> Result<_, Box<dyn std::error::Error + Send + Sync>> {
                    Ok({
                        (*(handler)
                            .get("command")
                            .ok_or("missing fixture index command")?)
                        .as_str()
                    })
                },
            )
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .flatten()
            .collect();
        assert_eq!(commands, vec!["third-before", "third-after", &command]);
        let owned = (*(*(config).get("hooks").ok_or("missing fixture index hooks")?)
            .get("UserPromptSubmit")
            .ok_or("missing fixture index UserPromptSubmit")?)
        .as_array()
        .ok_or("expected JSON array")?
        .iter()
        .map(
            |group| -> Result<_, Box<dyn std::error::Error + Send + Sync>> {
                Ok(
                    (*(group).get("hooks").ok_or("missing fixture index hooks")?)
                        .as_array()
                        .ok_or("expected JSON array")?,
                )
            },
        )
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .flatten()
        .map(
            |handler| -> Result<_, Box<dyn std::error::Error + Send + Sync>> {
                let include = {
                    *(handler)
                        .get("command")
                        .ok_or("missing fixture index command")?
                        == command
                };
                Ok((include, handler))
            },
        )
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .find(|(include, _)| *include)
        .map(|(_, entry)| entry)
        .ok_or("expected JSON array")?;
        assert_eq!(
            *(owned)
                .get("timeout")
                .ok_or("missing fixture index timeout")?,
            30
        );
    };
    Ok(())
}
#[test]
fn owned_memory_hooks_are_removed_when_absent_from_the_manifest()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let _: () = for agent in ["codex", "claude-code"] {
        let harness = Harness::new()?;
        let command = harness.memory_command(agent)?;
        harness . write_config (agent , & serde_json :: json ! ({ "hooks" : { "UserPromptSubmit" : [{ "hooks" : [{ "type" : "command" , "command" : command } , { "type" : "command" , "command" : "third-party" }] }] } }) ,) ? ;
        assert_success(&harness.install(agent)?);
        let serialized = serde_json::to_string(&read_json(harness.config(agent)?)?)?;
        assert!(!serialized.contains("agent-memory"));
        assert!(serialized.contains("third-party"));
    };
    Ok(())
}
