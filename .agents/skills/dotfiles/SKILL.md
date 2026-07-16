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

# Dotfiles Guidelines

## Overview

Apply this repository's conventions when changing managed configuration, symlinks, or tool
installation. Keep changes minimal, portable, and integrated through the repository's `Makefile`.

## Usage

Use this skill before editing a configuration managed by this repository or changing how a tool is
installed.

Examples:

- `$dotfiles add a managed CLI`
- `$dotfiles link a home configuration`
- `$dotfiles gate a platform-specific setting`

## Steps

1. Inspect the relevant configuration, the repository `Makefile`, and both the symlink source and
   destination for existing files, directories, or links.
2. If a requested direct package installation conflicts with this repository's convention, surface
   the conflict, propose the minimal `Makefile` target or change, and obtain direction before making
   a broader repository mutation. Do not execute the direct package-manager command.
3. Make the smallest approved configuration change that satisfies the request.
4. For tool installation, add or update a `Makefile` target; never run a package manager directly.
5. For a home-directory configuration deployed by the `Makefile`, keep its source in this
   repository, symlink the expected path under the user's home directory to that source, and add or
   update the symlink target in the `Makefile`.
6. Prefer settings that work on macOS and Linux. Add an explicit platform check only when a portable
   alternative is unavailable.
7. Verify install and link targets with `make -n <target>` or an isolated `HOME`. Never run a real
   home-mutating target from a temporary worktree because `DOTFILES_PATH` resolves to that checkout;
   run the actual installation from the canonical checkout only when the user's intent authorizes
   it.

## Gotchas

- **Installing directly under time pressure** — commands such as `brew install`, `npm install -g`,
  or equivalent bypass the repository's reproducible setup. Surface the conflict, refuse the direct
  command, propose the minimal `Makefile` change, and obtain direction before changing the
  repository.
- **Replacing an existing destination** — a real file, directory, or link to another source may
  contain user data. Inspect the source and destination first; never delete or overwrite them, and
  ask for direction when the destination is not already the expected link.
- **Running link targets from a temporary worktree** — `DOTFILES_PATH` resolves to the temporary
  checkout and can leave home-directory links pointing at disposable paths. Use `make -n <target>`
  or an isolated `HOME`, then perform the authorized installation from the canonical checkout.
- **Adding a home config without its Makefile symlink** — the file remains unmanaged or requires
  manual setup. Store the config in this repository and update the `Makefile` with the corresponding
  symlink target.
- **Assuming macOS-only behavior** — a configuration can break on Linux hosts. Prefer a portable
  option, or guard the platform-specific behavior with an explicit check such as `uname`.
- **Commenting obvious settings** — redundant comments make configuration harder to scan. Remove
  them and keep only comments that explain a non-obvious reason.

## Constraints

- All documentation and comments must be in English.
- Configuration must stay simple and minimal; comments must explain why, not what.
- macOS is the primary platform, but configurations must remain portable to Linux where practical.
- The repository `Makefile` is canonical for tool installation; never directly run `brew install`,
  `npm install -g`, or another global package-manager install command.
- Never delete or overwrite an existing destination file, directory, or link to an unexpected
  source.
- Never run a real home-mutating install or link target from a temporary worktree; dry-run it or use
  an isolated `HOME`, then run it from the canonical checkout only with appropriate user intent.
- Every home-directory configuration deployed and managed by the `Makefile` must use the established
  repository-source-to-home-destination symlink pattern.
