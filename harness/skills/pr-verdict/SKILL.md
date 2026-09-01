---
name: pr-verdict
description: >
  Deliver a PR verdict on an open pull request, yours or another author's. Use only when the user
  explicitly invokes `$pr-verdict` or `/pr-verdict`; never select it implicitly from a review,
  merge, approval, re-review, push, or pull-request-authoring request.
disable-model-invocation: true
compatibility: >
  Authenticated `gh` (GitHub) or `bkt` (Bitbucket) CLI, plus an issue tracker CLI (`linear`,
  `gh issue create`) when a blocking defect needs a fix ticket.
metadata:
  category: dev
---

# PR Verdict

## Overview

The output is a verdict that commits to a merge decision, not a list of remarks. Three properties
separate it from a default agent review: every blocking finding names a failure mechanism that
breaks an invariant the code claims to hold, every changed observable behavior has explicit
behavior-level evidence, and the limits of that evidence are declared inside the verdict. A green
barrier whose gaps stay implicit is the failure this skill exists to prevent — a verdict is only as
strong as what it admits it did not test.

Not for re-reading your own diff before committing: perform that review and verification directly.
This skill judges an open pull request and engages a decision the team will act on. That the PR is
your own changes nothing about the verdict, only about how it is enforced — see phase 6, since
GitHub refuses a blocking review on your own PR.

## Usage

Invoke `$pr-verdict <pr-number|pr-url>` in Codex or `/pr-verdict <pr-number|pr-url>` in Claude Code
and Cursor. The forge is detected from `git remote get-url origin`.

After explicit invocation, typical cases are: "is #1042 safe to merge?" (phase 1 finds the branch
stacked and the shown diff twice its real size), "review this PR before I approve it" (phase 3
turns a vague unease into a named mechanism, or drops it), "they pushed the fixes, re-review"
(phase 6 finds the previous marker, sees a new SHA, and publishes a second verdict rather than
editing the first).

Run the six phases in order; phase 4 precedes phase 5 because a verdict without an executed barrier
is an opinion. When `pr-fix` composes this skill before changing the head, stop after phase 5 and
return the unpublished verdict to that workflow: opening a ticket or publishing a verdict for a
head about to be repaired would create stale external state. This skill and its references are
English; the published verdict follows the language of the PR. Publishing (phase 6) is
outward-facing and visible to the team — ask for confirmation first, unless the request explicitly
says to post directly.

## Steps

1. **Anchor.** Resolve the PR number, head SHA, real destination branch and checks state with
   `references/forges.md`, then check out that exact head. A repository that has its own forge skill
   wins over those raw commands. If the PR is stacked on another unmerged PR, say so and make
   retargeting after the parent merges part of the verdict: the diff you are reading is not the diff
   that will land on the integration branch. A review not anchored on a named SHA is invalid, and
   the marker in phase 6 is what makes that anchoring visible to the next reader. When the request
   anticipates a PR that is not open yet, poll until it exists and anchor on it; reviewing the
   branch instead is a review of something the team has not proposed to merge.

2. **Understand before judging.** Read the PR description, the design documents it cites, and the
   whole diff. The attachments are part of the description: a screenshot or an uploaded log is
   evidence the author supplied, and the forge returns it as raw markup that a text pass slides
   over — download every one and look at it, in the comments too (`references/forges.md`).
   Restate the invariant the code claims to hold, in one sentence. Producing a blocking
   finding before the flow is traced end to end is forbidden: the mechanism is what makes a finding
   blocking, and you cannot name a mechanism you have not followed.

   Inventory every externally observable behavior added, removed or changed by the diff. Start a
   changed-behavior ledger with one row per behavior: the behavior, positive evidence on the exact
   head, a negative witness, and the result. A negative witness is an executed test-first RED or a
   controlled faulty variant derived from the exact head, shown to fail when the behavior is removed,
   reversed or broken. Restore and verify the exact head before running its positive barrier. A
   passing regression test without an observed failing counterpart is positive evidence only; a
   green aggregate barrier is evidence for no individual row by itself. Record missing evidence as
   `absent`, never as an inference.

