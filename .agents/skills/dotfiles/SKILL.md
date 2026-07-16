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

- Add a global CLI through a `Makefile` target.
- Add an application configuration and its symlink.
- Gate an unavoidable platform-specific setting.

## Steps

1. Inspect the relevant configuration and the repository `Makefile` for existing patterns.
2. Make the smallest configuration change that satisfies the request.
3. For tool installation, add or update a `Makefile` target; never run a package manager directly.
4. For a managed configuration, keep its source in this repository, symlink the expected path under
   the user's home directory to that source, and add or update the symlink in the `Makefile`.
5. Prefer settings that work on macOS and Linux. Add an explicit platform check only when a portable
   alternative is unavailable.
6. Verify the changed configuration and its related `Makefile` target without installing unrelated
   tools or changing unrelated files.

## Gotchas

- **Installing directly under time pressure** — commands such as `brew install`, `npm install -g`,
  or equivalent bypass the repository's reproducible setup. Add or use the canonical `Makefile`
  target instead, even when the request explicitly asks for a direct install.
- **Adding a config without its symlink** — the file remains unmanaged or requires manual setup.
  Store the config in this repository and update the `Makefile` with the corresponding symlink.
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
- Every configuration managed from this repository must use the established symlink pattern, with
  its symlink created or updated through the `Makefile`.
