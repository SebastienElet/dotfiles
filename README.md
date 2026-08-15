# Dotfiles

## Install

```bash
curl -fsSL \
  https://raw.githubusercontent.com/SebastienElet/dotfiles/main/install.sh | bash
```

## Manual install

```bash
cd && \
  git clone --depth 1 https://github.com/SebastienElet/dotfiles.git .dotfiles && \
  cd .dotfiles && \
  make all
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

- `home/` mirrors the destination-relative paths of files deployed under `$HOME`.
- `harness/` contains instructions and capabilities shared across agent harnesses.
- `tooling/` contains maintained local applications and extensionless kebab-case executables.
- Tool-mandated integration paths and repository entry points remain at the root.
