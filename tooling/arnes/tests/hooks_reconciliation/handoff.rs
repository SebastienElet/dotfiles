use super::*;
#[test]
fn installs_and_migrates_the_owned_claude_stop_hook_in_the_same_write()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let harness = Harness::new()?;
    let repository = harness.home.join("repository");
    let current = repository.join("tooling/agent-handoff");
    let legacy = repository.join("scripts/agent_handoff");
    fs::create_dir_all(current.parent().ok_or("fixture path has no parent")?)?;
    fs::create_dir_all(legacy.parent().ok_or("fixture path has no parent")?)?;
    fs::write(&current, b"binary")?;
    fs::set_permissions(&current, fs::Permissions::from_mode(0o700))?;
    harness . write_config ("claude-code" , & serde_json :: json ! ({ "keep" : true , "hooks" : { "Stop" : [{ "hooks" : [{ "type" : "command" , "command" : legacy , "timeout" : 7 , "async" : true , "asyncRewake" : true , "once" : true , "if" : "never" } , { "type" : "command" , "command" : "third-party" }] }] } }) ,) ? ;
    assert_success(&harness.install_claude_with_handoff(&current, &legacy)?);
    let config = read_json(harness.config("claude-code")?)?;
    let serialized = serde_json::to_string(&config)?;
    assert_eq!(
        *(config).get("keep").ok_or("missing fixture index keep")?,
        true
    );
    assert!(!serialized.contains(legacy.to_str().ok_or("required test value is missing")?));
    assert!(!serialized.contains(current.to_str().ok_or("required test value is missing")?));
    let deployed = harness.home.join(".local/bin/agent-handoff");
    assert_eq!(
        serialized
            .matches(deployed.to_str().ok_or("required test value is missing")?)
            .count(),
        1
    );
    assert!(serialized.contains("third-party"));
    let deployed_command = deployed
        .to_str()
        .ok_or("fixture command path is not UTF-8")?;
    let owned = (*(*(config).get("hooks").ok_or("missing fixture index hooks")?)
        .get("Stop")
        .ok_or("missing fixture index Stop")?)
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
                (*(handler)
                    .get("command")
                    .ok_or("missing fixture index command")?)
                .as_str()
                    == Some(deployed_command)
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
        *(owned).get("args").ok_or("missing fixture index args")?,
        serde_json::json!([])
    );
    assert_eq!(
        *(owned)
            .get("timeout")
            .ok_or("missing fixture index timeout")?,
        7
    );
    let _: () = for field in ["async", "asyncRewake", "once", "if"] {
        assert!(owned.get(field).is_none(), "{field} was preserved");
    };
    Ok(())
}
