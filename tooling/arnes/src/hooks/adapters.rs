use crate::manifest::Agent;

const CODEX: &[&str] = &[
    "SessionStart",
    "UserPromptSubmit",
    "PreToolUse",
    "PermissionRequest",
    "PostToolUse",
    "PreCompact",
    "PostCompact",
    "SubagentStart",
    "SubagentStop",
    "Stop",
    "SessionEnd",
];
const CLAUDE: &[&str] = &[
    "SessionStart",
    "UserPromptSubmit",
    "PreToolUse",
    "PermissionRequest",
    "PermissionDenied",
    "PostToolUse",
    "PostToolUseFailure",
    "SubagentStart",
    "SubagentStop",
    "Stop",
    "StopFailure",
    "PreCompact",
    "PostCompact",
    "SessionEnd",
];
const CURSOR: &[&str] = &[
    "sessionStart",
    "beforeSubmitPrompt",
    "preToolUse",
    "postToolUse",
    "postToolUseFailure",
    "subagentStart",
    "subagentStop",
    "afterAgentResponse",
    "stop",
    "preCompact",
    "postCompact",
    "sessionEnd",
];

pub struct Policy {
    pub directory: &'static str,
    pub filename: &'static str,
    pub events: &'static [&'static str],
    pub nested: bool,
    pub excluded: &'static [&'static str],
    pub handoff_args: bool,
    pub handoff_execution_fields: &'static [&'static str],
    pub memory_event: Option<&'static str>,
}

pub fn policy(agent: Agent) -> Policy {
    match agent {
        Agent::Codex => Policy {
            directory: ".codex",
            filename: "hooks.json",
            events: CODEX,
            nested: true,
            excluded: &[],
            handoff_args: false,
            handoff_execution_fields: &["async"],
            memory_event: Some("UserPromptSubmit"),
        },
        Agent::Claude => Policy {
            directory: ".claude",
            filename: "settings.json",
            events: CLAUDE,
            nested: true,
            excluded: &[],
            handoff_args: true,
            handoff_execution_fields: &["async", "asyncRewake", "once", "if"],
            memory_event: Some("UserPromptSubmit"),
        },
        Agent::Cursor => Policy {
            directory: ".cursor",
            filename: "hooks.json",
            events: CURSOR,
            nested: false,
            excluded: &["afterAgentThought"],
            handoff_args: false,
            handoff_execution_fields: &[],
            memory_event: None,
        },
    }
}
