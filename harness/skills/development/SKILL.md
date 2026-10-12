---
name: development
description: >
  Run Claude Code development through implementation, checks and independent Codex review.
  Use when Claude is asked to implement a feature, fix a bug, refactor, or change configuration.
  Make sure to use this skill for every authorized development task, even outside another skill.
  Excludes planning-only, explanation and review-only requests.
compatibility: Requires the repository toolchain, review-fix and the official codex@openai-codex plugin in Claude Code.
metadata:
  category: dev
---

# Development

## Overview

Own the task-wide development sequence in Claude Code: implement the requested change, run its
checks, then obtain independent Codex review before delivery. Apply to plain-language requests as
well as work started through another skill. This is instruction-based orchestration; automatic
skill selection and completion are not host-enforced guarantees.

## Usage

Use `/development <task>` or apply it to an authorized Claude development request, for example:
`Fix this bug and deliver the verified change.` Include declarative configuration changes even
when they need no TDD. Planning, explanation and review alone authorize no implementation.

An implementation request authorizes the local corrections necessary for its requested outcome,
including bounded Codex correction of demonstrated defects in that scope. Preserve stricter user
limits. Commits, pushes, PR publication, approvals and merges retain their existing authorization
requirements. Other skills own their specialized steps; this skill owns when development ends.

## Steps

1. **Establish the task and prerequisites.** Read repository instructions, current Git state,
   relevant sources and ADRs. Identify the requirements, allowed files, existing changes and
   verification boundary. Resolve only material unknowns with `requirements-clarification`.
   Before implementation, verify that `review-fix`, its Codex companion and the required checks
   are available. Use its Claude transport reference for runtime setup and keep the Stop review
   gate disabled. Missing runtime or essential evidence leaves the task incomplete; do not install
   dependencies or silently drop the final review during development.
2. **Respect the enclosing workflow.** For a Linear task, let `linear-start` and `linear-workflow`
   own eligibility, branch, tracker and PR state. Other specialized workflows retain their own
   authority and boundaries. When this task repairs an open PR, use the authorized `pr-fix`
   workflow for repair and independent review; do not start a concurrent local repair loop.
   Ordinary local implementation remains here even when no other skill was invoked.
3. **Implement the settled outcome.** Preserve unrelated work. Apply `tdd` for owned executable
   behavior and native or existing validation for declarative changes. Use the relevant domain
   skills without turning each test, tool call or subtask into a separate delivery stage.
   Record the complete task delta and any intentionally excluded changes.
4. **Run the relevant checks.** Index intended new files before format, lint and typecheck.
   Run the repository barrier appropriate to the actual change, with named environment and limits.
   Failed checks are unfinished development, not permission to skip the final stage or weaken
   assertions. Fix only authorized failures and keep missing essential evidence explicit.
5. **Finish through independent review once.** After implementation and checks pass, activate
   `review-fix` on the complete local task delta with requirements, checked source references,
   candidate identity and raw verification evidence. Explicitly authorize Codex to correct
   demonstrated defects only within the task's original scope and relevant tests. Suspend Claude's
   writes and wait for the workflow's fresh final review before delivering. It owns the two-pass
   bound, dispositions and incomplete outcomes. Do not re-enter `development` or `review-fix`
   inside its delegated correction. If `pr-fix` already owns repair and final review, retain that
   sequence and report its actual result instead of starting this second loop.
6. **Deliver the observed result.** Report the final candidate, changed behavior, finding
   dispositions, actual checks, environment and remaining limits. A missing, stale or incomplete
   review keeps the task incomplete. Do not say development is finished merely because code was
   written, checks passed, a child returned or a task was marked completed. Preserve any existing
   publication and approval checkpoints in the enclosing workflow.

## Gotchas

- **A plain request bypasses specialized skills** — its final review disappears; this workflow
  applies to Claude development regardless of how the task started.
- **TDD becomes the outer workflow** — its green test ends delivery too early; retain the task-wide
  final review after all implementation and checks finish.
- **Every child starts the finishing sequence** — reviews recurse and the pass counter resets;
  only the outer development owner invokes the final stage.
- **A configuration edit skips TDD** — skipping a value test can also skip review by mistake;
  use native validation and retain independent review.
- **An existing PR repair gets a second loop** — competing writers and verdicts diverge; retain
  the `pr-fix` owner and its final independent review.

## Constraints

- Never start implementation, repair or publication from a planning or review-only request.
- Never deliver completed development without the applicable independent final review.
- Never override user limits or expand beyond the requested outcome and confirmed corrections.
- Never duplicate the `review-fix` or `pr-fix` loop inside a delegated correction.
- Never enable a Stop review gate or present automatic skill selection as technically enforced.
- Never install missing runtimes or replace missing evidence with a successful-looking result.
