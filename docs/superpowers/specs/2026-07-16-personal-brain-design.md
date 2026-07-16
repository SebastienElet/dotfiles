# Personal Brain Design

## Objective

Create a simple personal Brain stored in iCloud Drive and exposed through a
stable `~/Brain` symbolic link. The Brain is the source of truth for learned
information, decisions, and ongoing work.

## Storage

The canonical Brain directory is:

```text
~/Library/Mobile Documents/com~apple~CloudDocs/Brain
```

The stable access path is:

```text
~/Brain
```

`~/Brain` must be a symbolic link to the canonical iCloud directory. The setup
must never replace an existing file, directory, or incorrect symbolic link.

## Makefile Integration

Add a phony `brain` target to the root `Makefile` and include it as a dependency
of `ai`. Define an overridable `BRAIN_PATH` variable whose default is the
canonical iCloud directory, so failure cases can be tested safely with temporary
paths.

The target must:

1. Fail with a clear message if the canonical iCloud directory does not exist.
2. Create the `~/Brain` symbolic link when the destination is absent.
3. Do nothing when the destination is already the correct symbolic link.
4. Fail without changing anything when the destination exists but is not the
   expected symbolic link.
5. Never create or modify the Brain's PARA content.

The checks should remain directly in the Makefile. A separate setup script is
unnecessary for this small operation.

## Initial Brain Structure

Initialize the existing canonical Brain directory once with:

```text
Brain/
├── AGENTS.md
├── Inbox/
├── Projects/
├── Areas/
├── Resources/
└── Archive/
```

Do not add example notes, placeholder files, or additional directories.

## Brain Rules

`~/Brain/AGENTS.md` defines the Brain's operating rules:

- All notes use Markdown.
- Prefer simplicity and human readability.
- Do not create complex structures.
- Create directories only when necessary and reuse existing directories.
- Use explicit filenames.
- Use relative paths for links between notes.
- `Inbox/` contains unprocessed information.
- `Projects/` contains work with a defined end.
- `Areas/` contains ongoing responsibilities.
- `Resources/` contains reusable knowledge.
- `Archive/` contains completed or inactive content.
- Every project has a `README.md` containing its objective, context, current
  state, important decisions, useful links, and next steps.
- Every note must remain understandable months later without the original
  conversation.
- Prefer incremental updates over complete rewrites.

The repository's root `AGENTS.md` must include a short instruction requiring
agents to read `~/Brain/AGENTS.md` before operating on Brain content. The Brain
rules remain canonical in the Brain and are not duplicated in the dotfiles.

## Verification

Verify the implementation by:

1. Running `make brain` and confirming `~/Brain` resolves to the canonical
   iCloud directory.
2. Running `make brain` again and confirming it is idempotent.
3. Confirming the five PARA directories and `AGENTS.md` exist.
4. Exercising missing-source and conflicting-destination cases with temporary
   `HOME` and Brain paths so real iCloud data is not affected.
5. Reviewing the final dotfiles diff for unrelated changes.

## Non-Goals

- Automating PARA directory creation through `make brain`.
- Adding note templates beyond the project `README.md` requirements.
- Adding scripts, applications, indexing, synchronization, or search tooling.
- Migrating or generating Brain content.
