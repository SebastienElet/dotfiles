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

The bootstrap downloads Bun 1.4.3, verifies its pinned SHA-256 checksum, and installs
Moon from `.github/actions/setup-moon/package.json` and its frozen `bun.lock` in a
temporary directory. It copies the native binary to `~/.moon/bin/moon` and prints
its executed version. Temporary downloads, dependencies and caches are removed; no
global pin or shell profile is written. Missing or invalid Moon configuration and
download/installation failures stop before workstation installation. Network access
is required on every bootstrap.

Cloning fails if `~/.dotfiles` already exists and is not empty. For an existing
checkout, use the manual install below.

## Manual install

From an existing checkout, rerun the bootstrap to align Moon with the checkout,
then install the minimal profile:

```bash
cd ~/.dotfiles && \
  ./tooling/install-moon && \
  "$HOME/.moon/bin/moon" exec --quiet install
```

Install the separately maintained optional profile with `moon exec repository:optional`.

[Herdr](https://herdr.dev/docs/install/) is optional. Install the agent multiplexer
with `moon exec repository:herdr` on macOS ARM64, using the official Homebrew formula.
The task also deploys `~/.config/herdr/config.toml` with automatic Catppuccin
dark/Latte switching when the host terminal reports a light/dark appearance change.
It installs `@sebastienelet/herdr-worktree-cleanup@1.0.1` with Bun in
`~/.local/share/herdr-worktree-cleanup` and links its `dotfiles.worktree-cleanup` plugin,
which removes a linked worktree's checkout when its workspace closes.
The plugin explicitly enables untracked-file visibility
for Git's removal check, including when `status.showUntrackedFiles=no` is configured.
Git refuses modified tracked files, non-ignored untracked files, and locked worktrees;
the plugin also skips detached HEADs. Removal failures are reported with a nonzero exit,
and a removed checkout's branch is retained. No removal uses `--force`.
Ignored files do not prevent removal and are deleted with an otherwise clean checkout.
Keep valuable ignored artifacts elsewhere or explicitly lock the worktree; any additional
ignored-file protection requires a separate decision.

Persistent deployments must use the canonical checkout at `~/.dotfiles` as their source.
For exceptional validation from a worktree, use isolated destinations and invoke only
fixture-safe Moon tasks after inspecting their dependencies; an isolated `HOME` alone
does not sandbox global installers. If a user link must temporarily target the worktree,
record its destination and intended canonical source, then run
`git worktree lock --reason 'temporary deployment source' /absolute/worktree/path`
before deploying. Keep the lock until the change is integrated into the canonical checkout,
the affected links are explicitly repointed there, and their resolved targets are verified.
Only then run `git worktree unlock /absolute/worktree/path` and allow cleanup.
The plugin does not detect deployed links or copy, repoint, or repair them automatically.

The unused issue launcher and its companion skill have been retired. For an existing installation,
inspect `herdr plugin list`, then disable and unlink only `dotfiles.issue-worktree` with
`herdr plugin disable dotfiles.issue-worktree` and `herdr plugin unlink dotfiles.issue-worktree`.
Remove the `herdr-issue-worktree` skill symlinks under `~/.claude/skills`, `~/.cursor/skills` and
`~/.agents/skills` only after verifying that they target the retired source. Existing worktrees,
agent sessions and launcher state are retained; deployment does not perform this removal.

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
`harness:assertledger`, using Bun and the version supplied by the skills package. Install it independently
with `moon exec harness:assertledger` on macOS. Its stable absolute entry is
`$HOME/.local/bin/assertledger`. The generated launcher directly executes the exact Volta-managed
Node from `package.json` on the installed `dist/cli.js`, without relying on Node in the caller's PATH
or installing anything at launch. It requires that runtime and the pinned global Bun package;
rerun the installation after updating the Node pin to refresh the launcher. Installation preserves
and reports a conflicting destination.

The [published skill](https://www.npmjs.com/package/@sebastienelet/agent-skills/v/0.3.0) is deployed to Claude Code
and Codex with the minimal profile, and to Cursor with the optional profile. Start with
`"$HOME/.local/bin/assertledger" --version` and
`"$HOME/.local/bin/assertledger" doctor /path/to/repository --json`.
The Git `check` workflow accepts committed dependency-free JavaScript `node:test` regressions;
Ce workflow ne qualifie pas les suites TypeScript/Bun de ce dépôt. Repository symlinks can
also cause the static diagnostic to refuse a checkout. Execution requires an explicitly chosen
backend; installation does not initialize projects or register an MCP server.

## Checks

### Contributing from a worktree

Use `tooling/install-moon` to install the locked Moon version and start with the
smallest relevant task below. Workstation installation is macOS-only; portable
checks also run on Ubuntu. Inspect the task definition and its native action graph before running
it, especially before either repository aggregate:

```bash
moon action-graph repository:prettier-check --dot
moon action-graph repository:check --dot
moon action-graph repository:test --dot
```

The graph identifies setup actions and dependency tasks; read their commands to
distinguish these effects:

- Moon caches, checkout `node_modules`, Cargo `target` directories and downloaded
  crates are development state. Bun/proto setup may also populate their user caches.
- Deployment tests use temporary fixture destinations and installer substitutes;
  invoking a workstation deployment task directly writes to its configured destination.
- `repository:homebrew`, `fish-setup`, `luacheck-setup`, `shellcheck-setup`,
  `actionlint-setup` and `rust` can install tools outside the checkout. An isolated
  `HOME` does not sandbox Homebrew, `sudo apt-get`, Go or rustup.

Do not run global installation dependencies from a worktree. Prepare the toolchain
from the canonical checkout or use a dedicated runner. If a selected graph still
contains such tasks, use a dedicated runner, or select a leaf check whose only
task dependencies are preparation. After verifying its tools and versions, a
prepared environment can run, for example:

```bash
moon run repository:shell-lint --upstream none --no-actions
```

These flags skip dependency tasks and toolchain setup. Never apply this shortcut
to an aggregate such as `harness:check`, `repository:check` or `repository:test`:
it would omit their checks. Also preserve required build/test dependencies; if
they cannot safely run in the worktree, use the runner. This checks existing tools,
not installation.

Before format, lint or typecheck, stage every new file in the intended change with
explicit pathspecs (`git add -- path/to/new-file` after replacing the example path).
Stop if staging fails. TypeScript lint/format select indexed paths via
[`tracked-typescript-paths.ts`](tooling/tracked-typescript-paths.ts); Prettier and
CSpell include indexed skill Markdown via
[`skill-markdown-paths.ts`](tooling/skill-markdown-paths.ts). Untracked files may
therefore be absent from these checks; the commands inspect working-tree contents,
so refresh the index after subsequent edits before committing.

All task names below are existing Moon targets, invoked with `moon run TARGET`.
Prerequisites include the pinned Bun toolchain and root package dependencies where
the task uses Bun. Effects include Moon's caches in addition to those listed.

| Changed zone                       | Existing Moon target(s)                                                                               | Additional prerequisites                         | Platforms                 | Effects / worktree constraint                                                            |
| ---------------------------------- | ----------------------------------------------------------------------------------------------------- | ------------------------------------------------ | ------------------------- | ---------------------------------------------------------------------------------------- |
| Markdown, JSON, YAML               | `repository:prettier-check`                                                                           | Root package dependencies                        | macOS, Ubuntu             | Reads files; no workstation deployment                                                   |
| Skills and spelling configuration  | `repository:cspell-check`                                                                             | Root package CSpell                              | macOS, Ubuntu             | Reads indexed inputs; deploys temporary configuration/dictionary links, then cleans them |
| TypeScript                         | `repository:typescript-lint`, `repository:typescript-typecheck`, `repository:typescript-format-check` | Root package dependencies                        | macOS, Ubuntu             | Reads sources; lint/format require new files in the index                                |
| Validation déterministe du harnais | `harness:check`                                                                                       | Dépendances Bun du dépôt                         | macOS, Ubuntu             | Contrôles et fixtures temporaires                                                        |
| Neovim / WezTerm Lua               | `repository:neovim-lint`, `repository:wezterm-lint`                                                   | Luacheck                                         | macOS, Ubuntu             | Reads configuration; `luacheck-setup` can install globally                               |
| Fish                               | `repository:fish-syntax`, `repository:fish-format`, `repository:fish-test`                            | Fish                                             | macOS, Ubuntu             | Syntax/format checks and behavior fixtures; `fish-setup` can install globally            |
| Shell helpers                      | `repository:shell-lint`                                                                               | Shellcheck                                       | macOS, Ubuntu             | Reads scripts; `shellcheck-setup` can install globally                                   |
| GitHub workflows                   | `repository:workflows-lint`                                                                           | Actionlint, Shellcheck                           | macOS, Ubuntu             | Reads workflows; both setup tasks can install globally                                   |
| Deployment helpers                 | `tooling:deployment-test`                                                                             | Root package dependencies                        | macOS, Ubuntu             | Temporary fixture links/configuration and installer substitutes                          |
| Hunspell installer                 | `tooling:hunspell-test`                                                                               | Root package dependencies                        | macOS, Ubuntu             | Temporary fixture dictionaries and simulated downloads                                   |
| Docker integration                 | `tooling:docker-smoke`                                                                                | Running Docker daemon and image/network access   | macOS, Ubuntu with Docker | Runs containers, creates volumes and downloads images; explicit opt-in                   |
| Full minimal installation          | `tooling:smoke-minimal`                                                                               | Dedicated macOS runner, installer/network access | macOS                     | Global packages and deployed user artifacts; never run from a worktree                   |

For a README-only change, inspect and run `repository:prettier-check`. For other
zones, choose the relevant row and inspect its graph first. `moon run harness:check`
is the explicit deterministic aggregate. **Do not substitute `moon check harness`**:
Moon selects build/test tasks inferred across the project, including operational
tasks such as `semctx` that install plugins. See the
[Harness evaluation guide](docs/arnes-eval.md).

On a prepared environment, the broad local aggregates remain `moon run check` and
`moon run test`; they are not the default worktree path. The full workstation smoke
is `moon exec tooling:smoke-minimal` on a dedicated macOS runner.
The CI routing matrix is owned by [#407](https://github.com/SebastienElet/dotfiles/issues/407)
and its sub-issues; this local task guide does not duplicate its path-to-workflow policy.

Deployment families run through `tooling:deployment-test` and `tooling:hunspell-test`.
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

Placement follows [ADR-038](docs/adr/038-frontieres-home-harness-tooling.md).

- `home/` mirrors the destination-relative paths of files deployed under `$HOME`.
- `harness/` contains instructions and capabilities shared across agent harnesses.
- `tooling/` contains maintained local applications and extensionless kebab-case executables.
- Tool-mandated integration paths and repository entry points remain at the root.

## Arnes Doctor

La tâche `harness:arnes` déploie le lanceur du paquet npm publié et le manifeste.
Les hooks sont installés manuellement ; Arnes vérifie leur configuration sans les modifier.
Le [guide Arnes](docs/arnes.md) décrit les diagnostics et leurs limites.
L’[index documentaire](docs/README.md) couvre aussi Agent-Reach et les évaluations du harnais.

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

L’export contient les sources non ignorées sous `harness/`, sauf l’index généré des skills et
les métadonnées du système, ainsi que les déclarations de hooks dans `home/.arnes.yaml`.
La sélection des sources et ses tests appartiennent au paquet Arnes publié.
