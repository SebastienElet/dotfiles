# Public Dotfiles Cleanup Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development
> (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use
> checkbox (`- [ ]`) syntax for tracking.

**Goal:** Remove obsolete professional configuration from the public dotfiles repository, rebuild
`skill-manager` from the generic lessons in `~/Code/brain`, and leave every project skill clean.

**Architecture:** Keep `.agents/skills/` as the canonical skill source and preserve its three
agent-specific adapters. Deliver three independent implementation commits: repository cleanup,
`skill-manager` reconstruction, then findings-driven global skill cleanup. Every deletion removes
its callers and current ADR claims; every skill write goes through the rebuilt doctor/fix workflow.

**Tech Stack:** GNU Make, Fish, portable Bash, Markdown/YAML/JSON agent skills, GitHub issues, local
macOS tooling (`fish`, `fish_indent`, `shellcheck`, `prettier`, optional `skills-ref`).

---

## File map

### Commit 1 — public repository cleanup

- Delete: `scripts/import-instance-start`, `scripts/import-instance-stop`,
  `scripts/cheque-parser-instance-reboot`
- Delete: `scripts/git_hook_push`, `scripts/git_hook_assert_typos`,
  `scripts/git_hook_assert_todoes`, `scripts/git_hook_assert_eslint`,
  `scripts/git_hook_assert_empty_files`, `scripts/git_hook_detect_copy_paste`
- Delete: `cursor/settings.json`, `cursor/extensions.json`, `cursor/keybindings.json`
- Delete: `docs/adr/020-hooks-de-push-maison.md`
- Modify: `Makefile` — retain Cursor CLI and its project skill, remove Cursor IDE/configuration
- Modify: `fish/conf.d/aliases.fish`, `fish/conf.d/git-abbreviations.fish` — remove old callers and
  make `gp` expand to `git push`
- Modify: `dict/user.txt` — retain an empty dictionary file
- Modify: `docs/adr/README.md`, `docs/adr/012-abbreviations-fish.md`,
  `docs/adr/021-branche-main-detectee.md` — keep only active claims
- Modify: `.agents/skills/apple-notes/SKILL.md`, `.agents/skills/johnny-decimal/SKILL.md`,
  `.agents/skills/scripts/SKILL.md`, `scripts/jdl` — neutral examples and valid references

### Commit 2 — `skill-manager` reconstruction

- Modify: `.agents/skills/skill-manager/SKILL.md`
- Modify: `.agents/skills/skill-manager/references/{conventions,create,doctor,fix,cross-check,sync-index,evals}.md`
- Create: `.agents/skills/skill-manager/evals/trigger-queries.json`
- Modify: `.agents/skills/README.md`

### Commit 3 — global skill audit findings

- Modify: `.agents/skills/do-nothing-script/SKILL.md` for the currently reproducible finding
- Modify only additional skill files named by a reproducible doctor or approved cross-check finding
- Modify `.agents/skills/README.md` only through the final `sync-index`

## Task 1: Remove obsolete configuration and callers

**Files:** all commit-1 files from the file map.

- [ ] **Step 1: Record the pre-change acceptance baseline**

```bash
git status --short --branch
rg -n 'cheque-parser-instance-reboot|import-instance-(start|stop)|git_hook_(push|assert_)|git_hook_detect_copy_paste|Cursor\.app|\.config/Cursor/User' Makefile fish scripts docs/adr
test -s dict/user.txt
```

Expected: the branch is `issue-64-public-dotfiles-cleanup`; `rg` prints every obsolete caller and
ADR reference; the dictionary assertion succeeds. This is RED because the forbidden state remains.

- [ ] **Step 2: Delete obsolete files with `apply_patch`**

Delete exactly the thirteen physical files listed under commit 1. Do not delete
`scripts/git_main_branch` or anything under `.cursor/`.

- [ ] **Step 3: Reduce the Cursor target to the CLI surface**

Replace the complete Cursor block in `Makefile` with:

```make
cursor: ~/.local/bin/cursor-agent ~/.cursor/skills/merge-verdict
~/.local/bin/cursor-agent:
	curl https://cursor.com/install -fsS | bash
~/.cursor/skills:
	mkdir -p $@
~/.cursor/skills/merge-verdict: ${DOTFILES_PATH}/.agents/skills/merge-verdict | ~/.cursor/skills
	ln -s ${DOTFILES_PATH}/.agents/skills/merge-verdict $@
```

Preserve aggregate wiring around `cursor`. Remove the `Cursor.app` sentinel,
`~/.config/Cursor/User` directory rule, and all three IDE symlink rules. Add no comment.

- [ ] **Step 4: Replace the Fish push wrapper and remove AWS aliases**

Delete these definitions from `fish/conf.d/aliases.fish`:

```fish
alias cheque-parser-instance-reboot='~/.dotfiles/scripts/cheque-parser-instance-reboot'
alias gp='~/.dotfiles/scripts/git_hook_push; and git push'
alias import-instance-start='~/.dotfiles/scripts/import-instance-start'
alias import-instance-stop='~/.dotfiles/scripts/import-instance-stop'
```

In the pull/push section of `fish/conf.d/git-abbreviations.fish`, use:

```fish
abbr -a gl 'git pull'
abbr -a gp 'git push'
abbr -a gs 'git status'
```

- [ ] **Step 5: Empty the user dictionary without deleting it**

Remove every line from `dict/user.txt` with `apply_patch`, then run:

```bash
test -f dict/user.txt
test ! -s dict/user.txt
```

Expected: both assertions succeed.

- [ ] **Step 6: Generalize examples outside `skill-manager`**

Invoke `/skill-manager doctor apple-notes`, `/skill-manager doctor johnny-decimal`, and
`/skill-manager doctor scripts`, and retain their baselines. These skills currently pass; #64 and
the user's explicit approval are the functional-change contract. The current `fix` procedure's
refusal to evolve a passing skill conflicts with that higher-priority request and is the known
defect corrected in Task 2; do not invent a doctor finding to disguise the conflict.

Apply these exact semantic substitutions, preserving syntax and behavior:

```text
.agents/skills/apple-notes/SKILL.md
  delete metadata.author
  Bigfoot : Recrutement : Template -> Company : Hiring : Template
  3 Resources/Recrutement -> 3 Resources/Hiring
  Recrutement : Template email de refus -> Hiring : Rejection email template
  2 Areas/Septeo -> 2 Areas/Company
  20-29 - Areas/21 - Septeo/21.08 - Recrutement -> 20-29 - Areas/21 - Company/21.08 - Hiring
  Septeo : Recrutement : Subject -> Company : Hiring : Subject

.agents/skills/johnny-decimal/SKILL.md
  21 - Bigfoot/21.03 - Meeting -> 21 - Company/21.03 - Meetings
  Bigfoot - Meeting - 2024-05-14 - Retour Septeo.xlsx -> Company - Meetings - 2024-05-14 - Quarterly Review.xlsx

scripts/jdl
  21 - Bigfoot -> 21 - Company
  21.03 - Meeting -> 21.03 - Meetings
  10.25001 - Import Quadral -> 10.25001 - Data Import
  Bigfoot - Meeting -> Company - Meetings
  Retour Septeo.xlsx -> Quarterly Review.xlsx

.agents/skills/scripts/SKILL.md
  git_hook_assert_eslint -> claude_handoff_check
```

Only explanatory strings change in `scripts/jdl`; no matching expression changes.

- [ ] **Step 7: Align the active ADR set**

Remove ADR-020 from `docs/adr/README.md`. In ADR-012, replace the obsolete exception with:

```text
Tous les raccourcis statiques, dont `gp`, sont des abbreviations. Seul `gpsup`, dont la commande
dépend de la branche courante, reste un alias.
```

In ADR-021, replace the consumer sentence with:

```text
Les deux appelants actifs — l'abbreviation `grbm` et `scripts/review` — passent par
`scripts/git_main_branch`, qui interroge le dépôt courant et retombe sur `master` le cas échéant.
```

- [ ] **Step 8: Run the commit-1 verification barrier**

```bash
fish --no-execute fish/conf.d/aliases.fish fish/conf.d/git-abbreviations.fish
fish_indent --check fish/conf.d/aliases.fish fish/conf.d/git-abbreviations.fish
fish --no-config -c 'source fish/conf.d/git-abbreviations.fish; abbr --show' | rg -Fx "abbr -a -- gp 'git push'"
bash -n scripts/jdl scripts/git_main_branch scripts/review
make -n -B cursor | tee /tmp/issue-64-cursor-plan
rg -F 'curl https://cursor.com/install -fsS | bash' /tmp/issue-64-cursor-plan
rg -F '.agents/skills/merge-verdict' /tmp/issue-64-cursor-plan
! rg 'Cursor\.app|brew install --cask cursor|\.config/Cursor/User|\.dotfiles/cursor/' /tmp/issue-64-cursor-plan
make -n -B aws | rg -F 'brew install awscli'
test -x scripts/git_main_branch
test "$(readlink .cursor/skills)" = '../.agents/skills'
! git grep -nE 'cheque-parser-instance-reboot|import-instance-(start|stop)|git_hook_(push|assert_typos|assert_todoes|assert_eslint|assert_empty_files|detect_copy_paste)' -- . ':(exclude)docs/superpowers/specs/2026-08-13-public-dotfiles-cleanup-design.md' ':(exclude)docs/superpowers/plans/2026-08-13-public-dotfiles-cleanup.md'
! git grep -nE 'Bellman|Bigfoot|Septeo|Quadral|Cheque Parser|Import Server|services/(api-graphql|lobby)' -- . ':(exclude,glob).agents/skills/skill-manager/**' ':(exclude)docs/superpowers/specs/2026-08-13-public-dotfiles-cleanup-design.md' ':(exclude)docs/superpowers/plans/2026-08-13-public-dotfiles-cleanup.md'
git diff --check
```

Expected: all checks pass on local macOS. `shellcheck` is absent locally, so the tracked-script CI
gate remains pending until push; do not claim it green locally. Prettier is already RED on several
commit-1 Markdown files and is not a repository gate, so do not reformat unrelated prose or claim a
Markdown-format barrier here.

- [ ] **Step 9: Commit the repository cleanup**

```bash
git add Makefile fish/conf.d/aliases.fish fish/conf.d/git-abbreviations.fish dict/user.txt scripts cursor docs/adr .agents/skills/apple-notes/SKILL.md .agents/skills/johnny-decimal/SKILL.md .agents/skills/scripts/SKILL.md
git diff --cached --check
git commit -m "chore: remove obsolete professional config"
```

Expected: one implementation commit containing only commit-1 paths.

## Task 2: Rebuild `skill-manager` with a failing baseline

**Files:** all commit-2 files from the file map.

- [ ] **Step 1: Capture RED mechanical checks before editing**

Run this exact block and retain the failures in the task log:

```bash
rg -q 'allowed-tools' .agents/skills/skill-manager/references/conventions.md
rg -q 'allowed-tools' .agents/skills/skill-manager/references/doctor.md
! rg -n '\| `Rules`|Missing `Rules`' .agents/skills/skill-manager/references/doctor.md
! rg -n '/rules|rules needed' .agents/skills/skill-manager/references/create.md
! rg -n 'author: Bellman|Lobby|api-graphql|prisma-(pm|lm)|integration-(pm|lm)' .agents/skills/skill-manager
rg -q 'explicitly requested functional evolution' .agents/skills/skill-manager/references/fix.md
test -f .agents/skills/skill-manager/evals/trigger-queries.json
prettier --check .agents/skills/skill-manager/SKILL.md .agents/skills/skill-manager/references/*.md .agents/skills/README.md
```

Expected: every assertion is RED; Prettier identifies at least `cross-check.md` and
`.agents/skills/README.md`. Do not edit before observing the failures.

- [ ] **Step 2: Run behavioral RED scenarios against the current skill**

Dispatch one fresh-context agent per prompt, allowing it to read only
`.agents/skills/skill-manager/`; record each answer verbatim:

```text
Scenario A: Doctor a skill whose frontmatter contains allowed-tools: Read Grep. Is that field valid?
Scenario B: Scaffold a project skill that needs references but no executable resource. List every directory and frontmatter field you create.
Scenario C: A user explicitly requests a behavior change to a skill that currently passes doctor and has no cross-check finding. May fix apply it?
Scenario D: Run sync-index twice over identical frontmatter. Specify exactly how a long folded description is reduced and whether the second output is byte-identical.
```

Expected current failures: A rejects or omits `allowed-tools`; B offers `rules/` and
`author: Bellman`; C refuses the requested evolution; D cannot guarantee byte identity because
`~100 characters` is undefined.

- [ ] **Step 3: Rewrite `SKILL.md` around the retained architecture**

Use this exact frontmatter:

```yaml
---
name: skill-manager
description: >
  Manage .agents/skills: create, doctor, fix, cross-check, and sync the README index. Use when
  creating or changing any skill, including requested behavior changes to a passing skill. Make
  sure to use this skill whenever editing SKILL.md, validating portability, resolving inter-skill
  conflicts, or rebuilding the skills index, even for one frontmatter field; read
  references/conventions.md first.
metadata:
  category: ops
---
```

Keep all five operations. State that `.agents/skills/` is canonical and `.claude/skills`,
`.cursor/skills`, and `.codex/skills` are relative adapters. List all six standard fields including
experimental `allowed-tools`; route literal positional-placeholder details to `conventions.md`;
forbid implicit `skills-ref` installation; preserve `cross-check` as read-only and run `sync-index`
after create/fix/rename/delete.

- [ ] **Step 4: Rebuild `conventions.md` without Brain-specific architecture**

Implement these exact normative rules:

```text
Standard:
- name: string, 1–64 lowercase letters/digits/hyphens, no edge hyphen or --, equals directory
- description: non-empty string, at most 1024 characters; local target under 400
- license: optional string
- compatibility: optional non-empty string, at most 500 characters
- allowed-tools: optional space-separated string, experimental
- metadata: optional string-to-string map; quote version-like values

Local:
- metadata.category is required and one of dev/support/product/ops
- H1, Overview, Usage, Steps or Workflow, Gotchas, Constraints are required in that order
- Gotchas and Constraints each contain at least three entries
- .agents/skills is canonical; adapters are links to it, never copies to synchronize
- resources created on demand are references, scripts, assets, evals
- executable shell with positional argument placeholders belongs in scripts; literal prose escapes the token
- an activation router is absent by default and requires repeated measured misrouting
- README is deterministic and derived from frontmatter
```

Use only `services/api`, `services/worker`, `integration-api.md`, and `integration-worker.md` as
routing examples. Remove every organization name and `metadata.author` example. Preserve the local
category vocabulary because `sync-index` depends on it.

- [ ] **Step 5: Make `create.md` emit only valid local structure**

Create `.agents/skills/<slug>/` plus only the requested subset of `references/`, `scripts/`,
`assets/`, and `evals/`; delete `rules/`. The minimal template emits only:

```yaml
---
name: <slug>
description: >
  <specific trigger description using the local activation pattern>
metadata:
  category: dev | support | product | ops
---
```

Add `license`, `compatibility`, and `allowed-tools` only with an established valid value. End
creation with conditional `skills-ref validate`, local doctor, eval JSON checks when present, and
deterministic `sync-index`.

- [ ] **Step 6: Make `doctor.md` authoritative and coherent**

Define status exactly:

```text
FAIL: a standard rule or mandatory local convention fails
WARN: only a qualitative, non-blocking weakness exists
PASS: no FAIL or WARN; unavailable optional tooling is reported separately
```

Check all six fields and their types/limits, local category, ordered sections, `Constraints` rather
than `Rules`, three gotchas, three constraints, size, progressive disclosure, conditional-reference
routing, positional placeholders, README membership, and optional eval JSON. For evals, require the
directory slug, string version, non-empty typed queries, and positive plus negative cases. Report
`Standard validation: PASS | FAIL | unavailable` and `Local conventions: PASS | WARN | FAIL`.
Never fail an absent activation router.

- [ ] **Step 7: Make `fix.md` accept only two justified inputs**

Allow a doctor/cross-check finding or an explicitly requested functional evolution. Use this exact
constraint:

```text
A PASS skill may change only for an explicitly requested functional evolution. Without such a
request, a doctor or cross-check finding is required before any automatic edit.
```

Never move an arbitrary custom field into `metadata`; preserve it there only when it is scalar
metadata and the user explicitly approves. Move executable positional shell examples into
`scripts/` or rewrite without positional placeholders. Finish with doctor, relevant evals,
baseline/contract comparison, and `sync-index`.

- [ ] **Step 8: Keep `cross-check.md` generic and strictly read-only**

Retain D1–D6 thresholds and stop-after-report. Replace organization scopes with `api`, `worker`,
`frontend`, and `backend`; make sibling files the sole D6 classification gate; call D6 conditional
reference routing. D4 treats `## Constraints` as the valid SKILL section. State that a missing
activation router is never a finding. Format its severity table with Prettier.

- [ ] **Step 9: Make `sync-index.md` deterministic**

Specify this algorithm:

```text
1. Read name, folded description, and metadata.category from every SKILL.md.
2. Order Dev, Support, Product, Ops, Uncategorized; omit empty sections.
3. Sort slugs bytewise within each section.
4. Normalize folded-description whitespace and take its first sentence without arbitrary truncation.
5. Escape Markdown table pipes and let Prettier determine column width.
6. Keep invalid skills under Uncategorized and report them as doctor failures.
7. Given identical inputs, write byte-identical README output and verify with a second run.
```

Use only `example-skill` and `another-skill` in its template. Rename `## Rules` to
`## Constraints`, and list `evals/` among optional directories.

- [ ] **Step 10: Replace the imaginary eval runner with truthful scenarios**

In `references/evals.md`, remove Phase-4 status and claims of an existing five-agent runner.
Document the doctor schema and the activation-router experiment: run identical queries at least
three times without a rule, require reproduced misrouting, then compare with the proposed rule.
State that no runner exists and no multi-agent validation may be claimed without actual runs.

Create `.agents/skills/skill-manager/evals/trigger-queries.json` with exactly:

```json
{
  "skill": "skill-manager",
  "version": "1.0",
  "queries": [
    {
      "query": "Create a new project skill for release checks",
      "should_activate": true,
      "reason": "Creating a skill is a skill-manager operation"
    },
    {
      "query": "Change one frontmatter field in the scripts skill",
      "should_activate": true,
      "reason": "Any SKILL.md edit must use skill-manager"
    },
    {
      "query": "Evolve the behavior of a skill that already passes doctor",
      "should_activate": true,
      "reason": "Explicit functional evolution is handled by fix"
    },
    {
      "query": "Doctor every project skill",
      "should_activate": true,
      "reason": "Global skill quality audit is a doctor operation"
    },
    {
      "query": "Cross-check the skills for trigger conflicts",
      "should_activate": true,
      "reason": "Inter-skill conflict analysis is a cross-check operation"
    },
    {
      "query": "Rebuild the shared skills README",
      "should_activate": true,
      "reason": "The skills index is maintained by sync-index"
    },
    {
      "query": "Review whether this pull request is safe to merge",
      "should_activate": false,
      "reason": "A normal pull request review belongs to merge-verdict"
    },
    {
      "query": "Correct a typo in the repository README",
      "should_activate": false,
      "reason": "A generic README edit is not skill management"
    }
  ]
}
```

- [ ] **Step 11: Run the same scenarios GREEN**

Repeat Step 2 with fresh agents. Expected: A accepts experimental `allowed-tools`; B creates no
`rules/` or author; C permits the requested evolution with a baseline; D defines first-sentence
normalization and byte identity. If one fails, revise only the instruction that allowed it and
rerun all four.

- [ ] **Step 12: Validate and regenerate the index twice**

Invoke `/skill-manager doctor skill-manager`; expected local `PASS`. Run `skills-ref validate
./.agents/skills/skill-manager` only if available; otherwise record
`Standard validation: unavailable (skills-ref not installed)` without installing it. Validate evals:

```bash
jq -e '.skill == "skill-manager" and (.version | type == "string") and (.queries | type == "array" and length > 0) and (all(.queries[]; (.query | type == "string") and (.should_activate | type == "boolean") and (.reason | type == "string"))) and (any(.queries[]; .should_activate == true)) and (any(.queries[]; .should_activate == false))' .agents/skills/skill-manager/evals/trigger-queries.json
```

Invoke `/skill-manager sync-index`, format its output, hash it, invoke sync again, and compare:

```bash
prettier --write .agents/skills/README.md
shasum -a 256 .agents/skills/README.md > /tmp/issue-64-skills-index.sha256
```

Invoke `/skill-manager sync-index` a second time, then run:

```bash
shasum -a 256 -c /tmp/issue-64-skills-index.sha256
```

Expected: the checksum passes.

- [ ] **Step 13: Run the commit-2 verification barrier**

Rerun every RED assertion from Step 1; all must pass. Then run:

```bash
prettier --check .agents/skills/skill-manager/SKILL.md .agents/skills/skill-manager/references/*.md .agents/skills/README.md
jq -e empty .agents/skills/skill-manager/evals/trigger-queries.json
test "$(readlink .claude/skills)" = '../.agents/skills'
test "$(readlink .cursor/skills)" = '../.agents/skills'
test "$(readlink .codex/skills)" = '../.agents/skills'
! rg -n 'Bellman|Lobby|api-graphql|prisma-(pm|lm)|integration-(pm|lm)|property-management' .agents/skills/skill-manager
git diff --check
```

Expected: all checks pass and no adapter changes direction.

- [ ] **Step 14: Commit the reconstructed skill**

```bash
git add .agents/skills/skill-manager .agents/skills/README.md
git diff --cached --check
git commit -m "refactor: rebuild skill manager"
```

Expected: one implementation commit containing only commit-2 paths.

## Task 3: Doctor, fix, and cross-check every skill

**Files:** findings-driven commit-3 files from the file map.

- [ ] **Step 1: Run the global doctor baseline**

Invoke `/skill-manager doctor` with no slug across all twelve skill directories. Expected objective
baseline: frontmatter, names, categories, sections, sizes, and README membership pass; the only
currently reproducible mandatory finding is the positional shell example in
`do-nothing-script/SKILL.md`. `skills-ref` absence is an environment limitation, not a FAIL.

Save the report in the task log, not the repository. If another FAIL appears, verify it against
`conventions.md`; if correction requires behavioral judgment, stop and ask the user.

- [ ] **Step 2: Apply the known finding through `/skill-manager fix do-nothing-script`**

Record the baseline, then replace only the example's main block with:

```bash
main() {
  VERSION="${VERSION:?usage: VERSION=1.2.3 release}"
  step_create_branch
  step_tag
  echo "Release ${VERSION} done."
}

main
```

This preserves the example while removing positional placeholders from the templated body. Invoke
`/skill-manager doctor do-nothing-script`; expected `PASS` with no regression.

- [ ] **Step 3: Fix additional objective doctor findings one skill at a time**

For each additional confirmed FAIL, invoke `/skill-manager fix <slug>`, record its baseline, change
only the named rule, rerun `/skill-manager doctor <slug>`, and keep all prior PASS checks. Do not
treat Prettier or cspell output as a doctor finding unless doctor explicitly defines it. Ask before
changing behavior or triggering for a qualitative WARN.

- [ ] **Step 4: Reach a clean global doctor report**

Invoke `/skill-manager doctor` again. Expected: twelve `PASS` statuses and, if `skills-ref` remains
absent, one shared environment limitation. Do not run cross-check while malformed descriptions or
sections remain, because they inflate D1/D4 noise.

- [ ] **Step 5: Run the read-only cross-check checkpoint**

Invoke `/skill-manager cross-check`, present its complete report, and stop that operation without
writing. Current measured expectation: no D1 overlap at 40%, no D5 distance at 2, no D6 scoped
group, no unresolved D3 reference, and no substantive D2/D4 finding.

If the report is empty, continue. If it reports a heuristic or ambiguous finding, obtain explicit
approval before `/skill-manager fix <slug>`. An objective dead reference or missing conditional
reference routing may be fixed in a new operation, followed by the target doctor.

- [ ] **Step 6: Re-run doctor and cross-check after approved fixes**

Invoke `/skill-manager doctor`, then `/skill-manager cross-check`. Expected: twelve PASS skills and
no critical/warning cross-skill finding. Informational D5 output is not a cleanliness failure unless
the user approves a rename.

- [ ] **Step 7: Run deterministic `sync-index` twice**

Invoke `/skill-manager sync-index`, run Prettier on `.agents/skills/README.md`, and capture:

```bash
shasum -a 256 .agents/skills/README.md > /tmp/issue-64-skills-index.sha256
```

Invoke `/skill-manager sync-index` a second time, then run:

```bash
shasum -a 256 -c /tmp/issue-64-skills-index.sha256
```

Expected: byte-identical output.

- [ ] **Step 8: Run the final skill barrier**

```bash
prettier --check '.agents/skills/**/*.md' .agents/skills/README.md
find .agents/skills -path '*/evals/*.json' -print0 | xargs -0 -n1 jq -e empty
if command -v skills-ref >/dev/null 2>&1; then
  for skill in .agents/skills/*/; do skills-ref validate "$skill"; done
else
  printf '%s\n' 'Standard validation: unavailable (skills-ref not installed)'
fi
git diff --check
git status --short
```

Expected: JSON and changed-skill Markdown validation pass. If the global Prettier command still
names pre-existing files not reported by doctor, keep them outside commit 3 and report that barrier
gap instead of claiming global Markdown formatting green.

- [ ] **Step 9: Commit only reproducible skill findings**

```bash
git add .agents/skills
git diff --cached --check
git diff --cached --name-only
git commit -m "chore: fix project skill findings"
```

Expected minimum diff: `.agents/skills/do-nothing-script/SKILL.md`; every other path must be named by
a doctor/approved cross-check finding or be the regenerated README.

## Task 4: Verify the complete branch and prepare issue closure

**Files:** no new implementation files.

- [ ] **Step 1: Run repository-wide relevant barriers**

```bash
git ls-files 'fish/*.fish' 'fish/**/*.fish' | xargs -I{} fish --no-execute {}
for file in $(git ls-files 'fish/*.fish' 'fish/**/*.fish'); do fish_indent --check "$file"; done
found_by_shebang=$(git grep -lI -E '^#!.*(bash|sh)' -- scripts install.sh)
found_by_extension=$(git ls-files '*.sh')
tracked_scripts=$(printf '%s\n%s\n' "$found_by_shebang" "$found_by_extension" | sort -u)
printf '%s\n' "$tracked_scripts" | grep -qx scripts/upgrade
printf '%s\n' "$tracked_scripts" | grep -qx install.sh
if command -v shellcheck >/dev/null 2>&1; then printf '%s\n' "$tracked_scripts" | xargs -I{} shellcheck --severity=error {}; else printf '%s\n' 'ShellCheck: unavailable locally; macOS CI pending'; fi
make -n -B cursor
git diff --check main...HEAD
git status --short --branch
```

Expected: Fish covers every tracked Fish file on local macOS; ShellCheck either covers all tracked
shell extensions or is explicitly pending in macOS CI; Cursor dry-run contains no IDE target; diff
check passes; worktree is clean. Do not run `make all`, which performs real installations.

- [ ] **Step 2: Audit retained and removed surfaces**

```bash
test -f dict/user.txt
test ! -s dict/user.txt
test -x scripts/git_main_branch
test -L .cursor/skills
rg -n '^aws:|^[[:space:]]*aws \\' Makefile
rg -n "abbr -a gp 'git push'" fish/conf.d/git-abbreviations.fish
! test -d cursor
! test -e docs/adr/020-hooks-de-push-maison.md
! git grep -nE 'cheque-parser-instance-reboot|import-instance-(start|stop)|git_hook_(push|assert_typos|assert_todoes|assert_eslint|assert_empty_files|detect_copy_paste)|Bellman|Bigfoot|Septeo|Import Quadral' -- . ':(exclude)docs/superpowers/specs/2026-08-13-public-dotfiles-cleanup-design.md' ':(exclude)docs/superpowers/plans/2026-08-13-public-dotfiles-cleanup.md'
```

Expected: retained CLI/adapters exist; removed surfaces and names do not.

- [ ] **Step 3: Review the three implementation commits**

```bash
git log --oneline --decorate main..HEAD
git show --stat --oneline HEAD~2
git show --stat --oneline HEAD~1
git show --stat --oneline HEAD
```

Expected implementation order: repository cleanup, `skill-manager` reconstruction, global skill
findings. Design and plan commits precede them and are not counted as implementation commits.

- [ ] **Step 4: Defer external issue writes until integration**

After integration, comment on #54 that its AWS and push-hook scope was completed by #64, then close
#54. Close #64 with the three implementation commit IDs and exact local/CI environments. Do not
close either issue from an unmerged branch.

## Delivery constraints

- Add no source-code or configuration comment; expected delivery comment list is empty.
- Do not rewrite Git history.
- Do not install `skills-ref` or any package implicitly.
- Do not run an actual install target locally; use `make -n -B cursor` only.
- Name local evidence as macOS/Darwin; CI macOS evidence is separate and available only after push.
  Make no Linux claim without a Linux run.
