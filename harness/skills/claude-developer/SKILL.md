---
name: claude-developer
description: >
  Prepare manual implementation and correction prompts for Claude Code without invoking it. Use
  when the user asks Codex or Cursor to pass work to Claude or returns its result for another manual
  iteration. Make sure to intercept delegation requests even when the user does not ask for a
  prompt; never invoke Claude automatically.
metadata:
  category: dev
---

# Claude Developer

## Overview

Prepare a launch command and one self-contained prompt for a manual Claude Code handoff. The user
runs the command, pastes the prompt, and returns with the result when ready. Local implementation
and correction handoffs before a PR finish with `review-fix` before Claude reports completion.

## Usage

`$claude-developer <task or returned result>` — prepare an implementation or correction handoff.
Direct implementation requests without Claude stay with the current agent.

## Steps

1. Inspect repository instructions and the minimum read-only context needed for the handoff.
   Record unresolved facts instead of guessing.
2. Identify the intended existing worktree from the task context; verify its absolute path and
   branch with `git worktree list --porcelain`. Use the current worktree when the task targets it.
   If the intended worktree is missing or ambiguous, ask for its path before issuing a launch
   command; do not invent a path or create a checkout.
3. Write one prompt with the outcome, verified worktree path and branch (or detached HEAD), relevant
   facts, constraints, acceptance criteria, allowed scope, expected verification, and delivery
   report. Tell Claude to inspect before editing, preserve unrelated changes, and surface conflicts.
   Include the user's authorization limits; reserve consequential actions for the user unless
   explicitly authorized.
4. For an implementation or correction handoff on local work before a PR, include this final
   stage in the prompt: after implementation and the repository checks pass, activate `review-fix`
   on the complete task delta, authorize Codex to correct demonstrated defects within the original
   task scope, and wait for the fresh final review before delivery. Require the final report to
   include its candidate identity, finding dispositions, checks, environment and remaining limits.
   Preserve stricter user limits: a review-only request authorizes no repair, and a missing runtime,
   failed check or incomplete final review must be reported as incomplete rather than skipped.
   Do not add this stage to planning-only handoffs or repeat it inside a `review-fix` correction.
   For work on an already open PR, retain the authorized PR workflow and its `pr-fix` review;
   do not substitute local `review-fix`, create a second review loop, or infer repair authority.
5. Return two fenced blocks in order: a `sh` block with the copyable one-line command
   `cd <shell-quoted absolute worktree path> && claude`, then a `text` block with the prompt to paste
   into that session. Substitute the verified path, quoting shell metacharacters safely. Repeat both
   blocks for every replacement or corrective prompt, then stop.
6. On a returned result, review only supplied evidence and explicitly authorized local state.
   Produce a corrective handoff only when the user requests another iteration; an unexecuted prompt
   revision replaces the previous handoff.

## Gotchas

- **Starting in the wrong checkout** — a bare `claude` uses the terminal's current directory;
  supply the verified absolute path and `&&` so a failed `cd` prevents launch.
- **Losing the manual handoff** — executing the command or starting another iteration removes the
  user's control; supply the two blocks and wait.
- **Trusting a returned summary** — it may omit defects or failed checks; inspect the available diff
  and evidence before making a verification claim.
- **Ending the prompt at green checks** — Claude can deliver without independent review; include
  the final `review-fix` stage once for local implementation and preserve incomplete outcomes.

## Constraints

- Activate only for requests explicitly involving Claude Code, a manual handoff, or its returned
  result; intercept automatic Claude delegation as a manual handoff.
- During preparation, never invoke Claude, an automation wrapper, or another implementation agent;
  never create a branch or checkout, edit files, or validate the implementation.
- Never authorize commits, pushes, merges, deletion, publication, or permission bypasses unless the
  user's current request explicitly permits them.
- Produce at most one prompt per response and wait for the user before every subsequent iteration.
- Never treat the prompted final review as observed evidence or bypass the user's repair limits.
- This is advisory policy, not a technical execution barrier. Never claim Claude's result is
  verified without reviewing the named evidence in its stated environment.
