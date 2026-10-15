---
name: requirements-clarification
description: >
  Resolve undecided requirements within delegated authority. Use when missing product or operational
  policy changes material outcomes. Make sure to use this skill for such decisions, including
  authorized isolated proposals, even if asked to proceed. Excludes established requirements,
  discoverable facts, and inconsequential style.
metadata:
  category: dev
---

# Requirements Clarification

## Overview

Separate discoverable facts, delegated choices, and decisions requiring the user. Research first,
prepare authorized isolated proposals, and collect sensitive policy decisions for human validation
on the prepared result. Uncertainty alone does not require an interruption.

## Usage

Apply when undecided policy admits materially different outcomes, such as authentication boundaries,
migration compatibility, or artifact retention. Read the user's delegation and task limits before
choosing whether to decide, prepare a proposal, or ask. Permission to edit alone does not delegate
missing policy decisions. An interview, plan, or review-only request still authorizes no implementation.

Implementing or testing an established contract does not require clarification. The domain alone is
not a trigger; discoverable facts and inconsequential local choices do not require this skill.

## Steps

1. Inspect applicable instructions, ADRs in force, relevant code, tests, documentation and tools.
   Separate established facts and their sources from working assumptions and unresolved decisions.
2. Resolve facts available through the repository, authorized tools or current official dependency
   documentation. For a technical blocker, use the task's additional investigation budget: test a
   discriminating hypothesis and request an independent opinion only when useful and permitted.
   Track elapsed investigation across continuations and delegation; stop attempts adding no evidence.
3. For remaining choices, identify plausible alternatives and their concrete effects on behavior,
   architecture, data, security or acceptance. Apply established constraints and the user's delegation;
   do not reopen a settled decision or infer authority from an unanswered recommendation.
4. Within delegated technical and product choices, select a reversible option supported by the
   available evidence, state consequential assumptions, implement and verify it without an interview.
   Confirm isolation before any speculative implementation: fixtures and local dependencies must not
   touch real data, users, shared systems or external effects. A branch or a rollback of code alone
   does not establish this boundary.
5. For sensitive policy, including access rights, amounts, retention and contractual commitments,
   prepare an isolated implementation only when that preparation is delegated. Label the policy as a
   proposal, retain the alternatives and consequences, and test the proposed behavior. Collect these
   decisions for one human validation on the prepared result before delivery or exposure. Existing
   delivery authorization, passing tests and independent review cannot approve a new sensitive policy.
6. If authority or safe isolation is absent, an ADR contradiction remains, or essential evidence is
   unavailable, stop only the dependent action. Continue independent preparation and ask the smallest
   question requiring the user. When the investigation budget expires, report the evidence, attempted
   resolutions and missing decision or capability; never reset the budget or waive a required check.
7. Deliver the actual state: verified implementation for settled choices, prepared proposal awaiting
   validation for sensitive choices, or a specific unresolved blocker. Separate a proposal's passing
   checks from approval of its policy; an unapproved sensitive proposal is not ready for delivery.

## Gotchas

- **Asking for repository facts** — the user becomes a search interface and may provide stale
  information; inspect the applicable sources and tools first.
- **Stopping at every product choice** — delegated work waits unnecessarily; apply the established
  mandate and prepare a concrete result before collecting decisions reserved to the user.
- **Mistaking a branch for isolation** — tests or hooks may mutate shared data; inspect their actual
  dependencies before running a speculative implementation.
- **Treating a green test as policy approval** — the test validates an assumption the agent chose;
  retain the sensitive proposal's human-validation requirement.
- **Blocking the whole task on one unknown** — independent progress is lost; pause only work whose
  result depends on the unresolved answer.

## Constraints

- Never ask for information that can be established reliably in the authorized context.
- Never infer implementation or policy authority from this skill, silence, or a recommendation.
- Never expose a speculative choice to real data, users, shared systems or external effects.
- Never deliver an unapproved sensitive policy or treat independent review as the user's approval.
- Never override an ADR, expand the task, reset its investigation budget, or waive essential evidence.
- Never replace the requested work with a specification, plan, ticket, or brainstorming session.
- Never add host-specific routing, hooks, or frontmatter without measured failures and explicit
  authorization.
