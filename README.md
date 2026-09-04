# Dotfiles

These dotfiles support macOS only. Linux, containers, and GitHub Codespaces are unsupported.

## Install

```bash
curl -fsSL \
  https://raw.githubusercontent.com/SebastienElet/dotfiles/main/install.sh | bash
```

If Git is unavailable, the installer exits without starting the Command Line
Tools installation. Run `xcode-select --install`, complete the macOS system
dialog, then rerun the command above after the installation finishes.

## Manual install

```bash
cd && \
  git clone --depth 1 https://github.com/SebastienElet/dotfiles.git .dotfiles && \
  cd .dotfiles && \
  make moon && \
  make minimal
```

Install the separately maintained optional profile with `make optional`.

Installation is moving to Moon, one dependency at a time. With Moon available:

```bash
moon exec install
moon action-graph repository:install
```

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
