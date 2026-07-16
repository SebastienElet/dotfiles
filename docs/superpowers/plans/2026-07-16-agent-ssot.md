# Agent SSOT Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make `AGENTS.md` and `.agents/skills/` canonical while replacing legacy Cursor content with thin, relative agent adapters.

**Architecture:** Repository-wide rules live only in `AGENTS.md`, and reusable skills live only in `.agents/skills/`. Claude, Codex, and Cursor receive the same skills through relative symlinks, while Claude receives repository rules through a root `CLAUDE.md` symlink.

**Tech Stack:** Markdown, agentskills.io skill format, POSIX symbolic links, Git

---

## File Map

- Modify `AGENTS.md`: make the document tool-agnostic and point to the canonical skills index.
- Create `CLAUDE.md`: relative symlink to `AGENTS.md`.
- Create `.claude/skills`: relative symlink to `../.agents/skills`.
- Create `.codex/skills`: relative symlink to `../.agents/skills`.
- Replace `.cursor/` contents with `.cursor/skills`: relative symlink to `../.agents/skills`.
- Move and fix `.cursor/skills/{dotfiles,johnny-decimal,neovim,scripts}` under `.agents/skills/`.
- Delete `.cursor/commands/` and obsolete Cursor skills.
- Modify `.agents/skills/README.md`: synchronize the canonical skills index.
- Delete the temporary design and implementation-plan documents before the final PR update.

### Task 1: Centralize the Selected Skills

**Files:**

- Move: `.cursor/skills/dotfiles/` → `.agents/skills/dotfiles/`
- Move: `.cursor/skills/johnny-decimal/` → `.agents/skills/johnny-decimal/`
- Move: `.cursor/skills/neovim/` → `.agents/skills/neovim/`
- Move: `.cursor/skills/scripts/` → `.agents/skills/scripts/`
- Delete: `.cursor/skills/cli-tools/`
- Delete: `.cursor/skills/commits/`
- Delete: `.cursor/skills/fish/`
- Delete: `.cursor/skills/makefile/`
- Delete: `.cursor/commands/`

- [ ] **Step 1: Verify the legacy layout**

Run:

```bash
for skill in dotfiles johnny-decimal neovim scripts; do
	test -f ".cursor/skills/$skill/SKILL.md"
	test ! -e ".agents/skills/$skill"
done
test -d .cursor/commands
```

Expected: PASS.

- [ ] **Step 2: Move the four retained skills**

Run:

```bash
for skill in dotfiles johnny-decimal neovim scripts; do
	git mv ".cursor/skills/$skill" ".agents/skills/$skill"
done
```

- [ ] **Step 3: Delete obsolete Cursor content**

Run:

```bash
git rm -r \
	.cursor/commands \
	.cursor/skills/cli-tools \
	.cursor/skills/commits \
	.cursor/skills/fish \
	.cursor/skills/makefile
```

- [ ] **Step 4: Verify the migration scope**

Run:

```bash
for skill in dotfiles johnny-decimal neovim scripts skill-manager things-tasks; do
	test -f ".agents/skills/$skill/SKILL.md"
done
test ! -e .cursor/commands
test ! -e .cursor/skills/cli-tools
test ! -e .cursor/skills/commits
test ! -e .cursor/skills/fish
test ! -e .cursor/skills/makefile
git diff --check
```

Expected: PASS.

- [ ] **Step 5: Commit the structural migration**

```bash
git add .agents/skills .cursor
git commit -m "refactor(agents): centralize shared skills"
```

### Task 2: Run Skill Manager Baselines

**Files:**

- Inspect: `.agents/skills/dotfiles/SKILL.md`
- Inspect: `.agents/skills/johnny-decimal/SKILL.md`
- Inspect: `.agents/skills/neovim/SKILL.md`
- Inspect: `.agents/skills/scripts/SKILL.md`
- Read: `.agents/skills/skill-manager/references/conventions.md`
- Read: `.agents/skills/skill-manager/references/doctor.md`
- Read: `.agents/skills/skill-manager/references/fix.md`

