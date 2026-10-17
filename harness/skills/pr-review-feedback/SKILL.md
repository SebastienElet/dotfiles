---
name: pr-review-feedback
description: >
  Address review feedback on a pull request you authored. Use when a reviewer or bot comments on
  your open PR and the user asks you to handle it, even if they only say feedback arrived. For
  another contributor's PR repair use pr-fix; for local pre-PR findings use review-fix.
compatibility: Requires Git, authenticated gh or bkt, jq for GitHub replies, write access to the PR source branch, and the repository validation toolchain.
metadata:
  category: dev
---

# PR Review Feedback

## Overview

Handle feedback on an open PR authored by the authenticated account. Verify each finding, make
bounded corrections on the existing PR branch, and answer in the discussion that raised it. This
workflow owns no merge verdict, native approval, or merge decision. A general PR comment never
replaces a reply in an available review thread.

## Usage

`/pr-review-feedback <pr-number|pr-url>` or `$pr-review-feedback <pr-number|pr-url>`.
An ongoing authoring task can invoke the skill when the user says that review feedback arrived;
the skill identifies the PR from the current branch or provided URL. A status-only or review-only
request permits reading and reporting, not correction or publication. For another contributor's
PR, route an authorized repair request to `pr-fix`; for uncommitted local review findings, use
`review-fix`.

## Steps

1. **Anchor the PR and feedback.** Resolve the forge, canonical PR URL, destination repository,
   base, source repository, source ref, head SHA, author and authenticated account. Require an open PR authored by that
   account before entering this workflow. Read the PR description, all reviews, timeline comments,
   inline or PR-level review threads, replies and tasks; follow pagination. Read
   `references/threads.md` for the forge-specific thread identity and resolution operations. Use the
   destination repository for PR discussions and the source repository only for the branch push.
   Record each finding's URL, original author, thread or comment ID, current resolution state and
   reviewed SHA. A standalone timeline comment or review body has no thread to resolve.
2. **Qualify the feedback.** For each finding, trace the affected behavior through code, tests,
   applicable ADRs and the PR contract. Mark it `confirmed`, `refuted` with evidence, or
   `decision-required`. Name the failure mechanism and a lift check for every correction. Keep
   unrelated remarks and new feature requests out of the repair slate. Resolve material policy or
   architecture decisions before dependent edits; continue independent corrections.
3. **Correct on the existing PR branch.** Verify the worktree and head against the forge. Preserve
   unrelated changes; use a clean temporary worktree at the recorded head if the caller's worktree
   is not safe for the repair. Reproduce confirmed defects before changing owned behavior, then
   use the smallest relevant test or native validation. Correct only the confirmed slate, including
   affected failure paths. Run a local `code-review` on the finished delta and fix demonstrated
   findings before delivery; this is a local review, not `pr-verdict`.
4. **Validate and push.** Run the repository checks that cover the changed behavior, recording the
   environment and unsupported targets. Index intended new files before checks. Re-read the PR
   source repository, ref and head SHA; if it moved, re-anchor and inspect overlap before pushing.
   Commit and push normally to that exact source ref, never with force. Verify the PR now names
   the pushed SHA. Wait for relevant remote CI on that SHA; repair applicable failures before
   claiming the feedback resolved. If no remote check is required, record that fact.
5. **Answer where the feedback lives.** For each corrected thread, re-read its complete discussion
   and the current head, then reply with the final correction and evidence in that thread. Read the
   reply back, resolve the thread when permitted, and verify the resolved state using
   `references/threads.md`. A failed or ambiguous write requires a read by stable marker before
   retrying. If the reviewer added a new objection or reopened the thread, return it to the slate;
   do not close it automatically. For a refuted finding, answer with the evidence in its thread
   but leave resolution to the reviewer. A standalone timeline comment can only receive a linked
   PR-level response, using the same read-before-retry discipline; report that it has no resolvable
   thread. Apply the same fallback to a finding only in a review body, linking that review's URL.
6. **Report the actual state.** Re-read the head and threads after the last write. Report the PR
   URL, final SHA, each finding's disposition, reply URL and verified thread state, checks and
   limits. Do not publish a second root-level repair summary for threaded findings. Do not approve,
   request changes, merge, or add the PR to a merge queue from this workflow.

## Gotchas

- **Treating a bot finding as authority** — a suggested fix may be wrong or outside the PR scope;
  reproduce its mechanism before editing and explain a refutation in the original thread.
- **Replying at the PR root** — the original discussion stays unanswered and unresolved; use the
  thread's parent ID, read the reply back, then verify resolution.
- **Retrying an uncertain reply** — the first write may have succeeded; search the thread for the
  stable marker and publishing account before another write.
- **Closing a reopened thread** — a new reviewer objection can arrive without a new commit; read
  the discussion again before resolution and before reporting completion.

## Constraints

- Never mutate a PR whose author is not the authenticated account or exceed the authorized repair
  scope; route another contributor's PR repair to `pr-fix`.
- Never equate a root PR comment, a successful API response, or a green aggregate CI badge with a
  verified reply and resolved review thread.
- Never duplicate a reply after an ambiguous result, resolve an uncorrected or contested thread,
  force-push, or overwrite a moved head.
- Never invoke `pr-verdict`, publish a merge verdict, approve, or merge from feedback authority.
- State an unavailable forge capability or missing execution evidence instead of claiming closure.

## References

- [references/threads.md](references/threads.md) — inventory, reply and resolution operations for
  GitHub and Bitbucket.
