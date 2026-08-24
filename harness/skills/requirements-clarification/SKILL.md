---
name: requirements-clarification
description: >
  Clarify requirements before implementation. Use when a request adds authentication or migrates a
  configuration manifest between versions without defining actors, trust boundaries, compatibility,
  data loss, failure behavior, or acceptance criteria. Make sure to use this skill for any
  authentication or migration request with unresolved material decisions, even if the user asks to
  proceed.
metadata:
  category: dev
---

# Requirements Clarification

## Overview

Separate missing detail from decisions that materially change the result. Inspect first, maintain a
verifiable preflight, and ask only questions whose answers affect behavior, architecture, data,
security, or acceptance criteria.

## Usage

Apply before implementation when the request may admit materially different outcomes. For example,
use it before adding authentication when the actors, trust boundary, or failure behavior are not
established.

## Steps

1. Load every applicable instruction and inspect the smallest relevant repository surface,
   including authoritative decisions, documentation, neighboring code, tests, and configured tools.
2. Maintain three distinct preflight groups: established facts with their sources, explicit working
   assumptions, and unresolved unknowns. Present the groups when they affect a decision or make the
   result independently verifiable.
3. Resolve unknowns available from the repository, authorized tools, or current official dependency
   documentation. Do not ask the user to supply discoverable facts.
4. For each remaining unknown, name at least two plausible answers and their concrete effect. Keep it
   only when the answers differ in behavior, architecture, data, security, or acceptance criteria.
5. When a material unknown remains, pause only the affected work and ask the minimum question needed
   to resolve it. State the concrete consequence of each answer and continue independent work.
6. When no material unknown remains, proceed without a clarification interview. Follow a verified
   repository convention or a safe, local, reversible default, and state an assumption only when it
   helps verify the outcome.
7. Before implementation, confirm that no retained question concerns a discoverable fact or a
   preference outside the five material criteria.

## Gotchas

- **Treating prompt length as uncertainty** — a short request can be complete and a long one can hide
  a material decision; classify the consequences instead of the wording.
- **Asking for repository facts** — the user becomes a search interface and may provide stale
  information; inspect the applicable sources and tools first.
- **Turning clarification into design facilitation** — work stalls behind a workshop that was not requested;
  ask only the smallest questions that separate materially different outcomes.
- **Blocking the whole task on one unknown** — independent progress is lost; pause only work whose
  result depends on the unresolved answer.

## Constraints

- Never ask for information that can be established reliably in the authorized context.
- Never block on an unknown unless plausible answers differ in behavior, architecture, data,
  security, or acceptance criteria.
- Never mutate the affected implementation while a material decision remains unresolved.
- Never replace the requested work with a specification, plan, ticket, or brainstorming session.
- Never add host-specific routing, hooks, or frontmatter without measured failures and explicit
  authorization.
