# Agent Instruction Maintenance

When creating or changing instructions for coding agents:

- Inspect the repository's existing instruction hierarchy and discovery paths before editing.
- Keep always-loaded files limited to stable guidance that applies to every task.
- Put conditional procedures in skills so agents load them only when relevant.
- Maintain one canonical source for shared guidance; agent-specific paths contain only adapters.
- Prefer `AGENTS.md` for shared project instructions and keep `CLAUDE.md` as a Claude adapter when both exist.
- Scope nested instruction files to the directory that needs them and keep root instructions concise.
- Give skills trigger-focused descriptions and actionable, task-oriented content.
- Update every importer, generated file, installer, index, and verification barrier that consumes a changed source.
- Add context only when it changes agent behavior; remove or refine stale guidance instead of accumulating it.
