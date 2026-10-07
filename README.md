# Dotfiles

These dotfiles support macOS only. Linux, containers, and GitHub Codespaces are unsupported.

## Install

The recommended path is to clone the repository, bootstrap Moon, and install
the minimal profile with Moon. Apple's Command Line Tools and working Git are
required. If `xcode-select --print-path` or `git --version` fails, run
`xcode-select --install`, complete the macOS system dialog, and wait for the
installation to finish before continuing.

```bash
cd && \
  git clone --depth 1 https://github.com/SebastienElet/dotfiles.git .dotfiles && \
  cd .dotfiles && \
  ./tooling/install-moon && \
  "$HOME/.moon/bin/moon" exec --quiet install
```

Cloning fails if `~/.dotfiles` already exists and is not empty. For an existing
checkout, use the manual install below.

## Manual install

From an existing checkout, bootstrap Moon if needed, then install the minimal
profile:

```bash
cd ~/.dotfiles && \
  ./tooling/install-moon && \
  "$HOME/.moon/bin/moon" exec --quiet install
```

Install the separately maintained optional profile with `moon exec repository:optional`.

Anarlog belongs to the optional profile, alongside Handy. Install it independently
with `moon exec repository:anarlog` (macOS 15 or newer).

[Minutes](https://useminutes.app/) is also optional. Install the desktop app with
`moon exec repository:minutes` (Apple Silicon, macOS 14 or newer), using the publisher's
`silverstein/tap` Homebrew cask. Native call capture requires macOS 15 or newer.
First launch downloads a local speech model;
transcription does not require an API key. Summarization is optional and configured
separately. The Minutes CLI is not installed by this task.

[Herdr](https://herdr.dev/docs/install/) is optional. Install the agent multiplexer
with `moon exec repository:herdr` on macOS, using the official Homebrew formula.
The task also deploys `~/.config/herdr/config.toml` with automatic Catppuccin
dark/Latte switching when the host terminal reports a light/dark appearance change.
It also links the `dotfiles.worktree-cleanup` plugin, which removes a linked worktree's
checkout when its workspace closes; Git refuses a checkout with modified or untracked
files, which therefore stays in place.

[Grok Bot](https://x.ai/bot) belongs to the minimal profile. Install it independently with
`moon exec repository:grokbot` (macOS 12 or newer, Apple Silicon
or Intel), using the official Homebrew `grok-bot` cask.

Moon installs the complete minimal profile. With Moon available:

```bash
moon exec --quiet install
moon action-graph repository:install
```

Install the Node development toolchain independently with `moon exec repository:pnpm`;
its dependencies install Homebrew, Volta, and the exact Node version from `package.json`.

## AssertLedger

[AssertLedger](https://github.com/hoklims/assertledger) qualifies regression tests against a
declared fault and neutral control. The minimal profile installs its pinned CLI through
`harness:assertledger`, using npm with the Volta-managed Node runtime. Install it independently
with `moon exec harness:assertledger` on macOS.

The locally maintained [skill](harness/skills/assertledger/SKILL.md) is deployed to Claude Code
and Codex with the minimal profile, and to Cursor with the optional profile. Start with
`assertledger --version` and `assertledger doctor /path/to/repository --json`.
The Git `check` workflow accepts committed dependency-free JavaScript `node:test` regressions;
it does not qualify this repository's TypeScript/Bun or Rust suites. Repository symlinks can
also cause the static diagnostic to refuse a checkout. Execution requires an explicitly chosen
backend; installation does not initialize projects or register an MCP server.

See the [skills audit](docs/skills-audit-412.md) for the obsolete instructions corrected in #412.

## Checks

With Moon available, run the shared checks and behavior tests:

```bash
moon run check
moon run test
```

Checks include TypeScript, Prettier, Lua, Fish, shell scripts, workflows, CSpell and Rust.
Their tools are prerequisites in the Moon graph. Run Lua lint independently with
`moon run neovim-lint` or `moon run wezterm-lint`.
The full workstation smoke is `moon exec tooling:smoke-minimal` on a dedicated macOS runner.
Docker integrations remain explicit tasks with their own prerequisites.

Deployment families run through `tooling:deployment-test`, `tooling:hunspell-test`,
`agent-memory:deployment-test`, and `agent-handoff:deployment-test`.
Their CI workflows use `moon ci --downstream none` to select consumers of changed inputs
on macOS and Linux. The general `repository:typescript-test` excludes tests owned by
these families; `moon run test` still includes their local tests.
Typechecking stays shared in Static gates.

## Architecture decisions

Structural choices — installer, shell, editor, container runtime, agent
instructions — are recorded as ADRs in [`docs/adr/`](docs/adr/README.md), one
file per decision, reconstructed from the git history. Only decisions still in
force are recorded; superseded ones survive as the "Alternatives écartées"
section of whichever decision replaced them.

Read the relevant ADR before changing one of these choices, and add a new ADR
when making one.

## Repository layout

See [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md) for the complete project map,
deployment flow, and placement rules.

- `home/` mirrors the destination-relative paths of files deployed under `$HOME`.
- `harness/` contains instructions and capabilities shared across agent harnesses.
- `tooling/` contains maintained local applications and extensionless kebab-case executables.
- Tool-mandated integration paths and repository entry points remain at the root.

## Arnes Doctor

See the [Doctor reference](docs/arnes-doctor.md) for its diagnostic scope,
resource coverage, defaults, examples, exit codes, and observation limits for
Claude Code, Cursor, and Codex.

## Harness project export

Generate the portable user-harness Markdown snapshot used by ChatGPT Projects, Claude Projects,
Gemini, or notebook tools:

```bash
arnes export
arnes export --check
```

Upload every Markdown file from `.harness-export/`, including `00-MANIFEST.md`. The directory is a
temporary ignored build artifact: never edit or commit it. The check command performs no writes and
fails when local sources, bundles, or the manifest drift, or when an obsolete artifact remains.

The export contains every non-ignored source under `harness/`, except the generated skills index and
operating-system metadata, plus the canonical hook declarations in `home/.arnes.yaml`. Source
categories are defined once in `tooling/arnes/src/export/sources.rs`; extend that selector
and its tests to add a category, then regenerate the snapshot.
