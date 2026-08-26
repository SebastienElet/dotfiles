---
name: linear-sync
description: >
  Reconcile assigned Linear issues with Bitbucket pull-request and checklist evidence. Use when the
  user asks to synchronize lifecycle state, check proven items, or repair missing pull-request
  links. Make sure to use this skill whenever merged work should close an issue or its checkboxes
  must reflect evidence.
compatibility: Requires Git, bkt, and an authenticated Linear transport with read and write access.
metadata:
  category: dev
---

# Synchronize Linear with Bitbucket

## Overview

Synchronize lifecycle facts and explicitly requested checklist evidence for the current user's
Linear issues from authoritative Linear, Bitbucket, and supplied verification state. This is a
manual reconciliation workflow, not a code review, implementation audit, or issue-shaping pass.

## Usage

`/linear-sync`

Run from the Bitbucket repository whose assigned Linear issues should be reconciled. The current
repository scopes branch-based discovery; an attached canonical pull-request URL may identify a
different repository explicitly. Report only applied transitions, repaired pull-request
attachments, unresolved inconsistencies, and relevant issues that were already synchronized.

## Workflow

1. **Load shared policy and adapters.** Activate `linear-workflow`, including its shared Linear and
   Bitbucket adapters, and apply all of its constraints. Use those transports rather than creating
   another integration. Verify current command schemas and authentication before reads or writes,
   and stop before any operation that the available transports do not cover.

2. **Establish identity and repository scope.** Retrieve the current Linear identity. Resolve the
   current Bitbucket repository from Git remotes and verify its workspace and repository slug with
   `bkt`; never trust the active `bkt` context alone. Consider only Linear issues assigned to that
   identity as synchronization candidates. Treat the resolved repository as the only scope for
   branch-based discovery; when a canonical attachment identifies another repository, verify that
   repository independently before reading its pull request.

3. **Read structured Linear facts.** Exhaust result pages while retrieving assigned candidates with
   their identifiers, workflow states, parents, direct sub-issues, blocking relations, links or
   attachments. Read the description when checklist reconciliation was requested or before a
   transition to `Done`. Read only the assignee and workflow state of an unassigned direct
   sub-issue when needed to decide whether an assigned parent is complete; never mutate or report
   that sub-issue as a candidate.

4. **Reconcile evidence conditionally.** When the user asks to update checkboxes, or an issue being
   completed contains a task list or a section that claims to track completion evidence, read and
   apply [checkbox evidence](references/checkbox-evidence.md). Each checkbox has its own evidence
   oracle. Reconcile it independently from workflow state, and do not inspect implementation or
   run a new functional review to manufacture missing proof.

5. **Resolve exact issue-to-work associations.** Prefer a canonical Bitbucket pull-request URL in
   Linear's structured links or attachments. Otherwise accept an exact issue identifier at the
   start of a source branch shaped `<ISSUE-ID>-<slug>`, after verifying the branch or pull request
   belongs to the resolved repository. Accept another signal only when structured metadata
   explicitly identifies both the issue and pull request. Title similarity may locate candidates
   for reporting, but never proves an association or authorizes a write. Record zero, one, and
   multiple exact matches as distinct outcomes.

6. **Repair certain attachments.** When an exact branch or explicit metadata proves a single
   issue-to-pull-request association and Linear lacks its canonical URL, attach that URL once.
   Independently read the issue back and verify the stored link. A failed attachment is a partial
   synchronization failure; preserve the Bitbucket state and retry only the idempotent Linear link
   operation.

7. **Complete assigned leaf issues from merged work.** For each assigned leaf with a certain linked
   Bitbucket pull request, read its state from Bitbucket. A merged state proves only the lifecycle
   transition: move the issue to `Done` and verify the state by an independent Linear read even when
   evidence checkboxes legitimately remain open. Preserve and name every unchecked residue in the
   reconciliation result. Do not inspect the diff, rerun tests, invoke `pr-verdict`, or perform a
   functional review. Several assigned leaves may transition from the same merged pull request only
   when each association is explicit and certain.

