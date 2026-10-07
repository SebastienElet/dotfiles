---
name: herdr-issue-worktree
description: >
  Start or resume issue work in a visibly labeled Herdr Git worktree with Codex or Claude.
  Use only when the user explicitly requests Herdr to work on an issue; never select it for
  issue lookup, generic implementation, or an agent or pane name alone.
compatibility: Requires HERDR_ENV=1, the native Herdr CLI, Git, and authenticated tracker access.
metadata:
  category: dev
---

# Herdr Issue Worktree

## Overview

Provision or reuse one isolated Git checkout, register it as a visible Herdr worktree workspace,
and start the chosen agent there without moving user focus. The workspace label is
`<ISSUE-ID> — <exact current issue title>`; a named agent or pane is insufficient.
Compose the installed `herdr` skill for terminal control instead of duplicating it.

## Usage

`$herdr-issue-worktree <issue ID or URL> <Codex|Claude> [repository]`

Example: "Use Herdr to resume TST-482 with Claude in the fixture repository."
Natural-language requests must explicitly ask for Herdr and starting or resuming issue work.
An explicit invocation authorizes this topology only when it supplies an issue to work on.
Ask for the agent choice if absent; map Codex to `codex` and Claude to `claude`.
Do not substitute a provider or infer the choice from the calling agent.

This workflow owns checkout provisioning and native agent launch. Repository Git conventions own
branch, base, and path policy. For Linear/Bitbucket implementation, `linear-start` and
`linear-workflow` retain eligibility, repository and pull-request resolution, branch policy,
lifecycle writes, and implementation workflow. Run their read-only preflight before provisioning;
continue implementation in the isolated checkout. Provisioning alone never updates the tracker,
creates a pull request, or authorizes a commit, push, merge, or deletion.

Choosing Claude explicitly for this Herdr launch authorizes that native launch; it is not an
agent-selected delegation. A request for a manual Claude handoff belongs to `claude-developer`.

## Steps

1. **Establish the native contract.** Verify `HERDR_ENV=1` before controlling Herdr; otherwise
   report the unavailable environment and stop. Load the installed `herdr` skill, using
   `herdr --skill` when it is not already available. Follow its safety and startup rules.
   Inspect `herdr --help`, client/server status, and the worktree, workspace, pane, and agent
   command groups. Rediscover `worktree create --help`, `worktree open --help`, and
   `agent start --help` before using them. Missing native worktree labeling support is a blocker;
   never replace it with a plain workspace, pane name, wrapper, or server upgrade.

2. **Resolve current issue and Git intent.** Read the identifier and exact title from the
   authenticated tracker, not a prompt, branch slug, or cached summary. Build the label without
   lowercasing, trimming, transliteration, truncation, or shell evaluation. Resolve the repository
   from verified remotes and repository instructions. Apply the applicable start workflow's
   read-only eligibility and attached pull-request checks. Resolve the expected branch, base,
   and worktree path from that workflow and repository Git conventions; an attached resumable
   pull request determines its source branch. Do not use the tracker's suggested branch name as
   authority over repository policy. Ask only for missing material decisions. Record the main
   checkout's branch and HEAD before any mutation; never check out or switch its branch.

3. **Select one checkout without duplication.** Inspect `git worktree list --porcelain`, local
   and remote branches, and native `herdr worktree list --cwd <verified-checkout>` alongside
   `herdr workspace list`. Compare canonical paths and Git common directories, not labels alone.
   Reuse the issue checkout only when repository identity and expected branch match. A matching
   label on another repository, a detached/prunable checkout, a branch checked out elsewhere,
   or several candidates requires resolution before creation. Do not create a second checkout
   to escape a collision. Existing changes stay intact. Inspect occupants before reopening:
   never relabel another agent's workspace. A conflicting or stale label must be resolved with
   its owner rather than silently renamed.

