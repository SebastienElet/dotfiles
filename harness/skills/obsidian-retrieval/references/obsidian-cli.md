# Read-only Obsidian CLI

Use this reference only for requests that depend on Obsidian's own link resolution, property model,
task model, or Bases evaluation. The command surface below follows the official
[Obsidian CLI documentation](https://help.obsidian.md/cli), verified on 2026-08-23.

## Health and targeting

Binary presence is insufficient. Confirm that the Obsidian desktop application is already running
before the first CLI call. Use read-only process inspection or reliable session state; when neither
can establish that fact, treat the CLI as unavailable. This prevents a health probe from launching
Obsidian and changing vault-internal application state.

From the selected vault root, require both `obsidian version` and `obsidian vault info=path` to
succeed, and require the reported path to resolve to that root. The CLI requires a compatible
Obsidian installation and must be enabled in Obsidian settings; report failures rather than changing
settings.

Run commands from the selected vault root so Obsidian targets that vault. Do not choose the active
vault implicitly from an unrelated directory, and do not select a different vault by guessing its
name or identifier.

## Read-only allowlist

Only these commands are allowed: `aliases`, `backlinks`, `base:query`, `base:views`, `bases`, `file`,
`files`, `links`, `properties`, `property:read`, `read`, `search`, `search:context`, `tag`, `tags`,
`tasks`, and `vault`.

This closed set covers text search, source reads, links, properties, tasks, Bases, target-vault
validation, and vault-local inventory. It excludes the global known-vault inventory because that
would reveal paths outside the selected corpus.

Prefer exact root-relative path parameters over ambiguous file-name resolution. Request structured
output where supported, but treat every returned path as a candidate and read the note before
answering from it.

## Degraded behavior

If either health check fails or the reported vault differs, do not use the CLI. State that Obsidian
semantics are unavailable, preserve the original question, and fall back to Markdown filesystem
retrieval for partial evidence only. A raw link string, YAML property, task marker, or `.base` file
does not prove the corresponding resolved Obsidian result.

## Constraints

- Run only commands in the read-only allowlist.
- Never invoke generic command dispatch, developer evaluation, plugin operations, or URI opening.
- Never change Obsidian settings or launch an installation workflow.
- Never use relation or query output without reading the relevant returned notes.
