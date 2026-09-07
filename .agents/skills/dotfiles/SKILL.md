---
name: dotfiles
description: >
  Apply this repository's conventions for configuration, symlinks, platform differences, and tool
  installation. Use when editing the Makefile or any file managed from here. Make sure to use it
  whenever a change lands in this repository, even if the request names a single application.
metadata:
  category: ops
---

# Dotfiles Guidelines

## Overview

Apply this repository's conventions when changing managed configuration, symlinks, or tool
installation. Moon owns migrated installation and development tasks; Make remains only for
optional operations, existing cleanup and compatibility entry points during the transition.

## Usage

Use this skill before editing managed configuration or changing how a tool is installed.
Examples: add a managed CLI, deploy a home configuration, or gate a platform-specific setting.

## Steps

1. Inspect the relevant ADRs, Moon task and dependencies, remaining Make consumer, and both the
   source and destination of every affected deployment.
2. Put a migrated installation directly in Moon, with its actual prerequisites. Use the existing
   task group or owning project; never call Make from a migrated Moon task.
3. Keep a non-migrated Homebrew package in its profile Brewfile. When its installation becomes an
   autonomous Moon task, remove the formula from the Brewfile and use a native Homebrew probe.
4. Keep sources under the existing home, harness or tooling boundary and add their deployment to
   the owning Moon project. Preserve the source-to-destination symlink pattern and the explicit
   assembled or copied artifacts described by the ADRs.
5. Preserve correct destinations silently and reject divergent links or files without overwriting
   them. Keep behavior specific to Fisher, Git and assembled instructions within its existing
   tested utility; do not extend it into a general repair mechanism.
6. Keep installation of the workstation macOS-only and preserve the existing portable checks.
   Homebrew mutations share the homebrew mutex; tests must not assume a runtime exists merely
   because one CI runner currently provides it.
7. In a worktree, inspect the task and its action graph before execution. Execute only deployment
   tasks proven to mutate fixture-local links or configuration, without their global installation
   dependencies; the macOS CI smoke exercises the full installation graph. For remaining Make
   installations, use their dry-run for inspection.
8. Validate declarative changes with Moon's native configuration and action graph, then run the
   relevant behavioral oracle. Do not add tests that parse task declarations or copy inventories.

## Gotchas

- **Running a global package manager directly** — bypasses reproducible setup; put its command
  in the canonical Moon task or the remaining profile declaration.
- **Treating isolated HOME as an install sandbox** — Homebrew and application installers can
  mutate global state; exclude those dependencies from fixture execution and use the CI runner
  for the full profile.
- **Leaving a formula in two places** — a Brewfile and a standalone task become competing
  installation declarations; remove the migrated entry and preserve the native package probe.
- **Linking inside an existing directory** — changes an unexpected destination; use the deployment
  helper's collision refusal and inspect any divergent state before a separate reconstruction.
- **Restoring a Make prerequisite on a migrated path** — recreates the second graph; update the
  Moon dependency and let any remaining Make adapter delegate to it.

## Constraints

- Write documentation in English except under docs/, whose instructions require French.
- Never run a global package-manager installation outside the repository's declared task.
- Never delete or overwrite a divergent symlink destination as an implicit installation repair.
- Never run a task with global installation side effects from a worktree or fixture HOME.
- Preserve macOS installation boundaries and every existing Linux check affected by the change.
- Keep package inventories canonical and use native oracles instead of declaration mirror tests.
