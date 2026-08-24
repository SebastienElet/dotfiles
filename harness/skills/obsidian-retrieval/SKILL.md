---
name: obsidian-retrieval
description: >
  Retrieve read-only knowledge from Obsidian vaults or local Markdown corpora. Use when answering
  from notes by exact search, conceptual retrieval, backlinks, properties, tasks, or Bases. Make
  sure to use this skill whenever local notes are evidence, even if Obsidian is not named.
compatibility: Requires read access to a local Markdown corpus; Obsidian CLI and semantic search are optional.
metadata:
  category: ops
---

# Obsidian Retrieval

## Overview

Retrieve evidence from an Obsidian vault or Markdown corpus without changing it. Select the cheapest
adequate healthy capability, treat every search result as a candidate, read the source notes, and
cite them by path. Filesystem access plus lexical search is the guaranteed baseline; Obsidian CLI
and semantic retrieval are optional enhancements.

## Usage

Use an explicit corpus input from the user, the configured default Obsidian corpus from loaded user
instructions, or the active workspace when it is trustworthy. Examples include “Find the note
titled ADR-183 and summarize it,” “What themes connect my notes about bounded contexts?”, and
“Which notes link to Architecture?”.

This skill returns an answer grounded in read source notes, source citations, and a concise account
of degraded or incomplete retrieval. It does not provide note-writing operations.

## Steps

1. Determine exactly one corpus root in this order:
   - a readable directory supplied as explicit corpus input;
   - the configured default Obsidian corpus declared in loaded user instructions, requiring both a
     readable directory and its `.obsidian/` child;
   - the nearest ancestor containing `.obsidian/` when the current directory is inside a vault;
   - the single current workspace root when it contains readable Markdown files.
     When a configured default is present but invalid, stop and report its exact path and failed
     check instead of selecting another corpus. Otherwise, stop and request a root when none is
     trustworthy or multiple workspace roots remain ambiguous. Never search parent directories,
     `$HOME`, or Obsidian's vault registry to guess a corpus.
2. Resolve the selected root, verify that it is a readable directory, and keep every filesystem
   read and search inside it. Exclude `.obsidian/` configuration files from note retrieval unless
   the user explicitly asks about vault configuration; configuration remains read-only.
3. Classify the request before choosing a capability:
   - **Exact anchors** such as titles, names, literal phrases, identifiers, and tags start with
     lexical filename and Markdown-content search.
   - **Conceptual questions** may use an already available semantic capability only after its
     configuration, corpus, index completeness, and health are established.
   - **Obsidian semantics** such as backlinks, outgoing links, properties, tasks, and Bases use the
     official CLI only after its read-only health checks succeed for the selected vault.
4. For exact anchors, search Markdown filenames and contents with the host's filesystem tools. Use
   fixed-string matching first, add case folding only when appropriate, return path and line data,
   and bound broad candidate sets before reading them. If `rg` is unavailable, use an equivalent
   filesystem listing and lexical grep without leaving the selected root.
5. For conceptual questions, use a semantic tool only when it is already installed, configured for
   the selected corpus, indexed, and healthy. Treat scores and snippets only as candidate ranking.
   When no semantic capability qualifies, report that limitation and fall back to several explicit
   lexical terms, synonyms, tags, and links found in the notes already read. Do not install QMD or
   create, update, or repair an index during retrieval.
6. For Obsidian semantics, read [references/obsidian-cli.md](references/obsidian-cli.md), run only
   its allowlisted commands from the selected vault root, and then read the returned source paths.
   If the CLI is missing, disabled, unhealthy, or targets another vault, report an explicit degraded
   result and use direct filesystem retrieval only for the partial facts it can actually establish.
   Never represent lexical link syntax or raw frontmatter inspection as complete Obsidian semantics.
7. Read every source note used for the answer, not merely the matching line. A search hit is a
   candidate, not evidence; snippets, semantic scores, and relation lists never support a factual
   answer by themselves. Follow only relevant candidate paths and keep the read set bounded.
8. Cite each supporting note with its root-relative path and line location when the retrieval tool
   provides one, for example `decisions/ADR-183.md:12`. Cite the path alone and say that line data is
   unavailable when a capability does not return stable locations.
9. Report unavailable tools, inaccessible roots, and incomplete indexes explicitly. For an empty
   result, name the root, query, capability, and known coverage limits; an empty candidate set does
   not prove that the information is absent from the corpus.

## Gotchas

- **Answering from snippets** — truncated context can reverse or omit the note's meaning. Read every
  cited source note before using it as evidence.
- **Treating a binary as healthy** — an installed CLI may be disabled or target a different vault,
  and a semantic index may be stale. Verify the selected corpus and health before trusting results.
- **Guessing a vault path** — scanning parent directories or the home directory can cross privacy
  boundaries. Use the closed root-selection order and stop on ambiguity.
- **Masking an invalid default** — silently choosing a workspace or the Web hides configuration
  drift. Report the configured path and failed check, then stop.
- **Presenting no matches as absence** — lexical wording and incomplete indexes can cause false
  negatives. Report the query and coverage limits instead of claiming the vault lacks the fact.
- **Emulating Obsidian relations with grep** — raw wikilinks and YAML do not reproduce link
  resolution, task state, or Bases evaluation. Label the filesystem fallback as degraded and partial.

## Constraints

- Keep all operations read-only and inside the selected corpus root.
- Never write, create, append, prepend, move, rename, or delete vault content or configuration.
- Never expose mutating or unrestricted Obsidian CLI operations.
- Never install QMD, another semantic tool, a dependency, or an MCP server during retrieval.
- Never replace a missing or inaccessible local corpus with Web retrieval.
- Never use an MCP transport as evidence that retrieval quality or index coverage improved.
- Never answer a factual question only from filenames, snippets, relation output, or scores.
- Never claim exhaustive absence without a named healthy oracle whose complete coverage is known.
