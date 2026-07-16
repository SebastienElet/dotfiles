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

# Neovim (LazyVim)

## Overview

Maintain the repository's Lua-based Neovim configuration under `nvim/lua/`. Use this skill for
LazyVim plugin specifications, custom options, keymaps, and other changes to Neovim behavior.

## Usage

Invoke this skill before editing Neovim configuration:

```text
/neovim
```

Examples:

- "Add and configure a LazyVim plugin with a keymap."
- "Disable swap files and change relative line numbers."

## Steps

1. Inspect the existing files under `nvim/lua/` and identify the smallest configuration change.
2. For a plugin, create or update `nvim/lua/plugins/<plugin>.lua`, keeping one plugin per file and
   using a minimal LazyVim plugin specification with only the required options.
3. Put custom editor options in `nvim/lua/config/options.lua`.
4. Put all custom mappings in `nvim/lua/config/keymaps.lua`, including mappings that invoke a
   plugin; do not embed `keys` in the plugin specification.
5. Format the changed Lua files and verify that Neovim starts without configuration errors.

## Gotchas

- **Using a conventional dotfiles path such as `nvim/.config/nvim/`** — this repository stores
  Neovim Lua directly under `nvim/lua/`; use the existing repository layout.
- **Embedding a plugin keymap in its `keys` field** — this scatters custom mappings across plugin
  specifications; place the mapping in `nvim/lua/config/keymaps.lua`.
- **Adding several plugin specifications to one file** — this makes plugin ownership unclear; keep
  one plugin per `nvim/lua/plugins/<plugin>.lua` file.
- **Copying a full upstream configuration** — unnecessary defaults make upgrades harder; keep the
  LazyVim specification and `opts` table minimal.

## Constraints

- All Neovim configuration must remain Lua under `nvim/lua/`.
- Each plugin must have its own file under `nvim/lua/plugins/`.
- Custom options must be placed in `nvim/lua/config/options.lua`.
- Custom keymaps must be placed in `nvim/lua/config/keymaps.lua`.
- Plugin specifications must use LazyVim conventions and include only necessary configuration.
