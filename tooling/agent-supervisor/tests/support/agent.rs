use serde_json::json;
use std::{env, fs, path::PathBuf};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let arguments: Vec<String> = env::args().collect();
    if arguments
        .get(1)
        .is_some_and(|value| value == "--descendant")
    {
        return hold();
    }
    if arguments.get(1).is_some_and(|value| value == "usage") {
        return capacity();
    }
    let argument = |name: &str| -> Result<PathBuf, Box<dyn std::error::Error>> {
        arguments
            .windows(2)
            .find(|pair| pair[0] == name)
            .map(|pair| PathBuf::from(&pair[1]))
            .ok_or_else(|| format!("missing {name}").into())
    };
    let repository = argument("--cd")?;
    let count_path = repository.join("count");
    let count = fs::read_to_string(&count_path)
        .unwrap_or_default()
        .parse::<u32>()
        .unwrap_or_default()
        + 1;
    fs::write(count_path, count.to_string())?;
    let scenario = fs::read_to_string(repository.join("scenario")).unwrap_or_default();
    if scenario == "failed" {
        return Err("synthetic private error".into());
    }
    let outcome = match scenario.as_str() {
        "decision" | "dependency" | "complete" => scenario.as_str(),
        "remove-checkout" => "decision",
        _ if count == 1 => "handoff",
        _ => "complete",
    };
    let checkpoint = json!({
        "outcome":outcome,
        "summary":"synthetic evidence", "next_action":"read the preserved draft",
        "task_constraints":["no publication for this task"],
        "permanent_rule_sources":["AGENTS.md"],
        "decision":if outcome == "decision" {json!({"question":"Which synthetic option?", "options":["A","B"],"recommendation":"A"})} else {json!(null)},
        "dependency":if outcome == "dependency" {json!({"event":"third-party PR merged", "authorized_action":"rerun integration"})} else {json!(null)}
    });
    println!(
        "{}",
        json!({"type":"thread.started", "thread_id":format!("thread-{}", if scenario == "old-thread" {1} else {count})})
    );
    if scenario != "missing-start" {
        println!("{}", json!({"type":"turn.started"}));
    }
    if scenario == "active" {
        fs::write(repository.join("agent-pid"), std::process::id().to_string())?;
        let child = std::process::Command::new(env::current_exe()?)
            .arg("--descendant")
            .current_dir(&repository)
            .spawn()?;
        fs::write(repository.join("child-pid"), child.id().to_string())?;
        return hold();
    }
    fs::write(
        argument("--output-last-message")?,
        if scenario == "invalid-result" {
            b"[]".to_vec()
        } else {
            serde_json::to_vec(&checkpoint)?
        },
    )?;
    if scenario == "truncated" {
        print!("{{\"type\":");
        return Ok(());
    }
    if scenario == "remove-checkout" {
        fs::remove_dir_all(&repository)?;
    }
    if scenario == "failed-after-start" {
        println!("{}", json!({"type":"turn.failed"}));
        return Ok(());
    }
    println!("{}", json!({"type":"turn.completed"}));
    Ok(())
}

fn hold() -> Result<(), Box<dyn std::error::Error>> {
    let ignored = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    signal_hook::flag::register(signal_hook::consts::SIGTERM, ignored)?;
    fs::write(env::current_dir()?.join("entered"), "ready")?;
    loop {
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}

fn capacity() -> Result<(), Box<dyn std::error::Error>> {
    let repository = env::current_dir()?;
    let count_path = repository.join("quota-count");
    let count = fs::read_to_string(&count_path)
        .unwrap_or_default()
        .parse::<u32>()
        .unwrap_or_default()
        + 1;
    fs::write(count_path, count.to_string())?;
    let mode = fs::read_to_string(repository.join("capacity")).unwrap_or_default();
    match mode.as_str() {
        "failed" => return Err("synthetic private capacity error".into()),
        "empty" => {
            println!("[]");
            return Ok(());
        }
        "invalid" => {
            println!("invalid synthetic response");
            return Ok(());
        }
        "no-windows" => {
            println!(
                "{}",
                json!([{"provider":"codex","source":"codex-cli","usage":{"primary":null,"secondary":null}}])
            );
            return Ok(());
        }
        _ => {}
    }
    let usage = if mode == "exhausted" || (mode == "exhausted-once" && count == 1) {
        100
    } else {
        0
    };
    println!(
        "{}",
        json!([{"provider":if mode == "wrong-provider" {"claude"} else {"codex"},"source":"codex-cli",
        "error":if mode == "provider-error" {json!({"message":"synthetic private provider error"})} else {json!(null)},
        "usage":{"primary":if mode == "secondary-only" {json!(null)} else {json!({"usedPercent":usage})},"secondary":{"usedPercent":0}}}])
    );
    Ok(())
}
