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

Route repeated-failure analysis through one executable contract. The contract owns the diagnostic,
proposal, approval record, owner routing, registry, verification, and lifecycle workflow; this file
only activates and routes it.

## Usage

Use after a second materially equivalent failure, after a recovery repeats an earlier failure, or
when reviewing what the harness should learn from completed work. Do not activate for one ordinary
error whose correction is already known.

Example: two agents independently omit the same required validation even though the applicable
project instructions are discoverable.

## Steps

1. Read [references/invariant-registry.md](references/invariant-registry.md) completely.
2. Execute `initialWorkflowOrder` exactly as structured.
3. When its diagnostic result is `harness-gap`, execute `harnessGapWorkflowOrder`, then prepare the
   selected branch's proposal and exact manifest before requesting contextual human approval. The
   approval record is an attestation whose origin the code cannot authenticate.
4. For `conditional-skill`, require `targetSkillPath` to resolve to an existing triggerable user
   skill, distinct from this router and deployed to every declared user-skill consumer. Use
   `skill-manager` to apply `candidateTextExact` to that exact target and run its doctor and contracts.
5. For every file-backed surface change, resolve the exact `surfaceOwners` entry, use its required
   skill, and run its named doctor and contracts. Then resolve `workflowRoutes.manifestValidation`;
   its export only checks the exact manifest and the already-applied surface snapshot. Write only
   the approved registry replacement after that check, then run `workflowRoutes.registryValidation`.
6. For an enforceable control, use the exact `externalControlRoutes` entry and its separate owner
   workflow. Present that owner-specific exact diff for approval, run its contracts, and record the
   registry only afterward; the generic manifest validator does not accept those implementation paths.
7. For retirement, remove the exact candidate text through the required owner, record the new exact
   approval attestation, preserve the historical fields, and follow the same validation and registry order.
   Stop with a finding when a route or owner is missing, malformed, unresolved, duplicated, or
   contradicted; never supplement it from prose.

## Gotchas

- **Skipping the contract** — the registry flow loses its gates; load the reference before deciding.
- **Treating prose as authority** — summaries can drift from executable fields; use only the JSON
  block for domain decisions.
- **Continuing after invalid input** — a parse failure silently disables policy; return the validator
  finding instead of choosing a fallback.
- **Treating validation as application** — the manifest export writes no surface; apply through the
  required owner before validating its resulting snapshot.

## Constraints

- Keep the authoritative JSON block as the sole source of domain workflow rules.
- Keep this skill as a router; do not restate contract decisions, classes, thresholds, or gates here.
- Keep this closed router byte-stable unless its routing contract changes; reject it as every
  `conditional-skill` target.
- Refuse the workflow when the authoritative block or its non-contractual surface is invalid.
- Never use the manifest validator as a surface writer or claim it authenticates an approval origin.