4. **Register the visible worktree.** Prefer a verified existing repository workspace as the
   explicit source. If none exists, rediscover `workspace create --help`, register the repository
   with native `herdr workspace create --cwd <verified-checkout> --label <repository-name> --no-focus`,
   and read its returned source workspace ID. This source workspace is not the issue workspace.
   For an existing matching checkout, use native
   `herdr worktree open --workspace <source-workspace-id> --path <existing-path> --label <exact-label> --no-focus`.
   Otherwise use native
   `herdr worktree create --workspace <source-workspace-id> --branch <expected-branch> --base <verified-base> --path <policy-path> --label <exact-label> --no-focus`.
   Adapt only to the discovered CLI contract. Pass arguments as data or use proper shell quoting;
   titles can contain apostrophes, backticks, dollar signs, and newlines. Read the returned
   workspace, root pane, checkout path, and branch; never predict IDs or paths. An already-open
   result must reuse that workspace. On failure or timeout, inspect Git and Herdr state before
   retrying; preserve a partially created checkout and retry only its missing registration.
   Do not add `--trust-repository` as a routine retry.
   Read the native focus snapshot immediately before and after each topology operation. Report a
   focus discrepancy instead of issuing a compensating focus command that could override the user.

5. **Launch or resume the selected agent in the returned pane.** Inspect that pane and its
   processes first. On resume, reuse an existing agent only after verifying its kind, pane, cwd,
   and issue session identity. Do not restart it or resubmit a prompt while it is working.
   For a new launch only, require an available shell with its cwd resolving to the returned checkout
   and use native
   `herdr agent start <unique-name> --kind <codex-or-claude> --pane <returned-pane-id>`.
   A different occupant,
   uncertain ownership, or the wrong cwd blocks launch; never replace, close, rename, or send
   input to another agent. Follow the composed Herdr skill for readiness, trust dialogs, prompts,
   timeouts, and partial startup. Give the ready agent the issue reference, verified checkout and
   branch, authorized outcome, and applicable implementation workflow. Require it to confirm
   `pwd -P`, Git top-level, and branch before editing. Do not relocate layout or focus.

6. **Verify independently and report.** Read `herdr workspace get <returned-workspace-id>` and
   the workspace list again: require the exact label, `worktree.is_linked_worktree=true`, matching
   repository identity, and the returned isolated checkout. Cross-check Git's worktree inventory,
   common directory, top-level, and actual branch. Use native `herdr agent get`, `pane get`, and
   `pane process-info --pane <returned-pane-id>` to check agent kind and foreground cwd; creation
   output alone is insufficient. Compare canonical paths to accommodate platform aliases.
   Check the agent's initial directory confirmation when ready. Verify that the main checkout's
   branch and HEAD remain unchanged and that no second issue checkout/workspace was created.
   Report label, branch, path, and agent concisely. If startup is blocked or a check is unavailable,
   name the observed state and missing proof; do not claim implementation started.

## Gotchas

- **Naming only the agent** — the sidebar still lacks the issue worktree; verify native workspace
  worktree metadata and its exact human label.
- **Slugging the title into the label** — punctuation and casing are lost; preserve the tracker
  title verbatim and restrict Git normalization to the branch/path policy owner.
- **Recreating on resume or timeout** — a live workspace or partial checkout becomes a duplicate;
  inspect both inventories and reuse the returned existing handles.
- **Launching in the caller or focused pane** — the agent starts in the main checkout or someone
  else's terminal; target only the verified pane returned by the native worktree operation.
- **Treating startup failure as absence** — Herdr may retain a blocked agent and name; inspect it
  before retrying, and report trust/readiness limits without bypassing the dialog.

## Constraints

- Activate only for explicit Herdr issue-work requests, never for issue consultation alone.
- Use native Herdr operations; add no wrapper, custom validator, or copied Herdr procedure.
- Keep the exact tracker identifier and title in the workspace label; keep IDs opaque.
- Never change the main checkout's branch or overwrite existing work on resume.
- Never create a duplicate checkout/workspace to work around ambiguity or an occupied pane.
- Never close, remove, rename, or commandeer another agent's work or move user focus.
- Honor the explicit Codex/Claude choice and the composed workflows' authorization boundaries.
- Use only synthetic tracker identities and disposable repositories in retained evals and evidence.

## References

- [references/validation.md](references/validation.md) — disposable native creation/resume observations
- [evals/trigger-queries.json](evals/trigger-queries.json) — synthetic activation scenarios