3. **Sweep the failure classes.** Put all ten questions in `references/failure-classes.md` to the
   diff. Record, per class, one of: not applicable, holds because `<evidence>`, or broken by
   `<mechanism>`. Only the third form can become a blocker. When the head under review was written
   in this session, delegate this sweep to a fresh context, read-only and scoped to the diff: the
   context that produced the diff shares the blind spot that produced the defect, and records
   `holds` for the class it has just broken. Reconcile the sweep with the changed-behavior ledger:
   the sweep can add findings, but cannot replace a row or turn absent behavior-level evidence into
   `holds`.

4. **Run the barrier, then declare its holes.** Run lint, typecheck and tests the way the project
   runs them — including inside a container when the project requires it, since numbers from the
   wrong runner are not evidence for this head. Read the CI configuration to find the gate that
   actually blocks the merge before running anything: it is often not the package script of the same
   name. Report counts: builds, errors, warnings against the project's threshold, tests passed over
   tests run. Then enumerate what the barrier does not reach: sequential tests say nothing about a
   race, jsdom nothing about a browser, an in-memory database nothing about PostgreSQL, one platform
   nothing about the others. If nothing exercises the changed code, that absence is the review's
   first finding, not a reason to announce green. Then attribute each piece of evidence: measured
   here, supplied by the author and not reproduced here, or absent. The three are not
   interchangeable — "not observed" written over evidence sitting in the description is a false
   statement about the author's work, and a lift criterion asking for a run the PR already shows
   asks them to repeat themselves. What is missing in that case is a control in the repository:
   name the control, not the re-run. Complete the ledger from evidence actually reproduced during
   this review; evidence supplied by the author but not reproduced stays attributed and does not
   satisfy an approval row until reproduced.

5. **Return a verdict.** Exactly one of _changes required_, _approved with reservations_,
   _approved_. Write the phase-5 verdict in the order defined by `assets/verdict-template.md`, with
   the complete changed-behavior ledger before the blocking paragraph. Keep one row per inventoried
   behavior; a prose summary does not replace the rows. Both approval verdicts require every ledger
   row to contain reproduced positive evidence on the exact head and a reproduced negative witness
   traceable to its controlled faulty variant, with no contradictory result. Otherwise the verdict
   is _changes required_, and the missing evidence is the lift criterion. Each other blocking
   finding carries its named mechanism and its lift criterion — what must become true for the block
   to go away. Reservations are for mechanisms whose consequence is bounded; a mechanism that can
   lose or corrupt data blocks even when the author disagrees. A style, naming or structure
   preference never blocks: label it non-blocking, or drop it.

6. **Trace and publish.** When running inside the pre-repair pass of `pr-fix`, return the phase 5
   verdict without a ticket or publication and let that workflow continue. Otherwise, open or reuse
   a fix ticket for blocking defects, and link the initial verdict if one exists. Never open a
   ticket solely to request or record a re-review: the new
   head-specific verdict comment is that record. Write one general comment from
   `assets/verdict-template.md`, prefixed with the idempotency marker
   `<!-- pr-verdict:<pr>:<head-sha-12> -->` — not a rain of inline comments. Search the existing
   comments for that marker first: a verdict carrying the same `<pr>:<sha>` is updated in place,
   never duplicated. Re-read before publishing — past about thirty lines, non-blocking remarks are
   posing as blockers; never truncate the required behavior ledger to meet that heuristic. On
   GitHub, _changes required_ is published with
   `gh pr review --request-changes`, the native state, unless you authored the PR: GitHub refuses it
   there, and Bitbucket has no reliable equivalent at all. Whenever that native state is
   unavailable, the comment _is_ the verdict and its closing sentence carries the whole enforcement
   — say so in the verdict, so the reader knows nothing mechanical is holding the merge button.

## Gotchas

