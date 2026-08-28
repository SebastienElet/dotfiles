---
name: memory-governance
description: >
  Govern explicit durable cross-session memory requests for source-backed invariants that are
  expensive to rediscover. Use when a user or workflow explicitly asks to remember, persist, or
  retain an invariant for later sessions, or supplies a previously returned memory candidate for
  use. Make sure to use this skill whenever an explicit durable-memory request has no supported
  store configured.
metadata:
  category: ops
---

# Memory Governance

## Overview

Preserve durable invariants that are expensive to rediscover without turning observations into
authority. A memory entry is a sourced retrieval aid: it may spare repeated investigation, but the
primary ADR, contract, configuration or official dependency documentation remains authoritative.
This pilot does not persist memory. It returns source-backed candidates and governs whether a prior
candidate may be used in the active task.

## Usage

Use this workflow for requests such as "remember this invariant for future sessions", "record what
the harness should retain", or "stop rediscovering this accepted architectural constraint".

Do not use it for read-only retrieval from an existing Markdown corpus; use `obsidian-retrieval`.
When the durable destination is an instruction rather than memory, use `agent-instructions` and do
not create a duplicate memory entry.

## Steps

1. **Locate the authority.** Identify the in-force primary source, its exact scope and the revision,
   version or status checked now. Do not infer authority from a transcript or remembered summary.
   If the source is unavailable or its status cannot be verified, return `status: rejected` with the
   failed criterion and stop.
2. **Apply the admission contract.** Admit an invariant only when all of these hold:
   - it is durable within a named scope;
   - normal code, configuration or documentation discovery does not reveal it cheaply;
   - a primary source establishes it now;
   - retaining it avoids material repeated investigation;
   - an observable future event can tell an agent when it may no longer hold.
     A stable human preference already canonical in `USER.md`, or an invariant already present in
     always-loaded instructions, stays there without a memory copy.
3. **Separate the invariant from the incident.** Keep only the source-backed rule that survives the
   current failure. Route an owned defect to a fix or focused issue and omit its workaround. An
   external defect may become a temporary compatibility constraint only when official behavior,
   affected versions and its removal condition are verified.
4. **Return the candidate in this exact shape:**

   ```yaml
   status: candidate
   statement: <one durable invariant>
   scope: <smallest project, domain, agent or user scope that owns it>
   authority: <ADR, contract, configuration, user decision or official dependency behavior>
   source: <stable locator plus the exact revision, version or status checked>
   why_non_derivable: <evidence that ordinary discovery is insufficient or repeatedly costly>
   validated_at: <ISO date and environment in which the source was checked>
   invalidate_when: <observable source, version, configuration or ownership change>
   ```

5. **Return without persisting.** Read the primary source, confirm its current status and scope, and
   remove volatile state, secrets, personal data, prompt text and transcript material. Return the
   candidate to the user or calling workflow, state that it was not persisted, and keep
   `status: candidate`. Codex local memory under `~/.codex/memories/` is generated state, not a
   supported write surface for this skill. Never create, edit or remove files there or in another
   memory store. A future supported write path requires a separate design and implementation.
6. **Check freshness at consumption.** Load only candidates relevant to the active task. Every time
   before applying one, reread its primary source and verify its status, scope, exact revision and
   each observable `invalidate_when` condition. If any check fails, cannot run or is ambiguous,
   return `status: invalidated`, name the failed check and do not apply the candidate. If every check
   holds, use it only for the active task; do not promote or repersist it.
7. **Route behavior changes separately.** If the candidate would change agent instructions, skills,
   hooks or enforcement, use `harness-reflection`; memory evidence cannot promote itself into
   harness behavior.

## Gotchas

- **Confusing inconvenience with difficult discovery** — easy lookups accumulate into stale duplicate
  memory; reject an entry that a targeted source search resolves cheaply.
- **Treating generated Codex state as a write API** — a format change can corrupt or strand manually
  written entries; return the candidate and leave `~/.codex/memories/` untouched.
- **Saving the successful workaround** — an owned defect becomes doctrine and survives its fix;
  retain only an independently sourced invariant and route the defect to repair or an issue.
- **Using a time-to-live for a durable invariant** — periodic expiry recreates the investigation the
  memory exists to avoid; use an observable `invalidate_when` condition instead.
- **Generalizing from one repository** — a project contract becomes a false global preference;
  choose the smallest scope established by the source.
- **Treating a remembered summary as authority** — drift becomes invisible; the primary source wins
  whenever current evidence conflicts with the entry.

## Constraints

- Never persist anything from this pilot, including a workaround for a defect in code or
  configuration we own.
- Never create, edit or remove `~/.codex/memories/` files or write to another memory store.
- Never let memory replace, amend or outrank its primary source.
- Never persist raw transcripts, private prompts, secrets, credentials or personal data.
- Never admit an entry without `why_non_derivable`, current source validation and an observable
  `invalidate_when` condition.
- Never apply a prior candidate without rechecking its source revision and invalidation conditions.
- Never broaden scope beyond what the authority establishes.
- Never promote memory into harness behavior without the evidence and trial required by
  `harness-reflection`.
