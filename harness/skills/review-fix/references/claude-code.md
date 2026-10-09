# Claude Code transport

Use the [official OpenAI Codex companion](https://github.com/openai/codex-plugin-cc) to run Codex
from Claude Code. The local `review-fix` skill owns scope, correction authority, pass accounting and
independent review. The plugin provides the transport; its review commands alone authorize no fix.

## Installation ownership

From the canonical dotfiles checkout, the explicit optional task is:

```sh
moon run harness:claude-codex-companion
```

Moon installs the plugin through Claude Code's native CLI, with Claude, Codex and Node dependencies.
It is deliberately absent from the default `harness:claude` aggregate. Arnes permits the plugin
and its three bundled skills in `home/.arnes.yaml`; permission neither installs it nor makes it
mandatory. The three skills observed in upstream version 1.0.6 are `codex-cli-runtime`,
`codex-result-handling` and `gpt-5-4-prompting`. Added upstream skills require a separate policy
decision. Do not copy or edit the plugin's instructions locally.

Reload plugins in Claude Code, then check setup:

```text
/reload-plugins
/codex:setup --disable-review-gate
```

Use the existing Codex authentication and configured model. If authentication or the CLI is
unavailable, stop the repair and report setup as missing. Do not run npm installation or substitute
credentials. Keep the optional `Stop` review gate disabled: the explicit workflow owns the loop.

## Delegation

Use the host's `Agent` tool with `subagent_type: "codex:codex-rescue"`, forwarding one self-contained
request with `--fresh --wait`. Do not invoke `Skill(codex:rescue)` or
`Skill(codex:codex-rescue)`; upstream documents recursive dispatch and a nonexistent skill.
The rescue subagent is only a forwarder. Do not ask it to read files, review, monitor or interpret
results itself: its task invocation sends the substantive request to Codex.

For the initial and final reviewers, the forwarded request must explicitly say **read-only; do not
edit, stage, commit or publish**, name the exact worktree and candidate identity, and require the
`code-review` procedure with its independent axes. For correction, explicitly authorize edits only
to the confirmed slate and relevant tests. Include the user's actual requirements, checked source
references, verification commands and permission limits in every request. Never inherit a rescue
thread with `--resume`; a fresh invocation supplies a new Codex context.

The correction request must require a candidate identity check before writing and test evidence
afterwards. The final reviewer receives requirements, code and raw checks before the correction
summary or previous findings. A model change is optional; a new context is required.

Leave model and reasoning effort unset unless the user explicitly selected them. Use foreground
completion so a repair cannot end while a child still writes. Give each host invocation a
15-minute wall-clock limit. On timeout, retain the returned job identity, cancel that exact job
through the plugin's cancellation command and verify it stopped before another writer starts.
If cancellation cannot be confirmed, report incomplete and keep all writers suspended. A missing
result or an invocation failure is incomplete, never a successful empty review.

Retain the raw output and the job identity returned by each invocation. Recover results by that
exact identity using `/codex:status`, `/codex:result` or `/codex:cancel`; do not select the latest job
implicitly when another session may use the same repository. Forwarder output can be empty on
failure, so verify actual completion and check the final worktree instead of trusting its summary.

## Source and limits

The command and subagent contracts were inspected at upstream commit
[`db52e28`](https://github.com/openai/codex-plugin-cc/tree/db52e28f4d9ded852ab3942cea316258ae4ef346)
on 2026-10-09. The native install command resolves the marketplace's current version; this is not
a pinned copy of the plugin. Recheck changed upstream contracts before relying on new behavior.

The plugin's standalone review commands remain read-only and return their output unchanged.
`review-fix` is a separately authorized repair request, not an implicit extension of those commands.
These instructions express workflow policy; host timeout, isolation and model behavior must be
observed before claiming they were enforced. Arnes projection checks prove deployment and policy
inventory, not successful execution of the full Claude/Codex repair loop.
