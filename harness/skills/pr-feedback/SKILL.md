---
name: pr-feedback
description: >
  Collect evidence-backed review feedback and reviewer-authored fixes from merged pull requests. Use
  when analyzing recurring PR review findings or preparing factual harness-improvement evidence.
  Make sure to use this skill whenever retrospective review feedback is requested, even if no
  forge or skill is named.
compatibility: Authenticated forge CLI for the repository remote.
metadata:
  category: dev
---

# PR Feedback

## Overview

Produce a concise, traceable record of problems found during review and fixes attributable to reviewers.
Establish observed behavior and evidence without recommending a harness change. This workflow is
read-only and never edits repository or forge state.

## Usage

`/pr-feedback <PR numbers or count>`

Accept an explicit list of pull request numbers or a count. By default, inspect the latest ten
merged pull requests authored by the user.

## Steps

1. Derive the forge from `git remote get-url origin` and use only its client. Derive the repository
   slug from that remote; never hard-code it. With `gh api`, use literal `{owner}/{repo}` where the
   client supports substitution. Verify options in the selected client's help before using them.

2. Resolve the requested scope. These examples list the default sample:

   ```bash
   bkt pr list --mine --state MERGED --workspace <workspace> --repo <repo> --limit 10 --json --jq '.pull_requests[] | "\(.id) | \(.title)"'
   gh pr list --author @me --state merged --limit 10 --json number,title
   ```

   Without a repository argument, `bkt pr list --mine` spans every repository in the workspace and
   applies the limit to that wider result, while `gh pr list` is repository-local. Scope Bitbucket
   to the derived repository before applying the requested count.

3. For each pull request, collect the push sequence. A squash merge erases reviewer-fix commits
   from the destination history and may assign them to the pull request author, so the forge remains
   the primary evidence:

   ```bash
   bkt api "/repositories/<slug>/pullrequests/<id>/activity?pagelen=50" --json --jq '.values[] | select(.update) | "\(.update.date) | \(.update.author.display_name) | \(.update.source.commit.hash) | \(.update.state)"'
   gh pr view <id> --json commits --jq '.commits[] | "\(.committedDate) | \(.authors[0] | if (.login // "") == "" then .name else .login end) | \(.oid[0:12]) | \(.messageHeadline)"'
   ```

   Bitbucket activity identifies the actor who updated the branch. GitHub's commit `authors` field
   identifies Git authorship and `Co-authored-by` trailers, not the actor who pushed the commit.
   Record push attribution as uncertain unless separate forge evidence identifies that actor. Treat
   third-party authorship or a third-party push only as a potential reviewer fix. Verify its delta;
   merges, rebases, and automated branch updates are not fixes. Do not limit collection to pushes
   after the author's last push.

4. Reconstruct the commit chain before computing a reviewer delta. Forge activity can omit author
   pushes, and branches can absorb integration merges or squash-merged pull requests attributed to
   the author:

   ```bash
   bkt api "/repositories/<slug>/commits/<head>?pagelen=10" --json --jq '.values[] | "\(.hash[0:12]) | \(.author.user.display_name // .author.raw) | \(.message | split("\n")[0])"'
   gh api "repos/<slug>/commits?sha=<head>&per_page=10" --jq '.[] | "\(.sha[0:12]) | \(.author.login // .commit.author.name) | \(.commit.message | split("\n")[0])"'
   ```

   The lower bound is the commit before the reviewer's first commit, including an integration merge
   when present, never the SHA shown in activity. Check the file summary before reading the diff. A
   delta touching files unrelated to the pull request indicates a wrong bound.

   ```bash
   bkt api "/repositories/<slug>/diffstat/<head>..<base>?pagelen=100" --json --jq '.values[] | "\(.status) +\(.lines_added) -\(.lines_removed) \(.new.path // .old.path)"'
   bkt api "/repositories/<slug>/diff/<head>..<base>?path=<file>"
   gh api "repos/<slug>/compare/<base>...<head>" --jq '.files[] | "\(.status) +\(.additions) -\(.deletions) \(.filename)"'
   gh api "repos/<slug>/compare/<base>...<head>" --jq '.files[] | select(.filename == "<file>") | .patch'
   ```

   Read every retained patch completely. Formatting, commit regrouping, and branch synchronization
   are not behavioral findings.

