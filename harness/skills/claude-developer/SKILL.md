---
name: claude-developer
description: >
  Launch Claude Code with a prepared implementation or correction prompt. Use when the user asks
  Codex or Cursor to pass work to Claude or returns its result for another iteration. Make sure to
  use this skill for Claude delegation even when no prompt is requested; preserve explicit
  prompt-only requests.
metadata:
  category: dev
---

# Claude Developer

## Overview

Prepare one self-contained prompt and launch Claude Code in an interactive terminal in the verified
worktree. Use a manual handoff when the user asks only for a prompt or the host cannot launch an
interactive session. Starting a process and displaying it in the app are separate outcomes.

## Usage

`$claude-developer <task or returned result>` — prepare an implementation or correction handoff.
Delegation requests authorize one launch; requests for a prompt alone do not authorize execution.
Direct implementation requests without Claude stay with the current agent.

## Steps

1. Inspect repository instructions and the minimum read-only context needed for the handoff.
   Record unresolved facts instead of guessing. For an existing launch, inspect its session and
   continue at step 6 instead of preparing another prompt or starting another process.
2. Identify the intended existing worktree from the task context; verify its absolute path and
   branch with `git worktree list --porcelain`. Use the current worktree when the task targets it.
   If the intended worktree is missing or ambiguous, ask for its path before issuing a launch
   command; do not invent a path or create a checkout.
3. Write one prompt with the outcome, verified worktree path and branch (or detached HEAD), relevant
   facts, constraints, acceptance criteria, allowed scope, expected verification, and delivery
   report. Tell Claude to inspect before editing, preserve unrelated changes, and surface conflicts.
   Include the user's authorization limits; reserve consequential actions for the user unless
   explicitly authorized.
4. For a prompt-only request, or when interactive execution is unavailable, return two fenced
   blocks: `sh` with `cd <shell-quoted absolute worktree path> && claude`, then `text` with the prompt
   to paste. Explain any unavailable capability, then stop without launching.
5. Otherwise, use the host's interactive execution tool. In Codex, call `exec_command` with
   `workdir` set to the verified absolute path, `tty: true`, and `cmd` set to
   `claude <shell-quoted complete prompt>`. Preserve newlines inside the single prompt argument;
   use POSIX quoting in a POSIX-compatible shell, or the host shell's native quoting otherwise.
   Do not interpolate raw prompt text,
   execute it as shell syntax, or substitute `claude -p`, which exits after its response.
6. Keep the returned session identifier and inspect startup output with `write_stdin`, using empty
   input for observation. A session identifier alone is not evidence that Claude started. Report
   missing executables, authentication, trust prompts, hook errors, or early exits as observed.
   If the sandbox prevents access to Claude's normal configuration or credentials, use the host's
   approval mechanism for the exact launch; stop the failed process before an approved retry.
7. In the Codex desktop app, request the terminal panel with `mcp__codex_app__open_in_codex`, target
   `{"type":"terminal","sessionId":"<returned session identifier>"}`. Check
   `mcp__codex_app__read_thread_terminal` for output matching the launched process. A `queued`
   response, an empty panel, or unrelated output does not prove attachment: report the observed
   execution state and that panel attachment is unconfirmed. Never launch another
   Claude process to compensate for a display failure or bypass a Computer Use refusal.
8. Report the worktree, command, session identifier, observed startup state, and panel state, then
   leave the live session available. Forward only user-requested follow-up input to that session;
   do not submit the initial prompt again. If execution was not started, provide the manual blocks
   from step 4; if its outcome is uncertain, inspect the original session before proposing a retry.
9. On a returned result, review only supplied evidence and explicitly authorized local state.
   Produce a corrective handoff only when the user requests another iteration; an unexecuted prompt
   revision replaces the previous handoff.

## Gotchas

- **Starting in the wrong checkout** — a bare `claude` uses the terminal's current directory;
  supply the verified absolute path and `&&` so a failed `cd` prevents launch.
- **Confusing terminal identifiers** — an execution session may not attach to the app panel;
  verify matching output and report uncertainty instead of claiming the panel shows Claude.
- **Duplicating a launch** — an opening failure does not stop the existing process; retain its
  session identifier and inspect it before retrying or forwarding input.
- **Expanding the prompt in the shell** — quotes, backticks, and substitutions can execute prompt
  content; pass one safely quoted argument and preserve its literal text.
- **Trusting a returned summary** — it may omit defects or failed checks; inspect the available diff
  and evidence before making a verification claim.

## Constraints

- Activate only for requests explicitly involving Claude Code, a handoff, or its returned result.
- Never launch for a prompt-only request. During preparation, never create a branch or checkout,
  edit repository files, or validate the implementation; delegate only the authorized task.
- Never authorize commits, pushes, merges, deletion, publication, or permission bypasses unless the
  user's current request explicitly permits them.
- Produce at most one prompt per response and wait for the user before every subsequent iteration.
- Preserve Claude's configured model, permission mode, and hooks unless the user requests a change.
  Leave authentication and permission decisions to the user; do not answer them automatically.
- This is advisory policy, not a technical execution barrier. Never claim Claude's result is
  verified without reviewing the named evidence in its stated environment.

## References

- [Claude Code CLI reference](https://code.claude.com/docs/en/cli-reference) — interactive initial
  prompts and print mode
- [Integrated terminal](https://learn.chatgpt.com/docs/integrated-terminal) — app terminal and
  reusable actions; execution-session attachment must be verified with the available host tools
