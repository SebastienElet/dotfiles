use super::HookAgent;

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
}

pub fn policy(agent: HookAgent) -> Policy {
    match agent {
        HookAgent::Codex => Policy {
            directory: ".codex",
            filename: "hooks.json",
            events: CODEX,
            nested: true,
            excluded: &[],
        },
        HookAgent::ClaudeCode => Policy {
            directory: ".claude",
            filename: "settings.json",
            events: CLAUDE,
            nested: true,
            excluded: &[],
        },
        HookAgent::Cursor => Policy {
            directory: ".cursor",
            filename: "hooks.json",
            events: CURSOR,
            nested: false,
            excluded: &["afterAgentThought"],
        },
    }
}
