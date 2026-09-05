---
name: code-simplify
description: >
  Simplify code to reduce understanding and maintenance costs. Use when asked to simplify code,
  lighten an implementation, or reduce abstractions. Make sure to use this skill whenever a code
  cleanup request targets conceptual complexity, even without naming refactoring. Excludes routine
  edits, merge reviews, and simplification of skills, issues, or the harness.
metadata:
  category: dev
---

# Code Simplify

## Overview

Reduce the concepts a maintainer must understand together while preserving the observable
contract. Start by examining what can disappear, then clarify what remains according to project
conventions. Fewer lines or files alone are not evidence of improvement.

## Usage

Use `$code-simplify <scope>` or `/code-simplify <scope>`, or a natural-language code simplification
request. For example: `Simplify the current diff in the invoice calculation code without changing
its public behavior.`

This skill owns a bounded refactoring pass, not merge approval, bug fixing, product scope decisions,
or a repository-wide cleanup. It does not activate merely because code was modified.

## Steps

1. Read applicable instructions, canonical user preferences, and project conventions. Use
   `code-search` for locating the relevant code and consumers. If the requested artifact is a skill,
   issue, or harness workflow, use its maintenance procedure instead.
2. Bound the surface before editing. Honor the supplied files, symbols, or diff. Without an explicit
   scope, inspect Git status and both staged and unstaged diffs; announce the selected changed code
   and exclusions. Account for relevant new files shown by status. Never infer a reference branch;
   if the diff is empty or the intended surface remains materially ambiguous, ask only for the
   missing scope. Reading dependencies does not authorize modifying neighboring modules.
3. State the contract to preserve: relevant outputs, errors, side effects and their ordering,
   public interfaces, compatibility, business invariants, and security boundaries. Read applicable
   decisions and consumers, including externally deployed callers where relevant. Treat tests as
   evidence of implementation, not authority over an active architectural decision. Report a
   discovered defect separately; do not silently repair or document it as intended behavior.
4. Examine removal candidates first: derivable state, unused options, delegation without a
   responsibility, speculative generalization, duplicated sources of truth, obsolete calculations,
   and unnecessary dependencies. Establish why each candidate is dispensable using its consumers,
   registration or configuration paths, and contract evidence. An empty search does not prove
   non-use; unresolved dynamic or external consumers prevent that deletion.
5. Classify candidates before editing: **means simplified, contract preserved**, or **requirement
   or contract change proposed**. Only the first category belongs to the authorized refactoring.
   For the second, explain the behavior lost or changed and wait for an explicit decision on that
   candidate; continue independent, authorized simplifications. A feature that seems unnecessary
   is still part of the contract until that decision changes it.
6. Choose the smallest useful change to what remains. Favor explicit intent, cohesion, locality,
   and fewer concepts to track together. Keep a single-use function when its name clarifies a
   meaningful step; do not replace it with an inline predicate or named local value merely because
   it has one caller. Preserve responsibility and trust boundaries; do not merge concerns to reduce
   file count or extract an abstraction merely to eliminate repetition. State the concrete gain
   and any material expansion of scope before editing; expansion requires a decision.
7. Inventory relevant existing oracles and identify the behavior each can actually detect before
   modifying code. Follow the project's canonical validation strategy. For unprotected historical
   code, propose the smallest characterization of the threatened observable behavior; establish
   that protection before a risky refactor, following existing authorization for tests. For a
   security or refusal control, use `enforcement-code` before changing it. Upstream validation alone
   does not prove a downstream trust-boundary check redundant. If the required specialized
   procedure is unavailable, pause that control's change and report the missing procedure.
8. Apply the supported simplifications and run the relevant checks. Use native syntax, schema,
   dry-run, or public execution for declarative configuration. Do not add mirror suites,
   tests of tests, or reflexive new gates. Preserve failure behavior as well as successful results;
   name gaps that prevent concluding equivalence rather than inventing a substitute oracle.
9. Stop when further gains are speculative or materially expand the scope. Report the selected
   surface, concrete reduction in understanding or maintenance cost, contract evidence, checks
   actually run and their environment and limits, and any separate decisions or defects.
   `No useful simplification found` is a valid result; speedups and automation are not required.

## Gotchas

- **Treating a wrapper as empty by its size** — an interface, transaction, or policy boundary can
  look like delegation; verify its responsibility and consumers before removing it.
- **Inlining every single-use helper** — a meaningful step loses its name and increases mental
  load; retain the helper when that name explains intent.
- **Trusting upstream validation** — alternate callers or entry points may bypass it; establish
  the actual trust boundary and follow `enforcement-code` before changing a refusal.
- **Calling a feature superfluous** — deleting it changes the requirement; propose that tradeoff
  for an explicit decision instead of including it in the refactor.
- **Counting green checks as equivalence** — uncovered errors or effects may change unnoticed;
  relate each oracle to the threatened behavior and disclose what it cannot establish.

## Constraints

- Never silently change observable behavior, fix defects, or remove a requirement as simplification.
- Never guess a diff base, widen scope opportunistically, or use search absence as proof of non-use.
- Preserve useful names and responsibility boundaries; do not optimize for syntax or file count.
- Do not duplicate canonical project preferences or replace specialized safety procedures.
- Never claim unexecuted validation, merge readiness, or universal behavioral equivalence.

## References

- [references/validation.md](references/validation.md) — targeted scenarios and evidence boundaries;
  read when validating this skill itself.
- [evals/trigger-queries.json](evals/trigger-queries.json) — prepared activation prompts, not run results.