- [ ] **Step 1: Read the mandatory conventions**

Read the three skill-manager references listed above completely before editing
any migrated `SKILL.md`.

- [ ] **Step 2: Run doctor on each migrated skill**

Invoke sequentially:

```text
/skill-manager doctor dotfiles
/skill-manager doctor johnny-decimal
/skill-manager doctor neovim
/skill-manager doctor scripts
```

Expected baseline findings for each skill:

- missing `metadata.category`;
- weak, non-pushy description;
- missing required `Overview`, `Usage`, `Steps`, `Gotchas`, or `Constraints` sections;
- obsolete Cursor-specific `metadata.globs`;
- README index entry missing.

- [ ] **Step 3: Record the baseline reports**

Keep each doctor report available for comparison during its fix. Do not edit a
skill before its report is complete.

### Task 3: Fix the Migrated Skills

**Files:**

- Modify: `.agents/skills/dotfiles/SKILL.md`
- Modify: `.agents/skills/johnny-decimal/SKILL.md`
- Modify: `.agents/skills/neovim/SKILL.md`
- Modify: `.agents/skills/scripts/SKILL.md`
- Modify: `.agents/skills/README.md`

- [ ] **Step 1: Fix `dotfiles`**

Invoke:

```text
/skill-manager fix dotfiles
```

Required outcome:

```yaml
---
name: dotfiles
description: >
  Apply this dotfiles repository's configuration and installation conventions. Use when changing
  dotfiles, symlinks, platform-specific configuration, or tool installation. Make sure to use this
  skill whenever editing the Makefile or a configuration managed from this repository, even if the
  request only mentions a single application.
metadata:
  category: ops
---
```

The fixed skill must:

- preserve English-only, minimal configuration, portability, Makefile installation, symlink, and
  comment-style rules;
- replace the dead `makefile.md` reference with the repository `Makefile`;
- remove the obsolete claim that Cursor configuration is canonical;
- include the required `Overview`, `Usage`, `Steps`, `Gotchas` with at least three entries, and
  `Constraints` with at least three hard rules.

Re-run:

```text
/skill-manager doctor dotfiles
```

Expected: PASS with no regression from the baseline.

- [ ] **Step 2: Fix `johnny-decimal`**

Invoke:

```text
/skill-manager fix johnny-decimal
```

Required outcome:

```yaml
---
name: johnny-decimal
description: >
  Organize files in ~/Documents with the Johnny Decimal and PARA hybrid system. Use when naming,
  filing, moving, or locating numbered personal documents. Make sure to use this skill whenever a
  request concerns Johnny Decimal categories, PARA mapping, or document filenames under
  ~/Documents, even if the user only asks where a document belongs.
metadata:
  category: ops
---
```

The fixed skill must:

- explicitly scope every filesystem rule to `~/Documents`;
- preserve the existing numbering, PARA mapping, naming rules, and Apple Notes index;
- state that it does not govern `~/Brain`;
- avoid requiring a named MCP tool that may not be installed;
- include all required sections and at least three project-specific gotchas and constraints.

Re-run:

```text
/skill-manager doctor johnny-decimal
```

Expected: PASS with no regression from the baseline.

- [ ] **Step 3: Fix `neovim`**

Invoke:

```text
/skill-manager fix neovim
```

Required outcome:

```yaml
---
name: neovim
description: >
  Maintain this repository's Neovim and LazyVim configuration. Use when editing Lua files under
  nvim/, adding plugins, changing options, or updating keymaps. Make sure to use this skill whenever
  a request affects LazyVim plugin specifications or Neovim behavior, even if Neovim is not named
  explicitly.
metadata:
  category: dev
---
```

The fixed skill must:

