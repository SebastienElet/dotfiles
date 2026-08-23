# Handoff — Design

**Date**: 2026-08-03 · **Status**: implemented

## Problem

Near the context limit Claude Code auto-compacts the conversation: the transcript is summarised and
the session continues with its noise carried forward. Preferred behaviour is the opposite — end the
session cleanly and restate only the live state, so a fresh agent starts small and accurate.

## Constraints discovered

Verified against CLI 2.1.220 with `claude -p --settings <file>` probes, not assumed:

- A Stop hook's stdin carries `session_id`, `transcript_path`, `cwd`, `prompt_id`,
  `permission_mode`, `hook_event_name`, `stop_hook_active`, `last_assistant_message`,
  `background_tasks`, `session_crons`. It does **not** carry `context_window` — that exists only in
  the status line's input. Usage must be computed from the transcript.
- Hooks inherit the `env` block from `settings.json`, so `CLAUDE_CODE_AUTO_COMPACT_WINDOW` is
  readable inside a hook script.
- `{"decision": "block", "reason": "..."}` re-invokes the model with that reason as an instruction.
  This is the only hook mechanism that produces model output automatically. `PreCompact` cannot: no
  model turn is in flight when it runs, so it can write nothing.
- Auto-compaction fires at `window - min(maxOutputTokens, 20000) - 13000` — about 567k for a 600k
  window, not at some round percentage of it.
- Hooks are snapshotted at session start; a `settings.json` edit needs a restart.

## Design

A race, not a fight: fire the handoff before auto-compaction rather than trying to block it.
Auto-compaction stays enabled as a backstop, so a missed handoff degrades to today's behaviour
instead of a context-overflow error.

**Trigger** — `scripts/claude_handoff_check`, wired as a user-level `Stop` hook:

1. Exit if `stop_hook_active` (the hook's own re-invocation).
2. Exit if a sentinel exists at `${XDG_STATE_HOME}/dotfiles/handoff/<session_id>` — the context is
   still full after the handoff, so without this the hook would block every subsequent turn forever.
3. Compute usage from the last main-chain assistant message (`isSidechain` excluded: subagent turns
   have their own context) as `input + cache_read + cache_creation`.
4. Threshold: `HANDOFF_TOKEN_THRESHOLD`, else 85% of `CLAUDE_CODE_AUTO_COMPACT_WINDOW`. With
   neither set, stay silent — guessing a window would hand off a third of the way into a session.
5. Past the threshold: write the sentinel and block, instructing the agent to use the `handoff`
   skill and stop.

Every other failure — no `jq`, unreadable transcript, no usage line, non-numeric total — exits 0 and
falls through to auto-compaction. Failing open is right here, but the failure is invisible.

**Content** — `.agents/skills/handoff/SKILL.md` emits one fenced block: Goal, Done, Next step,
Files, under ~200 words, in the conversation's language, then ends the turn. No file on disk: the
deliverable is text to paste.

**Wiring** — the `claude-code` Makefile target symlinks `~/.claude/hooks/claude_handoff_check` and
`~/.claude/skills/handoff` into the repo. The hook entry itself lives in `~/.claude/settings.json`,
which is a real file and not version-controlled.

## Known ceilings

- `Stop` fires at turn boundaries, so a single turn that crosses the threshold mid-tool-loop slips
  past. At 510k against a 567k compaction point the margin covers it.
- Sentinels accumulate under `~/.local/state/dotfiles/handoff` and are never pruned.
- A resumed session that already handed off cannot hand off again.
- Two blocking Stop hooks now coexist in this repo (this one and the instruction-distillation hook);
  both reasons reach the model together.

## Verification

`scripts/claude_handoff_check_test` asserts the decision logic: below threshold, above threshold,
`stop_hook_active`, sidechain-only usage, unreadable transcript, double block in one session, and no
configured window. End-to-end behaviour was confirmed with a real `claude -p` run at
`HANDOFF_TOKEN_THRESHOLD=1`: the block was honoured, the model emitted a handoff block, the sentinel
was written, and the following Stop arrived with `stop_hook_active: true` and did not loop.
