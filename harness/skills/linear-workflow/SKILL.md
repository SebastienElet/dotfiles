---
name: linear-workflow
description: >
  Apply the shared Linear and Bitbucket work invariants. Use when another Linear execution skill
  composes the common workflow policy, or to verify merged work against an issue's completion
  evidence. Make sure to use this skill whenever starting, resuming, verifying, or finishing
  Linear-backed implementation, even if the request mentions only an issue ID.
compatibility: Requires access to the relevant Linear workspace and Bitbucket repository.
metadata:
  category: dev
---

# Linear Workflow Policy

## Overview

Provide the single reusable policy for execution workflows backed by Linear and Bitbucket. Each
composing workflow owns its own procedure and transport selection; the one procedure this skill
owns is verifying merged work against an issue's completion evidence, because no execution skill
may produce that evidence on its own.

## Usage

Load `linear-workflow` alongside a specific execution skill such as `linear-start`. Do not invoke it
alone to shape, implement, or review an issue. Verifying merged work against an issue's completion
evidence, one issue or a whole queue, is the single procedure it carries directly.

## Steps

1. Read [transport adapters](references/transports.md) before accessing Linear and
   [Bitbucket operations](references/bitbucket.md) before accessing pull requests.
2. Read ownership, hierarchy, relations, and work state from structured Linear fields.
3. Read branch and pull-request state from Git and Bitbucket.
4. Before checking or restructuring a checkbox or evidence section, and before completing an issue
   from merged work, read [completion evidence](references/completion-evidence.md). Reading it is
   mandatory for every workflow; running its verification pass is not.
5. On an explicit request to verify merged work, run that reference's verification pass over the
   issues waiting in the review state, and report the queue it defines. This is the one procedure to
   execute here rather than hand back.
6. Apply every constraint below, then return control to the composing workflow when there is one.

## Gotchas

- **Trusting description text for ownership or blocking** — stale prose overrides structured state;
  use Linear's assignee and relation fields.
- **Starting a parent with open sub-issues** — work bypasses the executable leaves; select an
  assigned, unblocked sub-issue instead.
- **Equating issue and pull-request granularity** — related changes gain artificial review
  boundaries; let the implementation scope determine whether sub-issues share a pull request.
- **Checking a box because the issue is being completed** — the description then claims proof no
  run produced, and the missing check disappears with the closed issue; leave it unchecked and
  record the residue.
- **Completing merged work over an unchecked verification box** — the check nobody ran is buried
  under `Done` and never resurfaces; send the issue to its team's review state and name the line.
- **Carrying one team's review state name to another** — the transition targets a state the team
  does not expose; resolve the review state inside the issue's own team.
- **Retyping a description to change one marker** — serialized issue mentions such as
  `<issue id="…" href="…">ENG-482</issue>` are mangled; apply an anchored partial edit instead.

## Constraints

- Linear is the source of truth for work; Bitbucket is the source of truth for Git and pull-request
  state.
- Work only on issues assigned to the current user.
- Treat Linear blocking relations as the sole blocking oracle; never infer ownership, blockers, or
  workflow state from free text when Linear exposes structured data.
- Never execute a parent issue while any of its sub-issues remains open.
- Name a new work branch `<ISSUE-ID>-<slug>`.
- Never require a separate pull request merely because work is represented by a sub-issue.
- Treat a checked box as an assertion of proof, not a progress marker: check a line only from a run,
  CI result, or observation you can name, never from a merge, a closure, or a tidier list.
- Never derive box state from workflow state: no merge, closure, or transition ever checks a box.
- A merge completes an issue only when every verification box holds, or when the issue carries no
  acceptance or evidence section at all; a section that holds no state proves nothing, so the issue
  goes to its team's review state with the unproven lines named, and `Done` stays an explicit human
  decision.
- Resolve the review state by enumerating the issue's own team's states and matching by name, never
  by state type; stop when the transport cannot enumerate them or the team exposes none.
