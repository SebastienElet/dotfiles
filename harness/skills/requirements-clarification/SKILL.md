---
name: requirements-clarification
description: >
  Resolve undecided requirements within delegated authority. Use when missing decisions affect
  scope, cost, policy or acceptance. Make sure to use this skill whenever analysis or planning
  leaves a material prerequisite unresolved, even without implementation authority. Excludes
  settled decisions, discoverable facts and delegated technical choices.
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
missing policy decisions. An analysis, plan or review permits necessary clarification questions,
but authorizes no implementation.

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
   available evidence and state consequential assumptions without an interview. Apply it within the
   requested scope: use it in the analysis, or implement and verify when implementation is authorized.
   Confirm isolation before any speculative implementation: fixtures and local dependencies must not
   touch real data, users, shared systems or external effects. A branch or a rollback of code alone
   does not establish this boundary.
5. For sensitive policy, including access rights, amounts, retention and contractual commitments,
   prepare an isolated implementation only when that preparation is delegated. Label the policy as a
   proposal, retain the alternatives and consequences, and test the proposed behavior. Collect these
   decisions for one human validation on the prepared result before delivery or exposure. Existing
   delivery authorization, passing tests and independent review cannot approve a new sensitive policy.
6. For each material decision still reserved to the user, ask a direct question in the conversation,
   with a concrete recommendation and its consequences. Ask during analysis too: an unresolved
   prerequisite, a question buried in a document, or a future plan step is not a collected decision.
   Group related decisions when useful; do not re-ask one already pending. Suspend only the action
   dependent on the answer and continue independent work within the requested scope. If authority,
   safe isolation or essential evidence is missing, or an ADR contradiction remains, name that limit
   in the question. On investigation-budget expiry, include the evidence and attempted resolutions;
   never reset the budget or waive a required check.
7. Before delivery, account for every material prerequisite identified: resolved from a cited source,
   decided within delegation, or awaiting an answer to a question actually asked. Deliver the analysis
   or prepared result with those dispositions; never substitute “to be defined” or “remains to be
   decided” for the question. Separate a proposal's passing checks from approval of its policy;
   an unapproved sensitive proposal is not ready for delivery.

## Gotchas

- **Asking for repository facts** — the user becomes a search interface and may provide stale
  information; inspect the applicable sources and tools first.
- **Stopping at every product choice** — delegated work waits unnecessarily; apply the established
  mandate and prepare a concrete result before collecting decisions reserved to the user.
- **Mistaking a branch for isolation** — tests or hooks may mutate shared data; inspect their actual
  dependencies before running a speculative implementation.
- **Treating a green test as policy approval** — the test validates an assumption the agent chose;
  retain the sensitive proposal's human-validation requirement.
- **Listing prerequisites instead of asking** — the analysis appears complete while the decision
  remains untouched; ask the user directly, with the proposed choice and its consequences.
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
