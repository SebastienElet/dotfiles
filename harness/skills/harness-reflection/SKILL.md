---
name: harness-reflection
description: >
  Turn repeated agent failures into evidence-backed harness improvements. Use when the same material
  outcome fails twice, a failed approach recurs after recovery, or the user asks what the harness
  should learn. Make sure to use this skill whenever retries stop adding information, even if
  reflection or learning is not requested.
metadata:
  category: dev
---

# Harness Reflection

## Overview

Stop unproductive retry loops and turn them into one falsifiable learning candidate. Recurrence is
a signal to investigate, never proof that a permanent rule is correct. The skill may propose a
harness change, but promotion remains versioned, evidence-gated, and human-approved.

## Usage

Use after a second materially equivalent failure, after a recovery that repeats an earlier failure,
or when reviewing what the harness should learn from a completed task. Do not activate for one
ordinary command error that immediately reveals its correction.

Example: two agents independently omit the same required validation even though the applicable
project instructions are discoverable.

## Steps

1. Name the repeated observable outcome, the intended outcome, and why the attempts are materially
   equivalent. Stop repeating the unchanged approach.
2. Preserve the smallest useful evidence: failing command or factual `pr-feedback` finding, relevant
   environment, agent, repository, harness fingerprint when available, and the recovery that
   succeeded or failed. Do not interpret, rewrite, or mutate `pr-feedback` evidence.
3. Classify the cause as `task-specific`, `owned-defect`, `external-transient`, `missing-capability`,
   or `harness-gap`. Choose `harness-gap` only when a reusable instruction, skill, tool preference,
   sequence, avoidance rule, or routing decision could plausibly change the outcome.
4. Check current instructions, skills, ADRs, and official dependency behavior before proposing a
   compensating rule. Fix an owned defect instead of teaching the harness its workaround.
5. If the result is not `harness-gap`, return `skip` using the required report in the reference;
   mark the registry lookup, control kind, surface, and oracle as `not applicable` with their reason.
6. For `harness-gap`, read [references/invariant-registry.md](references/invariant-registry.md), then
   inspect the named registry before proposing a rule. Classify the registry cause after
   `harness-gap`, search for a matching source or invariant, and return exactly `skip`, `link`, or
   `propose` using the required report, even when a neighboring rule exists or sources are missing.
7. For `propose`, include the trigger, desired behavior, scope, supporting evidence, counterexample,
   falsifier, expiry condition, and the cheapest behavioral trial that could disprove the candidate.
   Keep every proposal session-local until explicit approval. Use `skill-manager` for a skill change
   and `agent-instructions` for instruction discovery or deployment changes.
8. After explicit approval, declare the surface, Claude, Codex, and Cursor consumers, and the
   appropriate oracle. Validate the registry with its CLI before presenting the change as valid.
9. Promote only after the trial changes the target behavior in three independent sessions without a
   contradictory result. Roll back on two failed trials, one safety regression, or a user veto.

## Gotchas

- **Counting unrelated failures** — a flaky network request and a wrong repository assumption do
  not form a pattern; compare observable outcome, cause, and recovery before reflecting.
- **Learning from recurrence alone** — repeated mistakes can share one bad premise; require a
  falsifiable behavioral trial before promotion.
- **Encoding an owned defect** — a workaround becomes permanent policy; fix the defect or record a
  focused issue instead.
- **Writing broad instructions first** — every future task pays the context cost; prefer the
  narrowest existing skill or tool boundary proven by the trial.
- **Duplicating a named invariant** — the evidence splits across records; inspect the registry and
  return `link` when the source belongs to an existing invariant.

## Constraints

- Never edit instructions, skills, hooks, repository files, or harness state without explicit user
  approval of the proposed trial or change.
- Never let a learned rule modify its own evaluator, evidence threshold, promotion policy, or
  authority.
- Never include secrets, credentials, private prompt content, or raw transcripts in a candidate.
- Never claim that activation scenarios or repeated failures prove improvement; name the behavioral
  oracle and the environment in which it ran.
- Keep candidates scoped per agent unless cross-agent trials independently support shared behavior.
- Never promote or mutate the registry without explicit approval, even under time pressure.