- preserve the existing Lua paths, minimal plugin specification, option, keymap, and one-plugin-per-file rules;
- remove `metadata.globs`;
- express path activation in the description and steps;
- include all required sections and at least three project-specific gotchas and constraints.

Re-run:

```text
/skill-manager doctor neovim
```

Expected: PASS with no regression from the baseline.

- [ ] **Step 4: Fix `scripts`**

Invoke:

```text
/skill-manager fix scripts
```

Required outcome:

```yaml
---
name: scripts
description: >
  Create and maintain portable Bash scripts in this repository. Use when adding or editing
  standalone scripts, shebangs, error handling, or executable tooling. Make sure to use this skill
  whenever a change touches scripts/ or introduces shell automation, even if the request calls it a
  helper or hook.
metadata:
  category: dev
---
```

The fixed skill must:

- preserve Bash, extensionless executable, descriptive naming, error handling, macOS/Linux
  portability, and `/usr/bin/env bash` rules;
- remove `metadata.globs`;
- include all required sections and at least three project-specific gotchas and constraints.

Re-run:

```text
/skill-manager doctor scripts
```

Expected: PASS with no regression from the baseline.

- [ ] **Step 5: Synchronize and verify the index**

Invoke:

```text
/skill-manager sync-index
```

Expected index groups:

- `Dev`: `neovim`, `scripts`
- `Ops`: `dotfiles`, `johnny-decimal`, `skill-manager`, `things-tasks`

Run:

```bash
git diff --check
git diff -- .agents/skills
```

Expected: only the four fixes and synchronized index.

- [ ] **Step 6: Commit the skill fixes**

```bash
git add .agents/skills
git commit -m "refactor(agents): fix migrated skills"
```

### Task 4: Create Agent Adapters and Rewrite AGENTS.md

**Files:**

- Modify: `AGENTS.md`
- Create: `CLAUDE.md` symlink
- Create: `.claude/skills` symlink
- Create: `.codex/skills` symlink
- Create: `.cursor/skills` symlink

- [ ] **Step 1: Verify adapters are absent**

Run:

```bash
test ! -e CLAUDE.md
test ! -e .claude/skills
test ! -e .codex/skills
test ! -L .cursor/skills
test ! -e .cursor/skills || test -d .cursor/skills
```

Expected: PASS.

- [ ] **Step 2: Replace `AGENTS.md` with the canonical instructions**

Use this exact content:

```markdown
# Dotfiles Agent Instructions

This file is the single source of truth for all coding agents working in this repository.

> **Conflict rule:** if an agent-specific adapter disagrees with this document, `AGENTS.md` wins.

## Scope

- **Prefer small iterations.** Do only what was asked; avoid expanding to all similar items.
- **Keep changes minimal.** Reuse existing structures and ask when the intended scope is ambiguous.
- **Avoid broad refactors.** Do not migrate or reorganize unrelated configuration by default.

## Personal Brain

- Before operating on content under `~/Brain`, read and follow `~/Brain/AGENTS.md`.

## Shared Skills

- `.agents/skills/` is the single source of truth for reusable agent skills.
- Read `.agents/skills/README.md` for the current skill index.
- Use `skill-manager` whenever creating, migrating, editing, validating, or organizing skills.

## Agent Adapters

- `CLAUDE.md` delegates repository instructions to this file.
- `.claude/skills`, `.codex/skills`, and `.cursor/skills` expose the canonical shared skills.
- Agent-specific directories must not duplicate repository-wide rules or skill content.
```

- [ ] **Step 3: Create the relative symlinks**

Run:

```bash
mkdir -p .claude .codex .cursor
if [ -d .cursor/skills ]; then
	test -z "$(find .cursor/skills -mindepth 1 -maxdepth 1 -print -quit)"
	rmdir .cursor/skills
fi
ln -s AGENTS.md CLAUDE.md
ln -s ../.agents/skills .claude/skills
ln -s ../.agents/skills .codex/skills
ln -s ../.agents/skills .cursor/skills
```

