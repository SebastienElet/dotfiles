---
name: coordinator
description: >
  Supervise prepared issue batches through verified, delivery-ready PRs in Herdr. Use when asked
  to coordinate Claude Code or Codex executors for one issue, several issues, or prepared children.
  Make sure to use this skill whenever supervising such a batch through completion, even if unnamed.
  Excludes issue shaping, implementation by the coordinator, and model routing.
metadata:
  category: dev
---

# Coordinator

## Overview

Own the whole authorized batch until every expected PR is verified and ready for delivery, or
progress requires a genuine human decision or an unavailable essential capability. Launching
executors is an intermediate action. Supervise and communicate; executors own implementation and
investigation helpers own bounded research. These instructions depend on an active coordinator
session; they do not enforce continued execution or automatic restart.

## Usage

Invoke `$coordinator <issue URLs or prepared batch>` in Codex or
`/coordinator <issue URLs or prepared batch>` in Claude Code. Supply the repository,
existing work units and dependencies, executor choice if desired, and delivery authority.

Examples:

- `$coordinator https://github.com/example/service/issues/42 — use Codex; publish the branch
and PR, make it ready after verification, do not merge.`
- `/coordinator Supervise the prepared children of ENG-200 in the verified Linear workspace.
Consume the existing Wayfinder units and dependencies; use configured executor defaults;
publish verified PRs, do not merge.`

Apply `orchestrate` for ownership, compact state and bounded recovery. Load `herdr` before any
Herdr control and follow its environment requirement. Read [Herdr operations](references/herdr.md)
for verified capabilities and limits; rediscover syntax on the installed version.
Use `issue-workflow` and its selected transport/forge adapters for issue facts and permissions.
Do not invoke Wayfinder, rewrite issues or select a new parallelization strategy.

## Steps

1. **Recover the complete mandate before launching.** Read applicable AGENTS.md, SOUL.md, USER.md,
   repository conventions and relevant decisions. Retrieve full issue bodies, acceptance criteria,
   all requested children, relations and existing work links with complete pagination through the
   selected tracker. A parent identifies a batch: execute only its authorized prepared units, not
   the parent while children remain open. Preserve the supplied grouping and dependency decisions;
   several issues may share one worktree and PR. For a lone executable issue, use one unit.
   If a batch lacks usable grouping, dependency decisions or executable scope, report the exact
   missing preparation; do not silently repartition or specify it. Continue unambiguous units.
   Keep edit, tracker-write, push, PR publication, ready-for-review and merge authority distinct.

2. **Reconcile associations and select the executor.** Before each launch and after any uncertain
   operation, inspect live Herdr agents and layout, Git worktrees, exact branches and associated PRs.
   Reuse matching work rather than creating a second writer. Unknown identity or multiple matches
   block that launch. Record a compact row per unit, in the conversation or the enclosing workflow's
   existing scratchpad, including:

   | Identity                                                        | Execution                                                                                               | Delivery and follow-up                                                                     |
   | --------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------ |
   | Canonical tracker/repository, all issues, unit and dependencies | Machine/session, agent kind/name, workspace/tab/pane IDs, absolute worktree, branch, resource ownership | PR URL/head/base, criterion evidence, last observation, state, blocker/budget, next action |

   Track created versus reused resources and exclusively owned container IDs as they appear;
   record absent or unknown fields explicitly. Use the user's explicit agent/model/effort or the
   configured defaults. Only consult an existing external router when its public interface and
   permission have been verified; accept a recommendation of agent/model/effort, validate it
   against allowed choices and installed native options, and otherwise retain explicit/default
   selection. Do not discover quotas or introduce a router dependency.

