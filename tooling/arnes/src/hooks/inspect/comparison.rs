use super::expectation::Expectation;
use super::{drift, healthy, presence};
use crate::diagnostic::Diagnostic;
use crate::manifest::HookKind;
use serde_json::Value;

pub fn superseded(config: &Value, expectation: &Expectation) -> Vec<String> {
    expectation
        .superseded
        .iter()
        .filter(|command| !presence::events(config, expectation.nested, command).is_empty())
        .cloned()
        .collect()
}

pub fn compare(
    expectation: &Expectation,
    installed: &[String],
    superseded: &[String],
    subject: &str,
    kind: HookKind,
) -> Diagnostic {
    let problems = problems(expectation, installed, superseded);
    if problems.is_empty() {
        return healthy(format!(
            "{subject} {kind} hook is installed on {}",
            coverage(expectation.events)
        ));
    }
    drift(format!(
        "{subject} {kind} hook is {}",
        problems.join(" and ")
    ))
}

fn problems(expectation: &Expectation, installed: &[String], superseded: &[String]) -> Vec<String> {
    let missing = expectation
        .events
        .iter()
        .filter(|event| !installed.iter().any(|name| name == *event))
        .copied()
        .collect::<Vec<_>>();
    let unexpected = installed
        .iter()
        .filter(|name| !expectation.events.contains(&name.as_str()))
        .cloned()
        .collect::<Vec<_>>();

    let mut problems = Vec::new();
    if !missing.is_empty() {
        problems.push(format!("missing from {}", missing.join(", ")));
    }
    if !unexpected.is_empty() {
        problems.push(format!(
            "installed on unexpected events {}",
            unexpected.join(", ")
        ));
    }
    if !superseded.is_empty() {
        problems.push(format!(
            "installed with superseded commands {}",
            superseded.join(", ")
        ));
    }
    problems
}

fn coverage(events: &[&str]) -> String {
    match events {
        [event] => (*event).to_owned(),
        events => format!("{} events", events.len()),
    }
}
