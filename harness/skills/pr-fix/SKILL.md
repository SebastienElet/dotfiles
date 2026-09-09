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
Accumulate corrections in a local journal across passes and sessions. Publish one cumulative repair
record only when the current head is independently approved and its required remote CI is green.
This factual record engages no merge decision and needs no separate publication consent.

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

   Before repairing, open or create
   `~/.local/state/pr-fix/<forge-host>/<destination-owner>/<destination-repository>/<pr>/journal.md`
   from `assets/repair-journal.md`. Derive the identity from the canonical PR URL, verify the URL
   stored in an existing journal, and encode path separators inside individual identity components.
   Keep this operational journal outside Git, any temporary worktree and durable agent memory.
   Read earlier passes before planning; reconcile them against the current diff and commits.
   If the journal is missing or damaged, recover only verifiable history from the forge and report
   any gap; never invent prior corrections. One repair session owns a PR journal at a time.

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

   Update the journal after each correction and validation: problem, final behavior, commits,
   failed-then-passing evidence, environment and limits. Preserve corrections to earlier corrections
   and mark reverted or superseded work. Write a sibling temporary file, then rename it over the
   journal; a failed write pauses the repair before another push or publication.

5. **Run a barrier sized to the delta.** Take the merge-blocking commands from the CI configuration,
   never from a same-named package script, then run the tier the corrections actually reach on the
   repair worktree:

   | Delta                                                 | Barrier                                                                            |
   | ----------------------------------------------------- | ---------------------------------------------------------------------------------- |
   | Comments, documentation or PR text only               | format, spellcheck, lint, typecheck                                                |
   | Code that changes no observable behavior              | the above, plus the unit tests of every touched package                            |
   | Behavior, a seam, a contract, a schema or a migration | the full merge-blocking barrier, failure-path tests and end-to-end suites included |

   A tier is a floor, not a ceiling: run the full barrier whenever the delta's reach is uncertain,
   and never let a tier excuse a gate the corrections do reach. Name the tier you ran in the report,
   so a reader sees which gates were skipped and why. Record counts and evidence gaps as
   `pr-verdict` requires. Keep independent mechanisms in separate commits and coupled corrections
   together.

6. **Re-anchor, then land the whole slate in one push.** Query the forge again and validate that the
   source repository, source ref and head SHA are all present and still identify the anchored head.
   If the head moved, do not push: anchor the new head, inspect the overlap, and reapply only
   corrections that remain valid. Push every correction of the slate together, normally, to the
   resolved source ref; never use a force option. A rejected push or missing permission leaves the
   commits local and becomes an explicit delivery blocker. One repair produces one judged head:
   every extra push discards a verdict already delegated and buys another review pass.

   Record prepared commits before pushing and mark them pushed only after verifying the remote
   result. On interruption or an ambiguous push result, reconcile remote commits before resuming.

7. **Judge the pushed head independently.** Resolve the SHA now shown by the PR and delegate a full
   `pr-verdict` review of that exact head to a fresh context, including its barrier. Do not delegate
   while a correction is still pending — a head you intend to amend is a head whose verdict you are
   about to throw away. When that review does find a defect in the repair itself, correct it, push
   once, and scope the second delegation to the new delta and its barrier tier instead of repeating
   the whole sweep. Record the verdict and its SHA in the journal; an older verdict is historical.
   All delegated passes stop after phase 5 without publishing comments or opening tickets.
   Return only the final head's verdict as current. Verdict publication remains subject to separate
   user authorization; repair authority alone publishes only the factual summary in step 8.

