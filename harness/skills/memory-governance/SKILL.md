---
name: memory-governance
description: >
  Govern durable local agent memory. Use when a user requests persistence, accepts a memory
  proposal, supplies a prior memory, or when durable knowledge is detected during work. Make sure
  to use this skill at the start of every Cursor task, even if memory is not mentioned.
metadata:
  category: ops
---

# Memory Governance

## Overview

Use `agent-memory` as the only memory runtime. Memory is a source-backed retrieval aid, never an
authority: the current ADR, contract, configuration, user decision, or official dependency behavior
remains primary. Read [references/entry-contract.md](references/entry-contract.md) before proposing,
admitting, retrieving, or confirming memory.

## Usage

Use this workflow for explicit remember/retain requests, acceptance of a prior proposal, detected
knowledge that may be costly to rediscover, consumption of prior memory, and every Cursor task
start. Use `obsidian-retrieval` for read-only Markdown corpus retrieval and `agent-instructions`
when the durable destination is agent behavior.

## Workflow

1. **Retrieve before Cursor work.** Announce retrieval before analysis or action. Pass the complete
   current user prompt on stdin to `agent-memory retrieve --query-stdin --format json`, wait for
   completion, and apply only a successful sanitized result. Announce each applied entry's kind,
   statement, proof source, and age of its last valid verdict. On any failure or unavailable result,
   announce that durable memory is unavailable and apply nothing.
2. **Propose without writing.** When work reveals durable knowledge that may be materially costly to
   rediscover, locate its current primary authority and apply the admission contract in the
   reference. Return a complete draft and state that it was not persisted. Do not run `admit`
   without an explicit persistence request or acceptance of that draft.
3. **Admit after authorization.** For an explicit request or accepted proposal, remove volatile
   incident detail and pass the complete draft on stdin to `agent-memory admit --format json`.
   Report `stored`, `duplicate`, or the redacted rejection; never claim persistence before a
   successful result.
4. **Confirm compatible outcomes.** When a human supplies a terminal conclusion for a compatible
   entry, pass only the reason on stdin to the exact `confirm` command in the reference. Report the
   returned status. Never infer a human conclusion from silence or task progress.
5. **Fail closed and redact.** Refuse raw transcripts, complete private prompts, secrets,
   credentials, personal data, and workarounds for defects in owned code. Name the failed criterion
   without reproducing the content, do not send refused material to `agent-memory`, and persist
   nothing.
6. **Keep behavior changes separate.** Memory cannot promote itself into instructions, skills,
   hooks, or enforcement. Use `harness-reflection` for that evidence and trial.

## Gotchas

- **Acting before Cursor retrieval completes** — memory can influence work without a freshness
  check; wait for the command and apply only its successful result.
- **Writing an automatic proposal** — observation becomes authority without consent; return the
  complete draft and wait for explicit acceptance.
- **Treating a successful command launch as persistence** — rejected or conflicting admission is
  misreported; inspect the JSON result before claiming storage.
- **Copying sensitive input into a refusal** — the diagnostic leaks what policy rejected; name only
  the criterion and effect.
- **Saving an owned workaround** — a defect becomes doctrine; fix it or route a focused issue.

## Constraints

- Use only the exact `agent-memory` commands in the reference; never read or write memory store files
  directly.
- Never apply memory when retrieval fails, is unavailable, needs confirmation, or returns no
  relevant injected entry.
- Never admit without an explicit request or acceptance of a complete proposal.
- Never let memory replace, amend, or outrank its primary source.
- Never persist raw transcripts, complete private prompts, secrets, credentials, personal data, or
  owned-defect workarounds.
- Never broaden scope beyond what the verified authority establishes.

## References

- [references/entry-contract.md](references/entry-contract.md) — admission criteria, complete draft,
  and exact runtime commands
