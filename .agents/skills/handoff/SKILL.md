---
name: handoff
description: >
  Hand the current work over to a fresh agent session instead of letting the context be compacted.
  Use when the context window is nearly full, when a Stop hook reports the handoff threshold was
  reached, or when the user asks for a resume prompt. Make sure to use this skill whenever a session
  must end and continue elsewhere — approaching the token limit, avoiding auto-compaction, or
  starting a new session on the same task — even if the request only asks for "the prompt to keep
  going" or to "start over from scratch" without naming the context limit.
compatibility: Claude Code with scripts/claude_handoff_check installed as a Stop hook
metadata:
  category: ops
---

# Handoff

## Overview

Produce a short, copy-pasteable prompt that lets a fresh session resume the current work, and then
stop. Compaction summarises a transcript and keeps its noise; a handoff restates only the live
state, so the next session starts small and accurate. `scripts/claude_handoff_check` triggers this
skill automatically once context usage passes the threshold, but it is also useful on demand.

## Usage

```text
/handoff
```

No options. The output is a single fenced block for the user to paste into a new session.

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
- **Expecting the hook to fire with no configured window** — it stays silent unless
  `CLAUDE_CODE_AUTO_COMPACT_WINDOW` or `HANDOFF_TOKEN_THRESHOLD` is set, because guessing a limit
  would hand off far too early. Invoke `/handoff` manually in that case.

## Constraints

- Never keep working after emitting the handoff block — end the turn there.
- Never invent progress: only claim what was actually run and verified in this session.
- Keep the block self-contained — the next session sees no part of this conversation.
- Do not write the handoff to a file unless the user asks; the deliverable is text to paste.
- Do not attempt to disable or block compaction from the skill — that is the hook's job, via the
  threshold in `scripts/claude_handoff_check`.
