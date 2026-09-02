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
registry, approval, verification, and lifecycle workflow; this file only activates and routes it.

## Usage

Use after a second materially equivalent failure, after a recovery repeats an earlier failure, or
when reviewing what the harness should learn from completed work. Do not activate for one ordinary
error whose correction is already known.

Example: two agents independently omit the same required validation even though the applicable
project instructions are discoverable.

## Steps

1. Read [references/invariant-registry.md](references/invariant-registry.md) completely.
2. Execute its single authoritative JSON contract exactly as structured. Stop with a finding when
   the block is missing, malformed, duplicated, or contradicted; never supplement it from prose.

## Gotchas

- **Skipping the contract** — the registry flow loses its gates; load the reference before deciding.
- **Treating prose as authority** — summaries can drift from executable fields; use only the JSON
  block for domain decisions.
- **Continuing after invalid input** — a parse failure silently disables policy; return the validator
  finding instead of choosing a fallback.

## Constraints

- Keep the authoritative JSON block as the sole source of domain workflow rules.
- Keep this skill as a router; do not restate contract decisions, classes, thresholds, or gates here.
- Refuse the workflow when the authoritative block or its non-contractual surface is invalid.
