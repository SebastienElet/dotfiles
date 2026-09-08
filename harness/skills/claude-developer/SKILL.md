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

Prepare one self-contained prompt and launch Claude Code interactively in the verified worktree.
Use a manual handoff for prompt-only requests or when interactive execution is unavailable.

## Usage

`$claude-developer <task or returned result>` — delegate one implementation or correction.
Requests without Claude stay with the current agent; prompt-only requests authorize no execution.

## Steps

1. Inspect repository instructions and only the read-only context needed. For an existing launch,
   continue at step 5 without preparing or resending a prompt. On a returned result, review only
   supplied evidence and explicitly authorized local state; continue only if another iteration is
   requested. An unexecuted prompt revision replaces the previous handoff.
2. Verify the intended existing worktree's absolute path and branch (or detached HEAD) with
   `git worktree list --porcelain`; use the current worktree when targeted. If missing or ambiguous,
   ask for its path before issuing a command.
3. Prepare the prompt: outcome, verified worktree and branch, relevant facts and uncertainties,
   constraints, allowed scope, acceptance criteria, verification, and delivery report. Tell Claude
   to inspect before editing, preserve unrelated changes, surface conflicts, and respect the user's
   authorization limits.
4. Choose the handoff:
   - **Manual:** return a `sh` block containing `cd <shell-quoted absolute worktree path> && claude`,
     then a `text` block containing the prompt. Explain unavailable execution when relevant and stop.
   - **Automatic:** use the host's interactive execution tool. In Codex, call `exec_command` with
     the verified `workdir`, `tty: true`, and `cmd: claude <shell-quoted complete prompt>`. Use the
     host shell's quoting to preserve the complete prompt, including newlines, as one literal
     argument. Do not use print mode (`claude -p`), which exits after responding.
5. Retain the session identifier and observe startup; in Codex, use `write_stdin` with empty input.
   Report missing executables, authentication or trust prompts, hook errors, and early exits as
   observed. For sandbox access failures, request host approval for the exact launch and stop the
   failed process before retrying. If no process started, provide the manual blocks; if uncertain,
   inspect the original session before retrying.
6. In Codex desktop, call `mcp__codex_app__open_in_codex` with target
   `{"type":"terminal","sessionId":"<returned session identifier>"}`, then check
   `mcp__codex_app__read_thread_terminal` for matching output. Report the worktree, command, session
   identifier, observed execution state, and panel state; leave a live session available. Forward
   only user-requested follow-up input, without resending the initial prompt.

## Gotchas

- **Wrong checkout** — a bare `claude` inherits the terminal directory; set `workdir` for execution
  or use the verified `cd ... && claude` for a manual launch.
- **Shell expansion** — quotes, backticks, and substitutions can execute prompt content; use POSIX
  quoting in compatible shells and native quoting elsewhere, never raw interpolation.
- **Unconfirmed panel attachment** — `queued`, empty, or unrelated output does not prove that the
  execution session is displayed; report attachment as unconfirmed, never launch a duplicate or
  bypass a Computer Use refusal.

## Constraints

- During preparation, never create a branch or checkout, edit repository files, or validate the
  implementation; delegate only the authorized task.
- Produce at most one prompt per response and wait for the user before each subsequent iteration.
- Preserve Claude's configured model, permission mode, and hooks unless the user requests a change;
  leave authentication and permission decisions to the user.
- Never authorize commits, pushes, merges, deletion, publication, or permission bypasses without
  explicit authorization in the current request.
- This is advisory policy, not an execution barrier. Neither a session identifier nor Claude's
  summary proves success; verify claims against observed output and named evidence in its stated
  environment.

## References

- [Claude Code CLI reference](https://code.claude.com/docs/en/cli-reference) — interactive prompts
  and print mode
- [Integrated terminal](https://learn.chatgpt.com/docs/integrated-terminal) — app terminal and actions
