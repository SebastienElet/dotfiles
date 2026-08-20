---
name: pr-fix
description: >
  Repair an open pull request after an independent merge review. Use when asked to find and directly
  correct blocking or objective non-blocking defects on a PR. Make sure to use this skill whenever
  edits, commits, or a push to a PR branch are authorized, even if the user only says to fix the
  review findings.
compatibility: >
  Requires the `pr-verdict` skill, authenticated `gh` or `bkt`, write access to the PR source branch,
  and the repository's validation toolchain.
metadata:
  category: dev
---

# PR Fix

## Overview

Turn a head-specific verdict into reviewed corrections on the contributor's branch. The explicit
repair request authorizes edits, commits and a standard push to that branch; it does not authorize a
force-push, issue creation or verdict publication. The repaired head earns its own verdict from a
fresh context because the context that wrote a fix cannot independently validate it.

## Usage

`/pr-fix <pr-number|pr-url>` — review the current head, apply bounded corrections, push them to the
PR source branch, then return a verdict on the pushed head.

Typical cases: "fix the blockers on PR 1042", "review this PR and correct the issues directly", or
"we can push small review fixes to the contributor's branch". A request only to judge, approve or
re-review a PR belongs to `pr-verdict` and must not mutate the branch.

## Steps

1. **Draft the verdict.** Activate `pr-verdict`, read its routed references, and run phases 1 through
   5 on the exact head SHA. Return the verdict to this workflow without opening a ticket or
   publishing it. Resolve the source repository and source ref from forge metadata; never infer the
   push target from the local branch name or `origin`.

2. **Bound the repair.** Present one repair slate before editing: every blocker has its failure
   mechanism, broken invariant and lift test; at most three non-blocking items may follow, and only
   when they are objective, localized and observable. Continue without another confirmation because
   invoking this workflow authorized the repair. Stop before editing when a correction requires an
   unresolved product, architecture, migration or public-contract decision.

3. **Isolate the work.** Create a clean temporary worktree at the anchored SHA so the caller's
   checkout and uncommitted work remain untouched. Preserve all author commits and unrelated changes.
   Keep the worktree when a failed push or unresolved conflict leaves local commits the user may need.

4. **Prove each correction.** Add the cheapest failure-path test that reproduces each mechanism,
   observe it fail for the expected reason, then correct the cause. A declarative change with no
   useful behavioral test gets the repository's relevant smoke or validation check. Do not turn
   naming, style or speculative cleanup into repair work.

5. **Run the real barrier.** Execute the merge-blocking lint, typecheck and test commands on the
   repair worktree, including the changed extensions and the failure-path tests. Record counts and
   evidence gaps as `pr-verdict` requires. Keep independent mechanisms in separate commits and
   coupled corrections together.

6. **Re-anchor before pushing.** Query the forge again and validate that the source repository,
   source ref and head SHA are all present and still identify the anchored head. If the head moved,
   do not push: anchor the new head, inspect the overlap, and reapply only corrections that remain
   valid. Push the commits normally to the resolved source ref; never use a force option. A rejected
   push or missing permission leaves the commits local and becomes an explicit delivery blocker.

7. **Judge the pushed head independently.** Resolve the SHA now shown by the PR and delegate a full
   `pr-verdict` review of that exact head to a fresh context, including its barrier. Return only that
   head's final verdict as current. Publish it only when the user separately confirms publication or
   explicitly included publication in the original request.

8. **Close the repair.** Report the original findings, commits pushed, final SHA, final verdict and
   residual evidence gaps. Do not create a fix ticket for a defect corrected by this run. Remove the
   temporary worktree only after its commits are pushed and it is clean.

## Gotchas

- **The repair context reviews its own head** — it shares the blind spot that produced the fix and
  can approve the same defect twice. Delegate the final `pr-verdict` sweep and barrier to a fresh
  context.
- **The destination is inferred from `origin`** — a fork PR is pushed to the wrong repository or
  fails after all work is complete. Resolve and validate the forge's source repository and ref before
  editing and again before pushing.
- **The contributor pushes concurrently** — corrections based on the old head become stale or a
  push collides with new work. Re-query the exact head, refuse every force option, and re-anchor when
  it moved.
- **Small remarks become a cleanup pass** — the PR gains unrelated churn and review risk. Keep at
  most three objective, localized non-blockers and drop preferences.
- **Publication is inferred from repair authority** — a team-visible verdict appears without
  consent. Treat branch mutation and verdict publication as separate permissions.
- **A failed push worktree is discarded** — the only copy of useful commits becomes hard to recover.
  Preserve the worktree and report the commit SHAs when the branch cannot be updated.

## Constraints

- Never mutate a PR unless the user explicitly asked to correct it or invoked `pr-fix`.
- Never force-push, overwrite a moved head or guess the PR source repository or ref.
- Never publish a verdict, create an issue or open another PR from repair authority alone.
- Never claim a defect fixed before its failure-path test and the merge-blocking barrier pass.
- Never let the context that wrote the repaired head perform its final failure-class sweep.
- Never repair subjective preferences or broaden the change beyond proven findings.
- Never remove a temporary worktree that contains unpushed commits or uncommitted changes.