- **The base the forge reports is not the base to review** — and it errs in both directions. A PR
  that targets its parent branch hides what the parent still owes; a PR that targets the integration
  branch while sitting on an unmerged parent swells with the parent's work — measured on one stacked
  PR, the reported diff was more than twice the PR's own. Recompute the base from the parent's head
  and put retargeting in the lift criterion.
- **The comment body passed inline** — apostrophes and backticks in a verdict break shell quoting
  and silently truncate the comment. Always pass the body through a file (see
  `references/forges.md`).
- **Renaming the visible command marker** — existing comments become invisible to the duplicate
  guard. Keep `pr-verdict` as the stable external protocol key.
- **A stale marker updated instead of superseded** — the SHA in the marker going stale on the next
  push is the point. Same `<pr>:<sha>` → update that comment; different SHA → publish a new verdict
  and leave the old one as the record of what was judged.
- **A re-review ticket opened for traceability** — it duplicates the head-specific verdict and adds
  tracker noise without tracking a defect. Link the previous verdict, publish the new SHA marker,
  and reserve tickets for blocking defects.
- **"Approved with reservations" used to avoid a disagreement** — that state is a claim that the
  consequence is bounded. If you cannot state the bound, the verdict is _changes required_.
- **A missing feature reported as an omission** — an absent regulatory or business control is either
  a declared contract (what, why, lift condition) or a blocker. Silence is the defect, not the
  absence.
- **The author's evidence read as absent** — a screenshot of the output comes back from the forge
  as an `<img>` tag inside the body, and a text pass slides over it. The verdict then announces that
  nothing was observed and makes the merge conditional on steps the author has already run and
  attached, which reads as not having read the PR. Open every attachment in phase 2, and label
  supplied evidence not reproduced here as theirs.
- **Numbers copied from the PR's own pipeline** — a green pipeline is context for phase 1, never the
  barrier of phase 4. The barrier is what you ran, authenticated, on the head you checked out.
- **The package script mistaken for the CI gate** — the repository's `lint` script may walk the whole
  tree while the pipeline lints only the changed files. Its count then measures a backlog that
  predates the head under review, and reporting it as the barrier drowns the diff in noise. Take the
  command from the CI configuration, and name in the verdict which one you ran.

## Constraints

- Never approve without having executed the barrier on the exact head under review.
- Never write "everything is green": report counts, or report that nothing ran.
- Never report a count from a command the pipeline does not run; name the gate you executed.
- Never publish a blocking finding without a named failure mechanism and a lift criterion.
- Never block on style, naming or structure preference; label it non-blocking.
- Never open a review that is not anchored on a head SHA.
- Never issue either approval verdict unless every changed observable behavior has reproduced
  positive evidence on the exact head and a reproduced negative witness.
- Never infer behavior-level evidence from an aggregate green barrier.
- Never call a passing test a negative witness without an observed failure when the claimed behavior
  is broken.
- Never leave a limit of the evidence implicit; the barrier's gaps belong in the verdict text.
- Never report as absent, or demand in a lift criterion, evidence the PR already supplies.
- Never publish two verdicts for the same `<pr>:<sha>`; update the existing comment instead.
- Never sweep the failure classes on a head you wrote in this session from the context that wrote it.
- Never create a ticket solely to request, schedule or record a re-review.
- Never create a ticket or publish the pre-repair verdict when `pr-fix` composes phases 1 through 5.
- Never publish without confirmation, unless the request explicitly says to post directly.

## References

- [references/forges.md](references/forges.md) — forge detection and the GitHub/Bitbucket command
  parity table. Read in phase 1, before the first CLI call.
- [references/failure-classes.md](references/failure-classes.md) — the ten failure classes as
  questions to put to the diff. Read in phase 3.
- [assets/verdict-template.md](assets/verdict-template.md) — the verdict skeleton with its required
  slots. Filled in phase 6.
- [references/cases.md](references/cases.md) — three behavioral cases with their expected verdicts,
  forge coverage, and the record of what they have never validated.