8. **Evaluate assigned parents from child state.** After leaf transitions are verified, process
   assigned parents bottom-up and re-read each parent with every direct sub-issue. A parent whose
   direct sub-issues are all `Done` is eligible for completion. Transition it automatically only
   when the selected Linear transport documents one atomic conditional mutation that writes `Done`
   if every direct sub-issue remains `Done` at commit time. A separate read followed by an ordinary
   state update does not qualify. The shared connector, CLI, and GraphQL adapters currently expose
   no such guarantee, so report the eligible parent and leave it unchanged for human decision. Do
   not infer completion from the parent description, pull-request title, or partial child set, and
   do not review the work again.

9. **Surface inconsistent active work.** For an assigned `In Progress` issue explicitly associated
   with the resolved repository, search exact attachments, exact issue-prefixed pull-request source
   branches, and exact issue-prefixed remote branches there. When none identifies Bitbucket work,
   report the observed repository, branch, pull-request, attachment, and blocking-relation facts and
   leave the state unchanged so the user can choose resumption, `Todo`, or `Done`. A missing match in
   the current repository says nothing about an issue with no explicit repository association;
   omit it unless conflicting explicit signals make it a relevant ambiguity. When evidence is
   ambiguous, report every conflicting fact and make no lifecycle or attachment write.

10. **Return a concise reconciliation.** Include only transitions applied, checklist edits with
    their evidence, unchecked or untracked evidence residue, attachments repaired, inconsistencies
    or ambiguities requiring a human decision, and relevant issues already in the state proven by
    the same oracle. Name partial failures. End with an automation assessment based on observed
    repeated usage; absent evidence of a stable reliable trigger, state that automation is not yet
    justified.

## Gotchas

- **Reviewing a merged pull request again** — synchronization becomes a third review and consumes
  context without improving the lifecycle oracle; trust Bitbucket's merged state and update Linear.
- **Matching titles across systems** — similar wording can close or link the wrong issue; use titles
  only to surface a candidate and require an explicit attachment, branch identifier, or metadata.
- **Closing a parent from assigned children only** — an unassigned open child is missed and the
  parent closes early; inspect every direct child's structured state while mutating only assigned
  issues.
- **Treating a post-write read as atomicity** — a child can change between the decision and the
  parent update; require a documented conditional mutation or leave the parent unchanged.
- **Treating a missing attachment as missing work** — an exact issue-prefixed branch can prove an
  existing pull request; query Bitbucket before reporting an `In Progress` inconsistency.
- **Searching only the current repository for a global issue** — unrelated work appears missing;
  require explicit repository scope before reporting the inconsistency.
- **Retrying the whole synchronization after a partial write** — verified transitions or links are
  repeated unnecessarily; re-read both systems and retry only the failed idempotent operation.
- **Cleaning the checklist when closing the issue** — unchecked claims become false assertions;
  keep workflow state and checkbox evidence independent and report the remaining items.

## Constraints

- Never mutate or report as a candidate an issue not assigned to the current Linear identity.
- Never inspect implementation, review a diff, rerun functional checks, or invoke a review skill.
- Never check an item from issue status, pull-request state, a generic green pipeline, or a desire
  to make the issue look complete; require item-specific evidence.
- Never rewrite a complete Linear description to change checklist markers when a verified targeted
  patch transport is available.
- Never transition an issue from title similarity, prose, an ambiguous link, or an unverified
  repository context.
- Never auto-transition a parent through a separate read followed by an unconditional state update.
- Never move an inconsistent `In Progress` issue automatically to `Todo` or `Done` without merged
  pull-request proof or complete child-state proof.
- Never add a hook, cron job, scheduled task, or implicit trigger; v0 remains manually invoked.
- Never claim future automation is justified without repeated-run evidence of a stable trigger and
  acceptable ambiguity rate, and never implement it from this workflow.
