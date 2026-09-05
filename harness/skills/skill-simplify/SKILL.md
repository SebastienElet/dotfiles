---
name: skill-simplify
description: >
  Simplify an identified skill's content. Use when asked to remove unnecessary instructions or
  shorten its procedure. Make sure to use this skill whenever reducing a named skill's reading
  burden, even if simplification is implicit; use skill-manager directly for conformity,
  installation, or index-only work.
metadata:
  category: ops
---

# Skill Simplify

## Overview

Find instructions that can disappear without losing useful behavior, then shorten what remains.
This skill owns content analysis and the simplification proposal; `skill-manager` owns editing,
conformity, indexing, and projection verification.

## Usage

`$skill-simplify <skill> [diagnose or simplify]` or `/skill-simplify <skill>`.
For example: “Diagnose unnecessary steps in the release-checks skill; do not edit it.”
An ordinary SKILL.md edit does not trigger this procedure.

## Steps

1. Identify the target and whether the request authorizes changes or diagnosis only. Ask for the
   target if ambiguous. Route conformity, installation, or index-only requests directly to
   `skill-manager`.
2. Read the complete target and follow references needed to understand its behavior, including
   exceptional paths. Record its useful result, inputs, activation and exclusion conditions,
   decisions, external effects, refusals, stop conditions, and expected guarantees. Preserve the
   original version for comparison without writing during a diagnosis.
3. Work backward from that result: which concrete defect does each step prevent? Identify repeated
   obligations, canonical instructions available at the point of use, and native capabilities
   that could make a step unnecessary. Investigate missing rationale through relevant sources,
   history, or a focused experiment; unresolved uncertainty means retain the step and report it.
4. Seek deletions first: steps with no useful effect, demonstrably obsolete procedures, normative
   duplicates, speculative variants, ceremonial outputs, and examples adding no new decision.
   For duplicates, verify identical scope, conditions, and authority before retaining one
   authoritative formulation. Judge rare exceptions by the consequence they prevent.
5. Shorten the surviving procedure into explicit actions. Keep non-obvious reasons that prevent
   real errors. Evaluate the whole reading path, including required references; extract material
   only when conditional loading reduces that path, and specify when each reference is needed.
6. Present a proposal and conceptual diff: for every material change, name the instruction,
   evidence, and behavior preserved, removed, or modified. Include unresolved questions and
   rejected reductions. If a local convention creates filler, propose a separate convention
   change instead of bypassing it.
7. Compare original and proposed versions on identical scenarios with fixed expectations:
   relevant activation, a nearby non-trigger, normal result, and an important exception. Check
   permissions, verification, and refusals explicitly. Revise or withdraw changes that regress
   behavior; distinguish written scenarios, manual analysis, and actual executions. When testing
   this skill itself, use the fixtures in `evals/behavior-scenarios.md` and activation queries in
   `evals/trigger-queries.json`; they are not part of routine target simplification.
8. For diagnosis only, deliver the proposal without writing. When changes are authorized and the
   contract is clear, invoke `skill-manager fix <target>` with that contract, evidence, and
   comparison scenarios; leave its editing and validation procedure there. Resolve material
   uncertainty before the affected change. Report observed benefit and remaining limitations:
   fewer words alone do not demonstrate better behavior.

## Gotchas

- **Merging similar-looking obligations** — different conditions can disappear; compare their
  scope and authority before deleting either formulation.
- **Deleting a rarely used refusal** — frequency hides the consequence it protects; retain it
  unless evidence establishes that protection is obsolete or preserved elsewhere.
- **Hiding text in references** — a smaller SKILL.md can require more navigation and the same
  reading; reject relocation without a shorter relevant reading path.

## Constraints

- A diagnosis authorizes no persistent writes; simplification does not authorize executing the
  target's external effects.
- Never weaken a permission, verification, refusal, trigger, or important exception under cover
  of editorial simplification; a behavioral policy change needs its own explicit authorization.
- Never change conventions or evaluators to make a simplification pass, or recursively simplify
  this skill during its own creation or validation.
- Keep removal of this entry an explicit option if subsequent observations show no benefit over
  `skill-manager fix` alone.