5. Collect comments, reviews, and tasks:

   ```bash
   bkt pr comments <id> --json --jq '.comments[] | "\(.user.display_name) | \(.inline.path // "general"):\(.inline.to // "") | \(.links.html.href // .id) | \(.content.raw)"'
   bkt pr task list <id> --json
   gh pr view <id> --json comments,reviews --jq '(.comments[], (.reviews[] | select(.body != ""))) | "\(.author.login) | \(.url // .id) | \(.body)"'
   gh api "repos/<slug>/pulls/<id>/comments" --jq '.[] | "\(.user.login) | \(.path):\(.line) | \(.html_url) | \(.body)"'
   ```

   Distinguish human reviewers, AI reviews posted by CI, and the pull request author's own notes.
   Exclude the author's notes.

6. Turn every retained comment or reviewer-authored fix hunk into a behavioral finding containing only:
   what was found or corrected, the comment URL or commit/file evidence, the observable outcome,
   and an observed impact when explicitly established. Otherwise write `not established`.

7. Deduplicate a comment and its fix into one finding while preserving both sources. Consolidate
   the same problem across pull requests without removing occurrences. Put any unproven
   interpretation under `Uncertainty`; never infer the original development cause or propose a
   harness improvement.

8. Return exactly this structure, with empty sections written as `_none_` and no preamble or
   conclusion:

   ```markdown
   # Review feedback — <repository>

   ## Scope

   | PR          | Title   | Reviewer(s)  | Merge             | Sources inspected |
   | ----------- | ------- | ------------ | ----------------- | ----------------- |
   | <linked id> | <title> | <identities> | <date and merger> | <counts>          |

   ## Consolidated findings

   ### C01 — <problem stated as behavior>

   - Occurrences: <pull requests>
   - Evidence:
     - <linked PR> · <human comment or AI review> · <author> · <file:line> — <faithful paraphrase>
     - <linked PR> · reviewer-authored fix · <author> · <SHA> · <file> — <before → after>
   - Observable outcome: <status and evidenced author or push actor without assumed causality>
   - Observed impact: <established fact or not established>
   - Uncertainty: <precise limit or none>

   ## Inventory by PR

   - PR <id> — <finding count and ids>; <excluded signal count>

   ## Excluded signals

   - <linked PR> · <signal> — <factual reason>

   ## Collection limits

   - <unavailable source, incomplete pagination, uncertain attribution or diff bound>
   ```

## Gotchas

- **Trusting squash history** — reviewer commits are assigned to the author or erased; reconstruct
  evidence from forge activity and the pre-squash branch chain.
- **Using the activity SHA as the lower diff bound** — integration changes contaminate the delta;
  locate the commit preceding the reviewer's first commit and verify the file summary.
- **Treating every third-party push as a fix** — merges, rebases, and automation become false
  findings; retain only a delta that changes identifiable behavior.
- **Turning evidence into recommendations** — the record biases the later harness analysis; report
  only observed behavior, outcomes, impacts, and explicit uncertainty.
- **Ignoring pagination or missing sources** — the record appears exhaustive when it is not; name
  every collection limit in the output.

## Constraints

- Remain read-only: never edit repository files or mutate forge state.
- Support only claims linked to a pull request, comment, task, commit, file, or patch.
- Keep unknown input raw and explicit; never complete missing evidence by plausibility.
- Use one command per tool call when a worktree guard rejects loops, substitutions, or heredocs.
- With `bkt`, provide `--json` whenever using `--jq`.
- Do not inspect the harness, recommend changes, assign priorities, or draft rules to paste.
