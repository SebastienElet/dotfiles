# Integration and provenance

## Source and adaptation

Adapted by Codex from [ayghri/i-have-adhd](https://github.com/ayghri/i-have-adhd),
commit `6f1f982d0a47c65899af3c5a7450b7098bc65325`, resolved on 2026-09-11.
The remote default branch resolved to that same commit; there were no later changes to adopt.
The upstream [MIT notice](../LICENSE), copyright 2026 Ayoub Ghriss, is retained in full.

The ten presentation rules, six exceptions, conversational persistence and pre-send editorial
check are retained. After a live test exposed reactivation on resume, the user explicitly chose
to remove conversational deactivation, preferring to edit the skill when necessary. This intentionally differs from upstream; there is no off command or session
suppression state in the final adaptation. Medical assertions were replaced with neutral presentation goals; the title,
commands, activation message and marker were renamed. The category and section structure follow
local skill conventions. Nothing in the adaptation diagnoses the reader.

The canonical body is `harness/skills/output-discipline/SKILL.md`. Claude's manual invocation flag
and Codex's `agents/openai.yaml` keep implicit model selection disabled. Explicit marker opt-in
allows lifecycle loading independently of those discovery controls.

Upstream's shared Node launcher is replaced by `arnes output-discipline`: the existing Rust binary
already owns hook deployment and repository resolution. This avoids a second installer, a runtime
dependency and copied rules. Arnes reads its deployed manifest link to locate the canonical source
from other projects; a missing or non-symlink deployed manifest never falls back to the current
project. A missing source or read error leaves startup non-blocking, with a diagnostic;
upstream suppresses these errors. No network request or persistent write belongs to this loader.
Only Claude Code and Codex are declared; no other upstream runtime integration is imported.

## Deployment and commands

These commands describe deployment into the user's active harness; they were **not** run against
the real home during this change. Use the repository's existing Moon tasks after reviewing the diff:

```sh
moon run harness:claude-skills harness:codex-skills harness:claude-hooks harness:codex-hooks
```

The hook tasks build/install Arnes and its existing dependencies. Skill targets derive their
individual symlinks from `home/.arnes.yaml`; there is no new Make target or installer.
Claude receives `~/.claude/skills/output-discipline`, Codex `~/.agents/skills/output-discipline`.

Manual activation in a session:

- Claude Code: `/output-discipline`
- Codex: `$output-discipline`

Enable automatic loading for subsequent lifecycle events:

```sh
mkdir -p "${CLAUDE_CONFIG_DIR:-$HOME/.claude}"
touch "${CLAUDE_CONFIG_DIR:-$HOME/.claude}/.output-discipline-always"
```

There is no conversational deactivation command. Edit the canonical skill when its behavior
needs adjustment; the loader reads the edited body on the next lifecycle event.
The marker remains shared with Claude's configuration directory even under Codex, preserving upstream's convention. If using
`CLAUDE_CONFIG_DIR`, give both runtimes the same value. Arnes's existing Claude hook destination
is `~/.claude/settings.json`; custom Claude configuration directories require their own existing
hook deployment, which this iteration does not add.

## Host contracts and limits

Official documentation checked on 2026-09-11:

- [Claude hooks](https://code.claude.com/docs/en/hooks): `SessionStart` matches `startup`, `resume`,
  `clear`, `compact`; stdout is context. Arnes writes `~/.claude/settings.json`.
- [Codex hooks](https://learn.chatgpt.com/docs/hooks): the same lifecycle source names and stdout
  context are supported; Arnes writes `~/.codex/hooks.json`. Each new or changed non-managed hook
  must be reviewed and trusted in `/hooks`; writing the file does not grant trust.
- [Claude skills](https://code.claude.com/docs/en/skills) documents explicit `/name` invocation and
  `disable-model-invocation`. [Codex skills](https://learn.chatgpt.com/docs/build-skills) documents
  local discovery. The paired OpenAI metadata is retained from upstream and local convention.

The hook declares only `SessionStart`, matcher `startup|resume|clear|compact`, timeout 30 seconds.
It reads the same full body used by manual activation and strips initial YAML frontmatter.
No per-message hook, word count, response rewriting or output compliance gate is introduced.
Injection consumes context; installation alone says nothing about improved answers.

The marker causes fresh loading on resume or compaction. This is intentional after the user
removed conversational deactivation from scope; no persisted session override is needed.
Higher-priority host and user instructions continue to apply.

`harness/SOUL.md` previously capped reports at five sentences and allowed sacrificing grammar.
Their removal was explicitly approved during this task. The remaining instruction to be concise
still applies; the skill does not claim higher authority than the harness.

## Verification record

See [verification.md](verification.md) for executed checks, live observations and unverified paths.
