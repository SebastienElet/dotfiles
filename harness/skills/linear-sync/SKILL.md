---
name: linear-sync
description: >
  Reconcile assigned Linear issues with Bitbucket pull-request reality without reviewing code. Use
  when the user manually asks to synchronize Linear lifecycle state or repair missing pull-request
  links. Make sure to use this skill whenever merged Bitbucket work should close Linear issues.
compatibility: Requires Git, bkt, and an authenticated Linear transport with read and write access.
metadata:
  category: dev
---

# Synchronize Linear with Bitbucket

## Overview

Synchronize lifecycle facts for the current user's Linear issues from authoritative Linear and
Bitbucket state. This is a manual reconciliation workflow, not a code review, implementation audit,
or issue-shaping pass.

## Usage

`/linear-sync`

Run from the Bitbucket repository whose assigned Linear issues should be reconciled. Report only
applied transitions, repaired pull-request attachments, unresolved inconsistencies, and relevant
issues that were already synchronized.

## Workflow

1. **Load shared policy and adapters.** Activate `linear-workflow`, including its shared Linear and
   Bitbucket adapters, and apply all of its constraints. Use those transports rather than creating
   another integration. Verify current command schemas and authentication before reads or writes,
   and stop before any operation that the available transports do not cover.

2. **Establish identity and repository scope.** Retrieve the current Linear identity. Resolve the
   current Bitbucket repository from Git remotes and verify its workspace and repository slug with
   `bkt`; never trust the active `bkt` context alone. Consider only Linear issues assigned to that
   identity as synchronization candidates.

3. **Read structured Linear facts.** Exhaust result pages while retrieving assigned candidates with
   their identifiers, workflow states, parents, direct sub-issues, blocking relations, and links or
   attachments. Read only the assignee and workflow state of an unassigned direct sub-issue when
   needed to decide whether an assigned parent is complete; never mutate or report that sub-issue
   as a candidate.

4. **Resolve exact issue-to-work associations.** Prefer a canonical Bitbucket pull-request URL in
   Linear's structured links or attachments. Otherwise accept an exact issue identifier at the
   start of a source branch shaped `<ISSUE-ID>-<slug>`, after verifying the branch or pull request
   belongs to the resolved repository. Accept another signal only when structured metadata
   explicitly identifies both the issue and pull request. Title similarity may locate candidates
   for reporting, but never proves an association or authorizes a write. Record zero, one, and
   multiple exact matches as distinct outcomes.

5. **Repair certain attachments.** When an exact branch or explicit metadata proves a single
   issue-to-pull-request association and Linear lacks its canonical URL, attach that URL once.
   Independently read the issue back and verify the stored link. A failed attachment is a partial
   synchronization failure; preserve the Bitbucket state and retry only the idempotent Linear link
   operation.

6. **Complete assigned leaf issues from merged work.** For each assigned leaf with a certain linked
   Bitbucket pull request, read its state from Bitbucket. A merged state is sufficient completion
   proof: move the issue to `Done` and verify the state by an independent Linear read. Do not inspect
   the diff, rerun tests, invoke `pr-verdict`, or perform a functional review. Several assigned
   leaves may transition from the same merged pull request only when each association is explicit
   and certain.

7. **Complete assigned parents from child state.** After leaf transitions are verified, re-read
   each assigned parent with direct sub-issues. Move it to `Done` only when every direct sub-issue is
   already `Done`, then verify the parent state independently. Do not infer completion from the
   parent description, pull-request title, or partial child set, and do not review the work again.

8. **Surface inconsistent active work.** For an assigned `In Progress` issue, search exact
   attachments, exact issue-prefixed pull-request source branches, and exact issue-prefixed remote
   branches in the resolved repository. When none identifies Bitbucket work, report the observed
   branch, pull-request, attachment, and blocking-relation facts and leave the state unchanged so
   the user can choose resumption, `Todo`, or `Done`. When evidence is ambiguous, report every
   conflicting fact and make no lifecycle or attachment write.

9. **Return a concise reconciliation.** Include only transitions applied, attachments repaired,
   inconsistencies or ambiguities requiring a human decision, and relevant issues already in the
   state proven by the same oracle. Name partial failures. End with an automation assessment based
   on observed repeated usage; absent evidence of a stable reliable trigger, state that automation
   is not yet justified.

## Gotchas

- **Reviewing a merged pull request again** — synchronization becomes a third review and consumes
  context without improving the lifecycle oracle; trust Bitbucket's merged state and update Linear.
- **Matching titles across systems** — similar wording can close or link the wrong issue; use titles
  only to surface a candidate and require an explicit attachment, branch identifier, or metadata.
- **Closing a parent from assigned children only** — an unassigned open child is missed and the
  parent closes early; inspect every direct child's structured state while mutating only assigned
  issues.
- **Treating a missing attachment as missing work** — an exact issue-prefixed branch can prove an
  existing pull request; query Bitbucket before reporting an `In Progress` inconsistency.
- **Retrying the whole synchronization after a partial write** — verified transitions or links are
  repeated unnecessarily; re-read both systems and retry only the failed idempotent operation.

## Constraints

- Never mutate or report as a candidate an issue not assigned to the current Linear identity.
- Never inspect implementation, review a diff, rerun functional checks, or invoke a review skill.
- Never transition an issue from title similarity, prose, an ambiguous link, or an unverified
  repository context.
- Never move an inconsistent `In Progress` issue automatically to `Todo` or `Done` without merged
  pull-request proof or complete child-state proof.
- Never add a hook, cron job, scheduled task, or implicit trigger; v0 remains manually invoked.
- Never claim future automation is justified without repeated-run evidence of a stable trigger and
  acceptable ambiguity rate, and never implement it from this workflow.
