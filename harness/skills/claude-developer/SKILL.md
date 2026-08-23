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

Keep the current agent responsible for analysis and prompt quality while the user controls every
handoff to Claude Code. Inspect the available context, produce one self-contained prompt for the
user to paste manually, then stop. Continue only after the user returns with Claude's result or asks
to revise the prompt.

This workflow is advisory at the agent-instruction layer. Removing the repository wrapper closes
the managed automation path but cannot technically prevent every possible shell invocation of
Claude; never present the behavior as an enforced execution boundary.

## Usage

Use `$claude-developer` when the user asks Codex or Cursor to prepare work for manual execution in
Claude Code.

Example: “Write a prompt I can paste into Claude Code to implement the approved authentication
fix.”

## Steps

1. Confirm that the request explicitly involves Claude Code, a manual Claude handoff, or a returned
   Claude result that may need another prompt. A direct request to implement a change without Claude
   stays with the current agent and must not trigger this skill.
2. Inspect repository instructions and the minimum read-only context needed to make the prompt
   accurate. State unresolved facts in the prompt instead of guessing.
3. Write one self-contained prompt that gives Claude the outcome, relevant facts, constraints,
   acceptance criteria, allowed scope, expected verification, and requested delivery report.
4. Tell Claude to inspect the implementation before editing, preserve unrelated changes, surface
   conflicts, and leave consequential actions such as commits or pushes to the user unless the
   prompt explicitly authorizes them.
5. Return exactly one fenced text block containing the prompt, then stop. Do not invoke Claude,
   create an isolated checkout, edit files, run validations, or begin a second iteration.
6. If the user asks to revise an unexecuted prompt, produce one replacement prompt and stop again.
7. After the user brings back Claude's result, review only the supplied evidence and any explicitly
   authorized local state. Produce one corrective prompt only when the user requests another
   iteration, then stop again.

## Gotchas

- **Treating any implementation request as delegation** — Claude becomes an implicit default;
  activate only when the request explicitly involves Claude Code or a manual Claude handoff.
- **Producing a vague prompt** — Claude must rediscover settled decisions and may widen the scope;
  include the known constraints, acceptance criteria, and verification expectations.
- **Continuing after the prompt** — the handoff becomes automatic and the user loses the validation
  point; return one prompt and wait for the user's next message.
- **Trusting a reported result** — Claude's summary may omit defects or unverified claims; review the
  returned diff and evidence before preparing a corrective iteration.
- **Calling prose an execution guard** — agent instructions can be bypassed by routing or obedience
  failures; describe this workflow as advisory and do not claim that it technically blocks Claude.

## Constraints

- Activate only when the request explicitly involves Claude Code or a manual Claude handoff.
- Never invoke Claude Code, an automation wrapper, or another implementation agent.
- Never create a branch or isolated checkout, edit files, invoke an implementation agent, or
  validate the implementation while preparing the prompt.
- Produce one prompt per response and wait for the user before every subsequent iteration.
- Treat the no-invocation rule as advisory policy, never as a technical execution barrier.
- Never claim that Claude's result is correct or verified without reviewing the named evidence in
  its stated environment.
- Never authorize commits, pushes, merges, deletion, publication, or permission bypasses unless the
  user's current request explicitly permits them.
