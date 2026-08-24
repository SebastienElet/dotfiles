# Docs Agent Instructions

Scoped to `docs/`. The repository-wide rules in the root `AGENTS.md` still apply; this file only
narrows the documentation language.

## Language

- **Documents under `docs/` are written in French**, by explicit exception to the
  English-documentation rule that governs the rest of the repository. They record reasoning —
  decisions, designs, trade-offs — for a French-speaking author, not a public interface of the
  repository.
- The exception covers prose only. Identifiers, commands, file paths, API names, commit hashes and
  quoted commit bodies stay verbatim.
- Do not translate an existing document to satisfy this rule.
- This file and any other agent-instruction file stay in English, being an interface rather than
  reasoning.

## Architecture Decisions

- `adr/` holds the architecture decision records; see `adr/README.md` for the index, the MADR format
  and the scope rule (only decisions still in force are recorded).
