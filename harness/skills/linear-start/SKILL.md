---
name: linear-start
description: >
  Start or resume implementation of an assigned Linear issue in a Bitbucket repository. Use when
  the user or context actually asks to begin coding an issue. Make sure to use this skill whenever
  implementation starts from a Linear ID or attached pull request, even if Linear is not named.
compatibility: Requires Git, bkt, and an authenticated Linear transport with read and write access.
metadata:
  category: dev
---

# Start Linear Work

## Overview

Start or resume one executable Linear issue without duplicating work or coupling the workflow to an
agent provider. Compose `linear-workflow` for shared policy, select transports by verified
capabilities, and keep Linear synchronization limited to lifecycle facts.

## Usage

`/linear-start <Linear issue ID or URL>`

Examples: "Start ENG-482", "implement the assigned Linear issue", or "resume ENG-482 from its
Bitbucket pull request". Do not activate for status lookup, review, ticket drafting, or speculative
planning without an actual request to begin implementation.

## Workflow

1. **Load the shared policy.** Activate `linear-workflow`, including its shared Linear and
   Bitbucket adapters, and apply all of its constraints. Stop before any lifecycle write when no
   authenticated transport covers the required operation.

2. **Establish executable Linear state.** Retrieve the current Linear identity, then retrieve the
   issue's identifier, title, assignee, workflow state, parent, sub-issues, blocking relations, and
   links or attachments. Refuse when the issue is not assigned to that identity, has an active
   Linear blocking relation, or is a parent with an open sub-issue. Do not replace missing
   structured fields with conclusions drawn from the description.

3. **Keep shaping separate.** If the issue appears too broad or not implementation-ready, stop and
   route it to the future `issue-shaping` skill. Do not run product analysis or `linear-issue-spec`
   merely to start work or maintain Linear.

4. **Resolve the Bitbucket repository.** Inspect Git remotes and match the working repository to an
   explicit Bitbucket workspace and repository slug. Verify that identity with `bkt`; never inherit
   its active context without comparing it to the Git remote.

5. **Resume an attached pull request.** Find Bitbucket pull-request links in Linear's structured
   links or attachments and inspect each candidate with an explicit workspace, repository, and pull
   request ID. When one open pull request matches the repository, inspect its source branch and
   check out or reuse that exact branch. Never create a concurrent pull request. If candidates are
   ambiguous or the attached pull request is merged or declined, report the state and stop before
   creating another branch or pull request.

6. **Start new work only when ready.** If no pull request is attached, derive a lowercase kebab-case
   slug from the issue title using ASCII letters and digits, collapsing separators and using `work`
   only when no title character remains. The branch is `<ISSUE-ID>-<slug>`. Inspect local and remote
   branches before reusing that name. Once repository checks are complete and implementation will
   actually begin, move the issue to `In Progress`, then create or check out the branch without
   overwriting existing work. Report any partial failure instead of silently compensating.

7. **Attach the pull request once.** Before creation, search Bitbucket for an open pull request whose
   source is the work branch; reuse and attach it when found. Otherwise create the pull request with
   `bkt`, retrieve its canonical URL, and attach that URL to the Linear issue. If attachment fails
   after creation, preserve the pull request, report the out-of-sync state, and retry only the
   Linear link operation; never create another pull request.

8. **Limit lifecycle maintenance.** Verify each Linear write by reading the changed issue field or
   link back. Do not perform extra review, functional analysis, issue rewriting, or status ceremony
   solely to keep Linear synchronized. A request to check off completed items in the description is
   an evidence decision, not lifecycle maintenance: follow the completion-evidence reference owned
   by `linear-workflow` and check only lines a named run proves.

## Gotchas

- **Using the configured `bkt` context as repository identity** — a context can stay pinned while
  the working directory changes; derive the target from Git and pass it explicitly.
- **Treating any URL as an active pull request** — stale, terminal, forked, or unrelated links can
  redirect work; inspect repository, state, and source branch before checkout.
- **Creating before searching by source branch** — a missing Linear link can hide an existing pull
  request; search Bitbucket first and attach the existing canonical URL.
- **Updating Linear before work can start** — authentication or repository failures leave false
  progress; defer `In Progress` until all read-only gates pass.
- **Retrying the whole create-and-link sequence** — a link failure becomes a duplicate pull
  request; retry only the failed idempotent read or link step.

## Constraints

- Never start unassigned, blocked, or non-executable parent issues.
- Never use issue description text as a substitute for structured Linear ownership, relations,
  hierarchy, or state.
- Never create a branch or pull request while an attached resumable Bitbucket pull request exists.
- Never use a provider-specific Linear transport without verifying its current schema and required
  read and write capabilities.
- Never use Linear's GitHub-specific pull-request helper for Bitbucket work.
- Never invoke issue shaping unless the issue itself appears too broad or not implementation-ready.
