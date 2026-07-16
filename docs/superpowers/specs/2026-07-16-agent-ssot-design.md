# Agent SSOT Design

## Objective

Make `AGENTS.md` and `.agents/` the single sources of truth for repository
instructions and shared skills. Agent-specific directories contain adapters
only.

## Canonical Structure

```text
AGENTS.md
CLAUDE.md -> AGENTS.md
.agents/
└── skills/
    ├── README.md
    ├── dotfiles/
    ├── johnny-decimal/
    ├── neovim/
    ├── scripts/
    ├── skill-manager/
    └── things-tasks/
.claude/
└── skills -> ../.agents/skills
.codex/
└── skills -> ../.agents/skills
.cursor/
└── skills -> ../.agents/skills
```

`AGENTS.md` is tool-agnostic and authoritative. It keeps the existing scope and
Personal Brain rules, states the conflict rule, and points to
`.agents/skills/README.md` instead of duplicating the skills index.

## Skill Migration

Move these existing Cursor skills into `.agents/skills/`:

- `dotfiles`
- `johnny-decimal`
- `neovim`
- `scripts`

Delete the obsolete Cursor-only skills:

- `cli-tools`
- `commits`
- `fish`
- `makefile`

Delete `.cursor/commands/`. After migration, `.cursor/` contains only the
relative `skills` symlink.

Run the `skill-manager` fix workflow on each migrated skill. Fix portability,
frontmatter, stale references, and index issues without expanding their
functional scope. Run doctor checks after fixing and synchronize
`.agents/skills/README.md`.

Johnny Decimal remains a separate system for `~/Documents`. It does not apply
to `~/Brain`, whose organization remains plain PARA.

## Agent Adapters

- `CLAUDE.md` is a relative symbolic link to `AGENTS.md`.
- `.claude/skills`, `.codex/skills`, and `.cursor/skills` are relative symbolic
  links to `../.agents/skills`.
- Codex and Cursor continue reading `AGENTS.md` natively.
- No commands, rules, or duplicated skill content remain under `.cursor/`.

## Verification

1. Run the `skill-manager` fix and doctor workflows for all four migrated
   skills.
2. Run `skill-manager` cross-check across `.agents/skills/`.
3. Confirm `.agents/skills/README.md` matches the actual skill directories.
4. Confirm each adapter is a relative symbolic link with the expected target.
5. Confirm `.cursor/` contains only `skills`.
6. Search for obsolete `.cursor/commands` and old skill references.
7. Run `git diff --check` and inspect the final diff.

## Non-Goals

- Migrating the four explicitly obsolete Cursor skills.
- Copying Modelo Suite's product-specific hooks, MCP servers, or tool settings.
- Changing the Personal Brain or Johnny Decimal folder structures.
- Keeping internal design or implementation-plan documents in the final PR.
