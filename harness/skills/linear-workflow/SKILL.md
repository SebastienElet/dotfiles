---
name: linear-workflow
description: >
  Apply the shared Linear and Bitbucket work invariants. Use when another Linear execution skill
  composes the common workflow policy. Make sure to use this skill whenever starting, resuming, or
  finishing Linear-backed implementation, even if the request mentions only an issue ID.
compatibility: Requires access to the relevant Linear workspace and Bitbucket repository.
metadata:
  category: dev
---

# Linear Workflow Policy

## Overview

Provide the single reusable policy for execution workflows backed by Linear and Bitbucket. This
skill defines invariants only; the composing workflow owns its procedure and transport selection.

## Usage

Load `linear-workflow` alongside a specific execution skill such as `linear-start`. Do not invoke it
alone to shape, implement, review, or finish an issue.

## Steps

1. Read [transport adapters](references/transports.md) before accessing Linear and
   [Bitbucket operations](references/bitbucket.md) before accessing pull requests.
2. Read ownership, hierarchy, relations, and work state from structured Linear fields.
3. Read branch and pull-request state from Git and Bitbucket.
4. Apply every constraint below, then return control to the composing workflow.

## Gotchas

- **Trusting description text for ownership or blocking** — stale prose overrides structured state;
  use Linear's assignee and relation fields.
- **Starting a parent with open sub-issues** — work bypasses the executable leaves; select an
  assigned, unblocked sub-issue instead.
- **Equating issue and pull-request granularity** — related changes gain artificial review
  boundaries; let the implementation scope determine whether sub-issues share a pull request.

## Constraints

- Linear is the source of truth for work; Bitbucket is the source of truth for Git and pull-request
  state.
- Work only on issues assigned to the current user.
- Treat Linear blocking relations as the sole blocking oracle; never infer ownership, blockers, or
  workflow state from free text when Linear exposes structured data.
- Never execute a parent issue while any of its sub-issues remains open.
- Name a new work branch `<ISSUE-ID>-<slug>`.
- Never require a separate pull request merely because work is represented by a sub-issue.
