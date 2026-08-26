---
name: linear-sync
description: >
  Reconcile assigned Linear issues with Bitbucket pull-request reality without reviewing code. Use
  when the user manually asks to synchronize Linear lifecycle state or repair missing pull-request
  links. Make sure to use this skill whenever merged Bitbucket work should advance Linear issues.
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

Run from the Bitbucket repository whose assigned Linear issues should be reconciled. The current
repository scopes branch-based discovery; an attached canonical pull-request URL may identify a
different repository explicitly. Report only applied transitions, repaired pull-request
attachments, the queue awaiting verification, unresolved inconsistencies, and relevant issues that
were already synchronized.

## Workflow

1. **Load shared policy and adapters.** Activate `linear-workflow`, including its shared Linear and
   Bitbucket adapters and its `references/completion-evidence.md` rule, and apply all of its
   constraints. Use those transports rather than creating another integration. Verify current
   command schemas and authentication before reads or writes, and stop before any operation that
   the available transports do not cover.

2. **Establish identity and repository scope.** Retrieve the current Linear identity. Resolve the
   current Bitbucket repository from Git remotes and verify its workspace and repository slug with
   `bkt`; never trust the active `bkt` context alone. Consider only Linear issues assigned to that
   identity as synchronization candidates. Treat the resolved repository as the only scope for
   branch-based discovery; when a canonical attachment identifies another repository, verify that
   repository independently before reading its pull request.

3. **Read structured Linear facts.** Exhaust result pages while retrieving assigned candidates with
   their identifiers, workflow states, parents, direct sub-issues, blocking relations, and links or
   attachments, plus the description of any candidate a merged pull request could complete. Read
   only the assignee and workflow state of an unassigned direct sub-issue when needed to decide
   whether an assigned parent is complete; never mutate or report that sub-issue as a candidate.

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

6. **Route assigned leaf issues by their verification boxes.** For each assigned leaf with a certain
   linked Bitbucket pull request, read its state from Bitbucket. A merged state proves the code
   landed, not that the issue's verification boxes hold. Read those boxes without ever writing one:
   all checked, or no evidence section at all, moves the issue to `Done`; at least one unchecked
   line, or an evidence section that holds no state, moves it to its team's review state instead,
   with the unproven lines named as residue. Guard the state write with the classification it rests
   on, following the anchored guard in `completion-evidence.md`: a box that moved must abort the
   save before any state is written, because a post-write read confirms the state and never that the
   boxes still say what they said. Where a transport cannot carry the guard and the state in one
   save, re-read the description immediately before the write, leave the issue untouched when it
   changed, and report the window between that read and the write as unguarded. Verify either
   transition by an independent Linear read. Do not inspect the diff, rerun tests, invoke
   `pr-verdict`, or perform a functional review; this workflow routes work by the evidence that
   already exists and never produces any. Several assigned leaves may transition from the same
   merged pull request only when each association is explicit and certain.

7. **Evaluate assigned parents from child state.** After leaf transitions are verified, process
   assigned parents bottom-up and re-read each parent with every direct sub-issue. A parent whose
   direct sub-issues are all `Done` is eligible for completion; a sub-issue waiting in the review
   state is not `Done` and makes the parent ineligible. Transition it automatically only when the
   selected Linear transport documents one atomic conditional mutation that writes `Done` if every
   direct sub-issue remains `Done` at commit time. A separate read followed by an ordinary
   state update does not qualify. The shared connector, CLI, and GraphQL adapters currently expose
   no such guarantee, so report the eligible parent and leave it unchanged for human decision. Do
   not infer completion from the parent description, pull-request title, or partial child set, and
   do not review the work again.

8. **Surface inconsistent active work.** For an assigned `In Progress` issue explicitly associated
   with the resolved repository, search exact attachments, exact issue-prefixed pull-request source
   branches, and exact issue-prefixed remote branches there. When none identifies Bitbucket work,
   report the observed repository, branch, pull-request, attachment, and blocking-relation facts and
   leave the state unchanged so the user can choose resumption, `Todo`, or `Done`. A missing match in
   the current repository says nothing about an issue with no explicit repository association;
   omit it unless conflicting explicit signals make it a relevant ambiguity. When evidence is
   ambiguous, report every conflicting fact and make no lifecycle or attachment write.

9. **Return a concise reconciliation.** Include only transitions applied, attachments repaired, the
   queue now awaiting verification with the unchecked line behind each issue, inconsistencies or
   ambiguities requiring a human decision, and relevant issues already in the state proven by the
   same oracle. Name partial failures. End with an automation assessment based on observed
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
- **Aligning description checkboxes with the new state** — a merged pull request becomes false proof
  for a line no run covered; never write a box here, and report the unchecked lines instead.
- **Closing a merged issue without reading its verification boxes** — the check nobody ran is buried
  under `Done`; route the issue to its team's review state and leave it in the verification queue.

## Constraints

- Never mutate or report as a candidate an issue not assigned to the current Linear identity.
- Never inspect implementation, review a diff, rerun functional checks, or invoke a review skill.
- Never check, uncheck, or reword a description checkbox from this workflow; it produces no
  evidence, so it reports unproven residue instead of resolving it.
- Never move an issue to `Done` from a merge while one of its verification boxes is unchecked; its
  team's review state is the destination and the unchecked lines are reported.
- Never transition an issue from title similarity, prose, an ambiguous link, or an unverified
  repository context.
- Never auto-transition a parent through a separate read followed by an unconditional state update.
- Never move an inconsistent `In Progress` issue automatically to `Todo` or `Done` without merged
  pull-request proof or complete child-state proof.
- Never add a hook, cron job, scheduled task, or implicit trigger; v0 remains manually invoked.
- Never claim future automation is justified without repeated-run evidence of a stable trigger and
  acceptable ambiguity rate, and never implement it from this workflow.
