---
description: General guidelines for dotfiles configuration
globs:
---

# General Guidelines

- All comments and documentation must be written in **English**
- Keep configurations **simple and minimal** — avoid over-engineering
- Prefer **self-documenting code** over excessive comments
- Primary platform: **macOS** (Darwin), but keep configs **portable to Linux** (EC2, servers)
- Avoid macOS-specific features when a cross-platform alternative exists
- Use `uname` or similar checks when platform-specific code is unavoidable

## Installation & Package Management

**CRITICAL**: Always use the Makefile for installing tools and applications. Never install packages directly with `brew install`, `npm install -g`, or other package managers. See `makefile.md` for details.

## Symlink Conventions

Configuration files are symlinked from this repo:

- `~/.config/fish` → `~/.dotfiles/fish`
- `~/.config/nvim` → `~/.dotfiles/nvim`
- `~/.config/Cursor/User/*` → `~/.dotfiles/cursor/*`

Always update the Makefile when adding new symlinked configurations.

## Primary Work Context

Main technologies used daily:

- **TypeScript** — Use strict mode, prefer explicit types
- **PostgreSQL** — Use UPPER CASE for SQL keywords
- **Node.js** — Managed via Volta
