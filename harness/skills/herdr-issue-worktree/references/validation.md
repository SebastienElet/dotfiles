# Native validation observations

Recorded on 2026-10-07 on macOS ARM64, using zsh, Git, and Herdr client/server 0.9.3
(protocol 22, endpoint compatible), inside a managed pane with `HERDR_ENV=1`.
Only synthetic issue identifiers and titles are retained here. No production tracker or repository
was changed. The fixture labels below were supplied as synthetic tracker facts; a synthetic tracker
round trip and the composed implementation workflow were not exercised.

## Disposable creation and resume

The disposable Git repository was `/tmp/herdr-issue-worktree.lCnLwY/repository` with an empty
initial commit on `main`. Native Git resolved `/tmp` to `/private/tmp`.

| Operation                 | Native observation                                                                                                                                                                                                                                     |
| ------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| Discover contracts        | `herdr worktree`, `worktree create --help`, `worktree open --help`, and `agent start --help` exposed the expected label, focus, path, branch, base, pane, and provider options.                                                                        |
| Create Codex checkout     | `worktree create` with explicit branch, base, path, label, and `--no-focus` returned workspace `wW`, tab `wW:t1`, pane `wW:p1`, branch `TST-482-validation`, and checkout `/tmp/herdr-issue-worktree.lCnLwY/synthetic-checkout`.                       |
| Label and checkout        | Independent `workspace get`, `worktree list`, `pane get`, and `git worktree list --porcelain` confirmed `TST-482 — chore(api): upgrade the runtime's test fixture`, `is_linked_worktree=true`, repository common directory, checkout path, and branch. |
| Resume by path and branch | Native `worktree open --label ... --no-focus` returned `already_open=true`, the same workspace/tab/pane, and the same checkout. No second issue checkout was created. Opening while Codex was present also preserved its name and pane.                |
| Codex launch              | `agent start herdr-validation --kind codex --pane wW:p1` detected Codex there. Independent `agent get` and `pane process-info` confirmed its actual foreground cwd as `/private/tmp/herdr-issue-worktree.lCnLwY/synthetic-checkout`.                   |
| Create Claude checkout    | `worktree create --workspace wT ... --no-focus` returned `wX`, `wX:t1`, and `wX:p1` for branch `TST-483-validation`, path `/tmp/herdr-issue-worktree.lCnLwY/claude-checkout`, and label `TST-483 — chore: verify Claude launch`.                       |
| Claude launch             | `agent start herdr-validation-claude --kind claude --pane wX:p1` detected Claude there. Independent `agent get` and `pane process-info` confirmed its actual foreground cwd as `/private/tmp/herdr-issue-worktree.lCnLwY/claude-checkout`.             |
| Main checkout             | Git rereads kept the fixture's main checkout on `main` at its original HEAD.                                                                                                                                                                           |

## Focus and startup limits

Immediate native API snapshots before and after resume retained the same focused workspace, tab,
and pane. The same immediate comparison around creation with an explicit source workspace retained
all three focus IDs. Registering a second disposable repository with
`workspace create --cwd ... --no-focus` likewise preserved all three IDs.

An earlier comparison spanning several operations showed a different focused source workspace;
that interval did not isolate the cause. It is not evidence of focus preservation for implicit
source registration through `--cwd`. The skill therefore registers a missing source explicitly
with `--no-focus`, uses its returned workspace ID, and checks each operation independently.

Both providers stopped at their project trust UI. Herdr returned `agent_not_ready` and retained
each agent with `agent_status=blocked` and `launch_pending=true`. No trust dialog was answered,
no prompt was sent, and no implementation was claimed. Process cwd and provider placement were
verified; interactive readiness and the agent's own initial directory confirmation remain unproven.

The disposable linked worktrees and test agents were removed through native Herdr operations after
inspection. The focused fixture source workspace was retained to avoid a cleanup-induced focus
change. No existing agent workspace was closed, renamed, or removed. Linux, Windows/WSL,
remote Herdr machines, other Herdr versions, and live tracker lifecycle writes were not exercised.

## Routing and composition review

The activation scenarios are synthetic examples, not executed model-routing evidence.
The scoped cross-check compares this skill with the installed native Herdr skill and the canonical
`linear-start`, `linear-workflow`, and `claude-developer` skills:

