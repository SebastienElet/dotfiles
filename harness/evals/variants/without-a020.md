## Context Management

- **Locate with `code-search`, in the main thread.** Any exploratory or structural search —
  architecture, call paths, dependencies, cross-package behavior, change impact, first contact with
  an unfamiliar repository — goes through the `code-search` skill, invoked here rather than handed
  to a subagent. Locating is cheap: `rg`, `fd` and `colgrep-search` return bounded output.
- **Delegate reading, not locating.** A subagent carries the bulk reading and the synthesis that
  follow a search, never the search itself. Test before delegating: can you state everything you
  need back in one line — a path, a count, a verdict? Then delegate. Otherwise it stays here.
- A subagent does not inherit skill routing. When one must search on its own, its prompt names
  `code-search` explicitly.
- Keep the main thread for orchestration and decisions.
