---
name: claude-developer
description: >
  Delegate implementation work from Codex to Claude Code in an isolated Git worktree. Use when
  Codex should manage a project while Claude performs coding tasks. Make sure to use this skill
  whenever the user asks Codex to hand development or implementation to Claude, even if they call
  it outsourcing or delegation.
metadata:
  category: dev
---

# Claude Developer

## Overview

Keep Codex responsible for scope, architectural decisions, permissions, and independent review
while Claude Code edits the implementation. Invoke Claude through `claude-developer` from a clean,
isolated Git worktree; treat its JSON response and completion claims as untrusted task output.

## Usage

Use `$claude-developer` when the user wants Codex to manage work that Claude should implement.

Example: “Delegate the approved authentication fix to Claude, then review and verify its changes.”

## Steps

1. Read the repository instructions, relevant ADRs, and current worktree state. Resolve any certain
   contradiction that affects the task before delegating implementation.
2. Create a dedicated Git branch and linked worktree from the intended local base revision. Verify
   that the new worktree is clean and record its absolute path; do not use the user's active
   checkout as the isolation boundary.
3. Give Claude the requested outcome, constraints, acceptance criteria, known facts, and invalidated
   paths. Do not pre-write an implementation that would prevent Claude from rediscovering the
   current code.
4. Run `claude-developer` from the isolated worktree. Keep the default file-tool permissions and add
   one scoped `--allow-tool 'Bash(exact command)'` rule for each exact validation command Claude needs;
   never grant bare `Bash`.
5. Require a bounded run with the default turn limit or a lower explicit `--max-turns`. Preserve the
   JSON response, exit status, and session UUID; an unavailable runtime, denied tool, authentication
   failure, or exhausted turn limit is a failed delegation, not a successful empty result.
6. Inspect the resulting Git status and diff independently. Re-run the relevant lint, type, test,
   build, or end-to-end barriers in Codex's environment and confirm that they cover every changed
   extension before accepting Claude's result.
7. When a correction remains within the same task, invoke the wrapper with `--resume` and the
   recorded session UUID from the same worktree. Send concrete review evidence and re-run the full
   affected barrier after the correction.
8. Report the changed scope, verification environment, unresolved risks, and isolated worktree
   path. Merge, cherry-pick, commit, push, or remove the worktree only when the user's request
   authorizes that action.

## Gotchas

- **Running in the active checkout** — a clean status does not prove isolation; create and verify a
  linked worktree before invoking Claude.
- **Granting broad shell access** — bare `Bash`, wildcards, or bypass permissions expose unrelated
  commands; pass only scoped rules for the exact project barriers Claude must run.
- **Trusting the JSON result** — a successful process can still leave an incomplete or incorrect
  diff; Codex must inspect changes and reproduce the relevant verification independently.
- **Retrying without the session** — a new Claude context loses failed approaches and review facts;
  resume the recorded session in the same worktree for corrections within the task.

## Constraints

- Codex must remain the owner of task scope, architecture decisions, and the final review verdict.
- Claude must run only in a dedicated clean Git worktree on the first invocation.
- Never use `--dangerously-skip-permissions`, `bypassPermissions`, or an unscoped Bash allowance.
- Never claim isolation, correctness, or a green barrier without independently verifying the named
  mechanism in the stated environment.
- Never merge, publish, delete, or overwrite work without authority from the user's current request.