- Herdr owns terminal safety and agent lifecycle; this skill requests the user-authorized worktree
  topology and consumes the native returned pane rather than applying the default sibling split.
- Linear workflows keep their eligibility, branch, pull-request, and lifecycle responsibilities;
  provisioning runs after read-only preflight and implementation continues in the isolated checkout.
- Explicitly choosing Claude for a native Herdr launch differs from asking for a manual handoff or
  automatically selecting Claude. Manual handoff remains with `claude-developer`.
- Git naming and base/path decisions come from the target repository; the human label does not
  determine the Git slug.

## Skill maintenance checks

`skill-manager doctor herdr-issue-worktree user` was applied as a read-only manual audit: standard
frontmatter fields/types/limits, slug/category, required sections, gotchas/constraints, reference
links, shell placeholder safety, synthetic eval structure, index membership, absence of a project
duplicate, and project adapters passed. Standard validator execution was unavailable because
`skills-ref` was not installed. The description is 249 characters and the body is below 500 lines.

The scoped composition cross-check found no unresolved conflict. D1 description token overlap
with the three canonical composed workflows ranged from 7.8% to 16.7%, below its 40% warning
threshold. D2 found composition rather than a duplicated start procedure; D3 references resolved;
D4 distinguished explicit native launch from manual handoff; D5 found no duplicate slug;
D6 was inapplicable because no scoped sibling references were added. Independent standards and
requirements reviews were performed; the shell-readiness prerequisite was narrowed to new launches
so existing agents can be reused on resume.

The user index was regenerated and formatted twice with identical SHA-256
`26df7976a2e7e69d8ee85929257f69775d818f731f6a3f4ba7eaed9dfce93c21`; its only membership change
was this new skill. On macOS, `repository:prettier-check` and `harness:validate-evals` passed with
the new files already indexed. The latter validates scenario structure, not model activation.

Claude, Cursor, and Codex user links were deployed with the existing `tooling/deploy-link.ts`
leaf mechanism used by the Moon deployment utilities. Each link resolves to this worktree's
canonical `harness/skills/herdr-issue-worktree` source. Existing user links were preserved.
The new Arnes installation declarations are in this worktree's `home/.arnes.yaml`; the live home
manifest still points to the primary checkout until the change is integrated and deployed there.
Whole-collection redeployment from this worktree was avoided because existing links target that
primary checkout. The new links consequently depended on this worktree remaining available at
validation time. This observation does not establish their current destinations or a completed
repatriation.

## Deployment source lifecycle

Persistent user deployments use the canonical checkout at `~/.dotfiles`. An exceptional validation
from a worktree must plan its exit before creating links:

1. Prefer disposable, isolated destinations. Inspect the Moon task and action graph, exclude global
   installation dependencies, and run only fixture-safe deployment leaves; isolated `HOME` alone
   does not sandbox Homebrew or application installers.
2. If a real user destination must temporarily point into a worktree, record the affected link and
   its intended canonical replacement source. Before deploying, run
   `git worktree lock --reason 'temporary deployment source' /absolute/worktree/path` and verify
   the lock with `git worktree list --porcelain`. Keep it locked while any deployed source depends
   on that checkout.
3. Once the validated change is integrated into the canonical checkout, verify the replacement
   source exists there. Inspect each recorded destination; explicitly redirect only links still
   targeting the validation worktree, then verify their resolved targets and source accessibility.
   An unexpected destination requires inspection rather than overwrite. The deployment helper
   refuses divergent links, so rerunning installation alone does not repatriate them.
4. After verifying that no recorded deployment still depends on the worktree, run
   `git worktree unlock /absolute/worktree/path`. Dispose of isolated destinations before removing
   their source checkout. Only then allow worktree cleanup; retain the lock if repatriation fails.

Herdr's cleanup plugin reports Git removal failures, preserves locked worktrees and skips detached
HEADs; it does not discover user links or repair them. Non-ignored untracked files prevent removal
even with `status.showUntrackedFiles=no`, but ignored files are deleted with an otherwise clean
checkout. Valuable ignored artifacts need separate storage or an explicit lock, not an assumption
of universal preservation. Additional ignored-file protection is a separate decision.

This lifecycle is a procedure, not evidence that the historical user links above were redirected.
