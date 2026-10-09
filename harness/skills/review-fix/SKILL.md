---
name: review-fix
description: >
  Repair local code-review findings with Codex before a PR. Use when the user asks to review and
  fix local changes or correct local findings. Make sure to use this skill whenever local review
  corrections are authorized, even if unnamed. Excludes review-only requests and open PRs,
  whose repairs belong to pr-fix.
compatibility: Requires Git, code-review, the repository checks, and fresh Codex reviewers. Claude Code additionally requires the official codex@openai-codex plugin.
metadata:
  category: dev
---

# Review Fix

## Overview

Turn local findings into bounded corrections by Codex and an independent review of the final
candidate. Preserve the author's work and keep every disposition tied to the code it concerns.
This is an agent workflow, not a host-enforced gate or a merge verdict.

## Usage

`/review-fix <local scope or findings>` in Claude Code, or `$review-fix <local scope or findings>`
in Codex. Example: `Review and fix the current local changes before opening a PR.`

A direct invocation or an explicit request to repair authorizes local edits and relevant checks.
A request only to review authorizes no correction. Commits, pushes and PR publication require
authorization from the surrounding task; this skill never approves or merges. An open PR routes
to `pr-fix`. Do not compose `pr-verdict` for local work.

## Steps

1. **Anchor the candidate.** Record the absolute worktree, HEAD, requested comparison and included
   files. Inspect staged, unstaged and relevant untracked changes with `code-review`; resolve an
   explicit base rather than assuming `main`. Preserve the exact scoped diff and untracked file
   contents outside the candidate. Record their hashes alongside HEAD: a SHA alone cannot identify
   local edits. Keep unrelated changes outside the repair slate.
2. **Establish one writer and the runtime.** Suspend the author's writes until the Codex correction
   ends. In Claude Code, read [references/claude-code.md](references/claude-code.md) and verify the
   plugin before delegating. In Codex, repair directly and use fresh Codex subagents for reviews.
   Do not invoke Claude from Codex. An unavailable runtime or independent reviewer leaves the
   workflow incomplete; preserve existing work and report the missing capability.
3. **Review before accepting findings.** Obtain a fresh, read-only `code-review` of the anchored
   candidate, with requirements, applicable ADRs and raw check evidence. Withhold the author's
   verdict and supplied findings until the reviewer records its first analysis, then reconcile
   them. For each finding, record the location, failure mechanism, violated requirement and relevant
   check in a scratchpad outside Git. Mark it `confirmed`, `refuted` with evidence, or
   `decision-required`. Style preferences do not enter the repair slate. Missing requirements or
   unresolved architecture, migration or product policy stop only the dependent correction.
4. **Correct one slate with Codex.** Present the confirmed scope, then proceed under the repair
   authorization. Give Codex the candidate identity, confirmed findings, requirements and allowed
   files; require it to verify the candidate before editing and preserve unrelated work. Use TDD
   for owned behavior: observe the relevant reproduction fail, fix the cause, then observe it pass.
   For declarative changes, use the native or existing relevant validation. A failed delegation is
   not permission for Claude to implement a substitute correction. Record changed files, evidence,
   environment and remaining findings before another pass.
5. **Validate and review the stable result.** Inspect Codex's actual delta against the slate,
   index only intended new files before checks, and run the repository barrier that delta reaches.
   After all planned edits finish, obtain a read-only review in a new Codex context that did not
   write the corrections. Supply the final candidate, requirements and raw evidence before earlier
   conclusions; reconcile the finding ledger after its first analysis. Review the repair delta
   and affected consumers. Changed verification mechanisms require `proof-integrity-review` over
   the complete affected change. Compare candidate hashes before and after review; concurrent
   edits invalidate the affected review and require a new anchor.
6. **Bound the loop and deliver.** Allow at most two correction passes, counted across continuation
   of this repair. Each pass ends with validation and an independent review. After the second,
   stop with unresolved findings or disagreements instead of restarting the counter or expanding
   scope. On success, report the final candidate identity, corrections, refutations, checks and
   limits. Say that no demonstrated defect remains in the reviewed scope; never equate that with
   merge approval. On timeout, failed checks, empty or malformed reviewer output, missing essential
   evidence or unavailable independence, report `incomplete` and preserve the scratchpad and work.

## Gotchas

- **Using the repair thread as reviewer** — its conclusions share the correction's assumptions;
  start a new context and withhold prior verdicts until its independent first analysis.
- **Both agents write at once** — the reviewed diff becomes stale or edits collide; suspend the
  author, verify the candidate identity, and review only after the writer has finished.
- **A finding becomes cleanup** — subjective preferences enlarge the change; repair only confirmed
  mechanisms and keep unresolved decisions explicit.
- **Reusing an old SHA for local edits** — HEAD may stay unchanged through every pass; bind the diff
  and untracked contents too, then invalidate reviews when that candidate changes.
- **Turning a review request into repair authority** — the user did not authorize mutation; return
  findings through `code-review` until a repair is requested.

## Constraints

- Never mutate without repair authority, broaden the confirmed slate, or overwrite unrelated work.
- Never let the context that wrote a correction perform its final independent review.
- Never exceed two correction passes or reset the count on continuation.
- Never treat failed, missing, stale or empty reviewer output as acceptance.
- Never approve, merge, publish a verdict, or activate `pr-verdict` from this local workflow.
- Never install missing runtimes during a repair or enable the companion's automatic Stop gate.
