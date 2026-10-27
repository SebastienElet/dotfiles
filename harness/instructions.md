## Context Management

- **Locate in the main thread.** Use bounded `rg` and `fd` searches to locate relevant code and
  consumers before delegating bulk reading. Verify consequential findings in source.
- **Delegate reading, not locating.** A subagent carries the bulk reading and the synthesis that
  follow a search, never the search itself. Test before delegating: can you state everything you
  need back in one line — a path, a count, a verdict? Then delegate. Otherwise it stays here.
- Keep the main thread for orchestration and decisions.