8. **Publish once approved.** Re-read the PR head and required remote checks. Publish only when
   the independent verdict is exactly `approved`, the required remote CI has succeeded on that
   same SHA, and no correction remains pending. `approved with reservations`, failed or pending CI,
   unavailable evidence, or a moved head keeps the journal pending with no intermediate comment.
   When no remote check is required, record that fact explicitly instead of claiming CI passed.
   A merged PR still requires evidence for the reviewed source head; merge alone is no substitute.

   Build `assets/repair-record.md` from all journal passes in the PR's language: one opening sentence
   with the final SHA, one short bullet per corrected problem, then validation counts and limits.
   Consolidate repeated fixes by their final outcome and omit superseded attempts; retain their
   history and detailed mechanisms in the journal. Block publication if missing history prevents
   an accurate cumulative summary. If nothing was corrected, no repair comment is needed.

   Search all existing comments for the stable marker `<!-- pr-fix:<pr> -->`. Update the matching
   comment only when owned by the publishing account; otherwise keep publication pending and report
   the ownership conflict. Create one only when none exists. For legacy `<pr>:<sha>` markers, reuse
   the latest repair comment owned by the publishing account and replace its marker, preserving other comments
   and replies. Pass the body through a file using `pr-verdict/references/forges.md` and save the
   returned comment ID, URL and published SHA in the journal. After a timeout or uncertain response,
   re-read remote comments before retrying; if lookup fails, keep publication pending.

9. **Close or pause the repair.** Report the final SHA, verdict, journal path and either the summary
   URL or the reason publication is pending. Keep the journal across sessions and after publication;
   a later correction resumes it and updates the same comment only after renewed approval and CI.
   Do not create a fix ticket for a defect corrected by this run. Remove the temporary worktree only
   after its commits are pushed and it is clean; retain the journal even when removing the worktree.

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
- **Repair publication and verdict publication are conflated** — either a team-visible merge
  decision appears without consent or each pass solicits premature replies. Keep intermediate
  results in the journal and publish the cumulative factual summary after final approval and CI;
  keep verdict publication subject to separate consent.
- **The record follows commits instead of mechanisms** — commits expose chronology but omit why a
  correction works, while one problem may span several commits. Keep mechanisms and proofs in the
  journal and summarize the final behavior once per corrected problem in the public comment.
- **The journal lives in a temporary worktree** — cleanup loses earlier passes. Use the stable
  PR path outside the checkout and reload it whenever another session resumes the repair.
- **A timeout is treated as a failed publication** — a blind retry duplicates an accepted comment.
  Look up the stable PR marker before retrying and preserve the journal when lookup is unavailable.
- **A failed push worktree is discarded** — the only copy of useful commits becomes hard to recover.
  Preserve the worktree and report the commit SHAs when the branch cannot be updated.
- **The head is amended after its verdict was delegated** — the fresh context spends its whole pass
  judging a SHA nobody will merge, and the repair turns into a fix-review-fix loop that finds
  something new every round. Freeze the slate, push once, then delegate.
- **The full barrier is re-run for a reworded comment** — end-to-end suites cost minutes and prove
  nothing about a doc change, so each round lengthens the feedback loop without adding evidence.
  Size the barrier to the delta and name the tier.

## Constraints

- Never mutate a PR unless the user explicitly asked to correct it or invoked `pr-fix`.
- Never force-push, overwrite a moved head or guess the PR source repository or ref.
- Never publish a verdict, create an issue or open another PR from repair authority alone; the
  mandatory factual repair record is not a verdict and requires no confirmation.
- Never publish a repair summary before independent `approved` and successful required remote CI
  on the current head; pause with the journal intact when that condition is unmet.
- Never create a new repair comment merely because the SHA changed; use the stable PR marker,
  preserve existing replies, and always pass the body through a file.
- Keep every correction, its mechanism, evidence and reasoned omission in the journal; publish a
  concise cumulative account of final outcomes with numeric validation evidence and its limits.
- Never claim a defect fixed before its failure-path test and the barrier tier its delta reaches
  both pass, and never report a gate you did not run.
- Never let the context that wrote the repaired head perform its final failure-class sweep.
- Never delegate a verdict on a head you still intend to amend; one repair lands one judged head.
- Never repair subjective preferences or broaden the change beyond proven findings.
- Never remove a temporary worktree that contains commits not yet pushed or uncommitted changes.

## References

- [assets/repair-journal.md](assets/repair-journal.md) — local cumulative state, loaded in step 1.
- [assets/repair-record.md](assets/repair-record.md) — concise public summary, filled in step 8.
