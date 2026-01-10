---
description: Makefile structure and conventions
globs: Makefile
---

# Makefile Conventions

**CRITICAL RULE**: **ALWAYS use the Makefile for installing tools and applications**. Never install tools directly with `brew install`, `npm install -g`, or any other package manager commands. The Makefile is the single source of truth for what tools are installed and how they're configured.

Structure targets to be self-documenting — the target name should clearly indicate its purpose, avoiding the need for comments.

## Sections (respect this organization)

- `terminal`: Shell and terminal tools (fish, tmux, nvim, wezterm...)
- `work`: Development tools (cursor, docker, gh, terraform...)
- `personal`: Personal apps (obsidian, rust, spotify...)
- `utils`: Utility applications (cleanshot, keycast...)

## Target Patterns

- Tool targets depend on `brew` and the binary path
- Config targets create symlinks from `~/.dotfiles/` to `~/.config/` or `~/`
- Use `${BREW_BIN}` and `${APP_BIN}` variables for paths
- GUI apps (`.app`) are macOS-only; CLI tools should work on both macOS and Linux

## Example

```makefile
newtool: brew ${BREW_BIN}/newtool
${BREW_BIN}/newtool:
	brew install newtool
```

Note: Homebrew works on both macOS and Linux. For Linux-only alternatives, consider adding conditional targets.

## Installation Workflow

**When installing a new tool:**

1. **First**: Add the tool to the Makefile in the appropriate section (terminal/work/personal/utils)
2. **Then**: Run `make <tool-name>` to install it
3. **Never**: Install tools directly with `brew install`, `npm install -g`, etc. outside the Makefile

**Example workflow:**
```bash
# ✅ CORRECT: Add to Makefile first, then install
# 1. Edit Makefile to add the tool
# 2. Run: make zoxide

# ❌ WRONG: Installing directly
brew install zoxide  # Don't do this!
```

This ensures:
- All installations are tracked and documented
- Reproducible setup across machines
- Easy to see what tools are installed
- Consistent with the dotfiles philosophy