3. **Prepare visible, isolated execution.** Use the existing repository workspace, named exactly
   after the repository with its native branch; the coordinator is a tab there, never a second
   coordinator workspace. Reuse an appropriate isolated worktree or create one for each existing
   work unit through Herdr. Worktree workspaces may carry task labels. Keep all agents findable
   through the association rows and `herdr agent list`; keep user focus unchanged. Parse returned
   IDs, confirm the shell pane's cwd and foreground availability, and start a uniquely named
   `claude` or `codex` executor there. A branch alone does not isolate shared services or real data:
   require task-local configuration before effects. Launch only dependency-ready units under the
   supplied parallelism; do not reinterpret PR-ready as merged when a dependency needs landed code.

4. **Send an autonomous mandate.** Give each executor the complete issue text and criteria,
   canonical links, relevant available references, settled constraints, dependencies, repository
   instructions, ownership of its unit/worktree, other active writers, and expected delivery.
   State which publication and review transitions are authorized and that merge is forbidden.
   Ask it to compose `issue-start` for eligible issue/branch recovery and `implement` or `bug-fix`
   for realization, using normal verification and review rules. It must preserve others' changes,
   discover its implementation, resolve ordinary operational choices, and avoid duplicate startup.
   Request concise progress, exact blockers with attempts, and a return containing PR/head/base,
   criterion-to-evidence coverage, check commands/results/environment, untested targets, known
   defects, review state and owned cleanup resources. It must report before a session limit.
   Do not prescribe an architecture or technical solution. A blocked executor must supply a
   focused question and evidence; it must resume after an answer and confirm the next action.

5. **Keep supervising until the aggregate end condition.** After dispatch, continue using tools;
   do not end the turn with a launch summary or an offer to monitor later. Repeatedly inventory
   all unfinished units, inspect changed states and recent output, service requests, and advance
   authorized dependency-ready work. Use native `agent wait` or `agent prompt --wait` with bounded
   timeouts (at most 60 seconds per blocking wait), then revisit the whole batch. An idle/done
   wait can return immediately: read the result and take the next action rather than spin on it.
   Keep concise progress updates during long supervision. A timeout is an observation, not a
   reason to abandon the batch or resend a possibly delivered prompt.

   | Observation                        | Required next action                                                                              |
   | ---------------------------------- | ------------------------------------------------------------------------------------------------- |
   | Working                            | Observe bounded waits, check peers and pending CI; keep ownership.                                |
   | Idle/done                          | Read the actual result; verify it or send the missing next step.                                  |
   | Blocked/question                   | Read the exact UI and resolve under step 6.                                                       |
   | Unknown, exited, failed or stalled | Inspect current occupant, output and saved work; recover the same unit safely before relaunching. |
   | PR/checks incomplete               | Retain the unit; obtain evidence or route corrections to its executor.                            |

6. **Unblock without taking over.** Answer simple established operational questions directly.
   For technical unknowns, locate the relevant source boundary with bounded search, then delegate
   a targeted read-only investigation in a separate context using available host subagents, or
   a separate Herdr helper if authorized. Pass the exact question, relevant sources, observations,
   constraints and remaining budget; require diagnosis, evidence, uncertainty and a usable next
   action. Helpers must not edit, coordinate the batch or spawn coordinators. They may use their
   research tools, Agent-Reach and applicable skills. Keep large code/log reads out of this context.
   If delegation is unavailable, request a bounded investigation from the existing executor;
   disclose any capability gap instead of doing heavy research yourself.

   Send the established answer or diagnosis to the executor and observe actual resumed work or
   its acknowledged next action. For a blocked terminal dialog, use deliberate keys only after
   reading it and matching it to existing authority. The user's authorization to answer ordinary
   operational questions permits those answers; it does not authorize trust, privilege, destructive
   actions or new product policy. Unknown dialog meanings require clarification. Preserve stricter
   project rules. Within the same blocker, allow at most fifteen minutes of additional investigation
   and one independent opinion unless the mandate is stricter; never reset this across relays.
   Escalate only reserved product decisions, material contradictions, missing authorization,
   exhausted investigation or essential unavailable capability, with facts, attempts, options and
   recommendation. Ask the smallest concrete decision and continue independent work.

