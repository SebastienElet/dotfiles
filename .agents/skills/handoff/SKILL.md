---
name: handoff
description: >
  Hand the current work to a fresh session instead of letting the context compact. Use when the
  window is nearly full, a Stop hook reports the handoff threshold, or a resume prompt is asked
  for. Make sure to use it whenever a session must continue elsewhere, even if the context limit
  is never named.
compatibility: Claude Code or Codex with scripts/agent_handoff installed as a Stop hook
metadata:
  category: ops
---

# Handoff

## Overview

Produce a short, copy-pasteable prompt that lets a fresh session resume the current work, and then
stop. Compaction summarises a transcript and keeps its noise; a handoff restates only the live
state, so the next session starts small and accurate. `scripts/agent_handoff` triggers this
skill automatically once context usage passes the threshold, but it is also useful on demand.

## Usage

```text
/handoff
$handoff
```

Use `/handoff` in Claude Code and `$handoff` in Codex. There are no options. The output is a single
fenced block for the user to paste into a new session.

## Steps

1. Stop the current work — do not start a new edit, search, or tool loop.
2. Finish making the work durable: save unsaved files and, if a change is complete and the user
   asked for it, commit it. A resume prompt pointing at lost edits is worthless.
3. Write the resume prompt as one fenced block, addressed to the next agent, in the language of the
   conversation, covering exactly:
   - **Goal** — the task in one or two sentences, including the user's own constraints.
   - **Done** — what is already done and verified, with file paths.
   - **Next step** — the single next concrete action.
   - **Files** — the paths the next session needs to read first.
4. Keep it under ~200 words. Name files instead of quoting them; the next session can read them.
5. End your turn immediately after the block. Do not add follow-up work or offer to continue.

## Gotchas

- **Summarising the conversation instead of the state** — a handoff is not a transcript summary.
  Drop abandoned approaches, tool noise, and anything the next agent can read from a file; keep only
  what it needs to act.
- **Emitting the prompt and then continuing to work** — the point is to end the session before
  compaction. Any further tool call adds context and defeats it, so stop after the block.
- **Leaving work uncommitted** — the next session inherits the working tree, not the reasoning. State
  explicitly under **Done** whether changes are committed, staged, or only on disk.
- **Writing the block in English out of habit** — the section labels above are English because this
  file must be, for multi-agent portability; the block itself follows the conversation's language.
- **Being triggered mid-task by the hook** — the threshold fires at the end of a turn regardless of
  where the work stands. Say plainly under **Next step** that a step is half-done, rather than
  implying it is complete.
- **Expecting the hook to fire with no known window** — Claude Code needs
  `CLAUDE_CODE_AUTO_COMPACT_WINDOW` or `HANDOFF_TOKEN_THRESHOLD`; Codex supplies its window in the
  transcript. Invoke the skill manually when the hook cannot determine a limit.

## Constraints

- Never keep working after emitting the handoff block — end the turn there.
- Never invent progress: only claim what was actually run and verified in this session.
- Keep the block self-contained — the next session sees no part of this conversation.
- Do not write the handoff to a file unless the user asks; the deliverable is text to paste.
- Do not attempt to disable or block compaction from the skill — that is the hook's job, via the
  threshold in `scripts/agent_handoff`.
