---
name: code-review
description: >
  Review a local diff for standards and requirement compliance. Use when reviewing current changes,
  a branch or work before committing. Make sure to use this skill whenever local implementation
  needs review, even if the changes are uncommitted. Excludes open-PR merge verdicts and PR repairs.
license: MIT
metadata:
  category: dev
---

# Code Review

## Overview

Review two independent axes: compliance with repository standards, and correctness against the
requested behavior. A green check is evidence, not a review. Keep findings tied to the actual
candidate, distinguish defects from style preferences and inspect every included change.

Adapted from [code-review by Matt Pocock](https://github.com/mattpocock/skills/tree/d81f3a183412e71a5b1e84ca21bc1a35eea03a60/skills/engineering/code-review).
The upstream license is preserved in [LICENSE](LICENSE).

## Usage

Use `$code-review <scope>` or `/code-review <scope>`, or apply it to a requested local review.
For example: `Review the current staged and unstaged changes before committing.` Without a
supplied scope, use the current working changes. This skill returns local findings; it neither
publishes a review nor grants a merge verdict. `pr-verdict` retains its explicit invocation contract;
an authorized `pr-fix` retains responsibility for open-PR repair.

## Steps

1. Freeze the scope with `git status --short`, `git rev-parse HEAD` and the requested comparison.
   For current changes, inspect `git diff --cached`, `git diff` and each relevant untracked file
   reported by status. Read an untracked file directly; index it before repository checks when it
   belongs to the intended change. Never stage an unrelated file merely to review it.
2. For a branch or supplied ref, resolve the ref and capture `git diff <ref>...HEAD` and
   `git log <ref>..HEAD --oneline`. Report staged, unstaged or untracked changes outside that
   committed comparison explicitly. Include them when the requested scope is the complete current
   implementation. An invalid ref blocks that comparison; an empty comparison is not an approval
   and does not hide pending worktree changes. Review the captured diff if no changes remain after
   a commit; do not infer an unrelated base branch.
3. Locate and read the relevant standards, ADRs and consumers. Establish the requirement from
   the user's request, a supplied spec or linked issue via the existing authorized tracker tools.
   No dedicated tracker configuration or setup skill is required. If the requirement is unavailable,
   continue the standards review and report the spec axis as unavailable.
4. Run independent standards and spec reviews with fresh subagents when available. Give each
   the exact scope, file inventory and source references, without the author's prior verdict.
   Locate sources in the main thread before delegation. Reviewers must not edit or publish.
   If subagents are unavailable, perform the axes sequentially and state the independence limit.
5. For standards, identify concrete violations with the rule and location. Use
   [references/smells.md](references/smells.md) for design heuristics only. Repository conventions
   override those heuristics; style and abstraction preferences alone do not block the change.
6. For spec, trace affected behavior through the consumers and report missing, wrong or unsolicited
   behavior, including failure paths and compatibility. Test evidence must exercise the boundary
   owning the behavior. Use `proof-integrity-review` when verification mechanisms or agent review
   rules change; preserve its independence and execution-evidence requirements.
7. Consolidate the two axes into one concise severity-ranked report, retaining each finding's axis,
   location, failure mechanism and source. Omit unsupported suspicions. Report check outcomes,
   environment, untested supported targets and unresolved limitations; do not equate style advice
   or passing checks with approval. If the candidate changed during review, refresh the affected
   scope and evidence before concluding.

## Gotchas

- **Comparing only commits during a pre-commit review** — pending defects disappear; inspect
  index, working tree and relevant untracked files as separate inputs.
- **Requiring an issue-tracker setup file** — local review becomes a tooling migration; use the
  existing request or spec and the repository's established tracker access.
- **Treating a code smell as a certain defect** — subjective advice blocks delivery; name the
  heuristic and reserve blocking findings for demonstrated contract or standard violations.
- **Calling a local review a merge verdict** — authority and evidence are overstated; retain the
  separate `pr-verdict` activation and publication rules.

## Constraints

- Never omit pending changes that belong to the requested review scope.
- Never edit, commit, publish or repair files merely because a review was requested.
- Never treat empty diffs, green checks or stylistic preferences as a merge decision.
- Never invoke `pr-verdict` outside its authorized activation contract.
- Never claim reviewer independence or execution evidence that was not actually available.