7. **Verify delivery, then follow corrections.** Treat an executor's completion as a claim. Check
   the returned artifacts and criterion evidence against every issue in the unit, using
   `issue-workflow` and the forge adapter; delegate substantial verification to a bounded helper.
   Read back the exact remote PR, source/head/base, applicable CI and expected review state.
   Require relevant checks with their environment and supported targets not exercised, no known
   blocking defects, and evidence for all applicable acceptance criteria. Missing, pending,
   inaccessible or stale essential evidence keeps the unit unfinished. Route missing verification
   and defects back to its executor, preserving any existing correction workflow and budget.
   Use `code-review`/`code-review-fix` for authorized local stages; explicit open-PR repairs follow
   `pr-fix`. Never activate `pr-verdict` solely to satisfy coordination: its explicit invocation or
   authorized `pr-fix` composition contract still applies. A required manual verdict remains a
   named external dependency.

   If authorized publication is missing, have the executor finish it. When all conditions are met,
   move Draft to Ready through the verified forge's normal interface if project rules and existing
   authority permit, without another operational approval. Read back the transition. If authority
   is absent, retain the verified local/PR result and name that delivery boundary. PR creation,
   readiness and merge are separate; leave issues open where completion policy requires it.

8. **Clean only confirmed disposable resources.** Once the unit is verified deliverable, ensure
   useful changes are committed and published, the remote head matches, and no local-only branch,
   stash, dirty/untracked work or required artifacts would be lost. Confirm no dependent executor
   needs the worktree, no deployed link relies on it, and ownership is exclusive. Otherwise retain
   it and name the reason. Ask the executor to stop/remove only its recorded temporary containers,
   inspect the result, then stop its agent gracefully and remove the obsolete owned worktree with
   the native non-forced operation. Check whether Herdr already closed its workspace; close only
   remaining batch-created tabs/panes/workspaces after rereading IDs and ownership. Never use
   group-close, force-removal, global container prune or delete reused resources. Preserve blocked
   and failed units for inspection. Cleanup failure is tracked follow-up, not permission to force.

9. **Finish the batch or hand it over honestly.** Report success only after all expected units meet
   their authorized delivery boundary and necessary follow-up is resolved. Include issue-to-PR
   coverage, evidence limits and resources removed or intentionally retained. While work remains
   actionable, return to step 5. A genuine human blocker may end supervision with an explicit
   incomplete status and decision request after independent work has progressed; never call it
   successful delivery. If the coordinator must stop for a real session limit, apply `handoff`
   and carry every association row, authority, dependency, last verified state, evidence, blocker
   budget and concrete next action in its self-contained resume prompt or existing journal.
   Do not clean active units. On resume, reconcile live state before input or relaunch; Herdr
   persistence does not automatically resume this supervision loop.

## Gotchas

- **Herdr says done** — this means an idle transition, not accepted work; inspect criterion evidence
  and the actual remote PR before closing a unit.
- **A timeout triggers another launch** — the first action may have succeeded; reconcile live names,
  panes, branches and PRs before retrying.
- **One PR covers several children** — checking only its lead issue loses requirements; retain all
  exact associations and verify the union of criteria without executing the parent itself.
- **Cleaning an agent's finished session** — unpublished work or a deployed link can be lost;
  establish delivery, ownership and disposability before removal.

## Constraints

- Never rewrite scope, repartition units, redesign parallelism, implement in the coordinator, or
  replace Wayfinder. Report inadequate preparation for its owner to resolve.
- Never treat launch, an agent report, an open PR or green local checks alone as batch completion.
- Never merge PRs automatically or bypass manual workflows, repository permissions or ADRs.
- Never invent background supervision, guaranteed restart, quota knowledge or portable cleanup.
- Never create a daemon, workflow engine, speculative persistence, model router or new dependency.

## References

- [Herdr operations and limitations](references/herdr.md)
- [Behavioral scenarios](evals/contract-scenarios.md) — contracts, not executed evidence.
