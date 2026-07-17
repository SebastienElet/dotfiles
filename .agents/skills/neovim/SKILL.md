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

Maintain the Neovim configuration rooted at `nvim/`. The `Makefile` symlinks this directory to the
runtime path `~/.config/nvim`, so edit repository paths rather than the symlink destination.

## Usage

Examples:

- "Add and configure a LazyVim plugin with a keymap."
- "Disable swap files and change relative line numbers."

## Steps

1. Inspect the existing owner and nearby patterns before editing:
   - startup and LazyVim: `nvim/init.lua`, `nvim/lua/config/lazy.lua`, `nvim/lazyvim.json`
   - editor behavior: `nvim/lua/config/{options,keymaps,autocmds}.lua`
   - plugins and generated pins: `nvim/lua/plugins/`, `nvim/lazy-lock.json`
   - snippets and formatting: `nvim/snippets/`, `nvim/stylua.toml`
2. Extend the existing concern-based plugin file when it owns the change, or add a focused file
   under `nvim/lua/plugins/`. Related specs may share a file, as `colorscheme.lua` and `disabled.lua`
   do; never split or combine mechanically.
3. Prefer minimal `opts` and preserve lazy loading with the appropriate `event`, `cmd`, `ft`, or
   `keys` trigger.
4. Identify who owns each keymap:
   - Put global editor mappings in `nvim/lua/config/keymaps.lua`.
   - Put plugin-owned, lazy-loading, or LSP mappings in the owning plugin specification's `keys`
     field.
5. Change `nvim/lazy-lock.json` only when dependencies or pins change. Generate it through Lazy,
   such as `nvim --headless '+Lazy! sync' +qa`; do not hand-edit generated state.
6. Verify without installing tools:
   - Run `~/.local/share/nvim/mason/bin/stylua --check <changed-lua-files>` when available, or use
     `stylua --check <changed-lua-files>` when already on `PATH`.
   - Run `luacheck nvim/lua --no-unused-args --no-max-line-length --globals vim` when `luacheck` is
     available; this matches CI.
   - Run `nvim --headless -i NONE '+qa'` and resolve every startup error.

## Gotchas

- **Editing `~/.config/nvim` as a separate copy** — the `Makefile` symlinks `nvim/` there; edit the
  repository source.
- **Centralizing every keymap** — moving plugin-owned mappings out of `keys` can break lazy loading;
  determine ownership first.
- **Creating one file per plugin by default** — existing files group related concerns; extend the
  current owner or create a focused file based on responsibility.
- **Hand-editing `nvim/lazy-lock.json`** — update this generated state through Lazy only when
  dependencies or pins change.
- **Running a formatter across all Lua files during a focused change** — unrelated files may already
  differ from current StyLua output; check only changed files unless broader formatting is requested.

## Constraints

- Neovim artifacts must remain under `nvim/` and follow their existing owner.
- Plugin changes must preserve LazyVim loading behavior and prefer `opts` over replacement `config`
  functions when the plugin supports it.
- Keymaps must be placed according to ownership: global in `config/keymaps.lua`, plugin-owned or
  load-triggering in the owning plugin spec's `keys`, including LSP mappings in the relevant plugin
  specification.
- `nvim/lazy-lock.json` must not change unless dependency resolution or plugin pins change.
- Do not install tooling directly; use available project tools and request authorization before
  dependency changes.
