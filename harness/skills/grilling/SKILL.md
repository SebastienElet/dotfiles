---
name: grilling
description: >
  Run a design interview by dependent decisions. Use only on explicit `$grilling` or `/grilling`
  invocation, or an authorized interview from `grill-me`, `wayfinder` or
  `improve-codebase-architecture`. Never select it for ordinary implementation clarification or
  an unrelated workflow.
license: MIT
metadata:
  category: product
---

# Grilling

## Overview

Challenge the user's thinking until the consequential branches of the design are understood.
Organize decisions by their prerequisites, ask questions that can be answered now, and use each
answer to expose the next decisions. Facts are the agent's research responsibility; policy and
product choices belong to the user.

Adapted from [grilling by Matt Pocock](https://github.com/mattpocock/skills/tree/d81f3a183412e71a5b1e84ca21bc1a35eea03a60/skills/productivity/grilling).
The upstream license is preserved in [LICENSE](LICENSE).

## Usage

Invoke `$grilling <decision>` or `/grilling <decision>`, or use it when a user-invoked `grill-me` or
`wayfinder` delegates an interview. A user-invoked `improve-codebase-architecture` may also compose
it after the user selects a candidate. These compositions permit interviewing and source research only;
it does not authorize implementation, persistent documentation or issue publication. The caller
restriction is instruction-level; model invocation remains available for the authorized composition.

## Steps

1. Locate and inspect the relevant flow, consumers, domain glossary and ADRs in force. Separate
   established facts, working assumptions and undecided choices. Research discoverable facts
   yourself; delegate bounded reading only after locating the relevant sources.
2. Map the decisions and their dependencies. Identify the frontier: choices whose prerequisites
   are already settled. A question depending on an unanswered choice belongs to a later round.
3. Ask a small group of independent frontier questions, with a recommendation and its concrete
   trade-off for each. Prioritize decisions affecting the outcome, users, scope, security, data,
   error states, compatibility or acceptance criteria. Challenge consequential assumptions using
   realistic failure and edge scenarios rather than adding hypothetical features.
4. Wait for the user's answers before using those choices downstream. Investigate independent facts
   while waiting. Do not interpret silence or a default option as a decision.
5. Incorporate the answers, surface contradictions and recompute the frontier. Keep an ADR in force
   authoritative; an interview cannot silently supersede it. Continue until material decisions are
   settled or a named blocker requires unavailable evidence or another decision-maker.
6. Return a concise decision log: intended outcome, settled choices, constraints, explicit
   assumptions and remaining blockers. State the limit when a branch remains unresolved. Return
   to the caller without automatically writing a spec or starting implementation.

## Gotchas

- **Asking a dependent question too early** — the user answers against an unstable premise; defer
  it until its prerequisite is settled and recompute the frontier after each round.
- **Asking the user for a repository fact** — stale recollection replaces evidence; inspect the
  source or identify the access limitation before presenting a choice.
- **Grilling every inconsequential preference** — the workshop grows without improving the design;
  focus rounds on choices with a concrete effect on the requested outcome.
- **Finalizing unresolved branches as assumptions** — the resulting specification appears settled;
  retain the blocker and its consequence explicitly in the decision log.

## Constraints

- Select only for direct invocation or the authorized caller compositions described in Usage.
- Never override an ADR or treat a proposal as an established requirement.
- Never decide a material user choice from silence or an unanswered recommendation.
- Never start implementation, write persistent documentation or publish an issue from this interview.