- [ ] **Step 4: Verify exact link targets**

Run:

```bash
test "$(readlink CLAUDE.md)" = "AGENTS.md"
for adapter in .claude/skills .codex/skills .cursor/skills; do
	test -L "$adapter"
	test "$(readlink "$adapter")" = "../.agents/skills"
	test -f "$adapter/skill-manager/SKILL.md"
done
```

Expected: PASS.

- [ ] **Step 5: Verify Cursor contains only the adapter**

Run:

```bash
test "$(find .cursor -mindepth 1 -maxdepth 1 | wc -l | tr -d ' ')" = "1"
test -L .cursor/skills
```

Expected: PASS.

- [ ] **Step 6: Commit the canonical instructions and adapters**

```bash
git add AGENTS.md CLAUDE.md .agents .claude .codex .cursor
git diff --check
git commit -m "refactor(agents): add SSOT adapters"
```

### Task 5: Run Global Skill Validation

**Files:**

- Inspect: `.agents/skills/**`
- Inspect: `.agents/skills/README.md`

- [ ] **Step 1: Run doctor across all skills**

Invoke:

```text
/skill-manager doctor
```

Expected: all six skills PASS. If an existing skill fails, report the exact finding before changing
it; do not expand this task without user approval.

- [ ] **Step 2: Run the read-only cross-check**

Invoke:

```text
/skill-manager cross-check
```

Expected: no critical issues. Cross-check is read-only; if it reports a critical issue, stop and ask
which skill should be fixed through `/skill-manager fix <name>`.

- [ ] **Step 3: Verify index parity**

Run:

```bash
expected="$(find .agents/skills -mindepth 1 -maxdepth 1 -type d -exec basename {} \; | sort)"
indexed="$(rg -o '^\\| `[^`]+`' .agents/skills/README.md | sed 's/^| `//; s/`$//' | sort)"
test "$expected" = "$indexed"
```

Expected: PASS with six skills in both lists.

### Task 6: Remove Temporary Planning Documents and Finalize

**Files:**

- Delete: `docs/superpowers/specs/2026-07-16-agent-ssot-design.md`
- Delete: `docs/superpowers/plans/2026-07-16-agent-ssot.md`

- [ ] **Step 1: Delete the internal documents**

```bash
git rm \
	docs/superpowers/specs/2026-07-16-agent-ssot-design.md \
	docs/superpowers/plans/2026-07-16-agent-ssot.md
```

- [ ] **Step 2: Verify the final repository state**

Run:

```bash
test ! -e docs/superpowers/specs/2026-07-16-agent-ssot-design.md
test ! -e docs/superpowers/plans/2026-07-16-agent-ssot.md
test "$(readlink CLAUDE.md)" = "AGENTS.md"
for adapter in .claude/skills .codex/skills .cursor/skills; do
	test "$(readlink "$adapter")" = "../.agents/skills"
done
test "$(find .cursor -mindepth 1 -maxdepth 1 | wc -l | tr -d ' ')" = "1"
git diff --check
```

Expected: PASS.

- [ ] **Step 3: Commit the cleanup**

```bash
git add -A
git commit -m "docs(agents): remove internal planning files"
```

- [ ] **Step 4: Inspect the final diff against `main`**

Run:

```bash
git diff --stat main..HEAD
git diff main..HEAD -- AGENTS.md .agents .claude .codex .cursor CLAUDE.md Makefile
```

Expected: Brain setup remains present; obsolete Cursor content is removed; canonical skills,
instructions, and adapters are present; no internal planning docs remain.

- [ ] **Step 5: Run final checks**

Run:

```bash
git status --short
git diff --check main..HEAD
make brain
test "$(readlink "$HOME/Brain")" = "$HOME/Library/Mobile Documents/com~apple~CloudDocs/Brain"
```

Expected: clean worktree, no whitespace errors, idempotent Brain link.
