---
name: memory-governance
description: >
  Govern durable cross-session memory for source-backed invariants that are expensive to rediscover.
  Use when a user or workflow asks to remember, persist, or reuse a hard-to-derive invariant across
  sessions. Make sure to use this skill whenever durable invariant memory is explicitly proposed,
  even if the request calls it notes or lessons learned.
metadata:
  category: ops
---

# Memory Governance

## Overview

Preserve durable invariants that are expensive to rediscover without turning observations into
authority. A memory entry is a sourced retrieval aid: it may spare repeated investigation, but the
primary ADR, contract, configuration or official dependency documentation remains authoritative.

## Usage

Use this workflow for requests such as "remember this invariant for future sessions", "record what
the harness should retain", or "stop rediscovering this accepted architectural constraint".

Do not use it for read-only retrieval from an existing Markdown corpus; use `obsidian-retrieval`.
When the durable destination is an instruction rather than memory, use `agent-instructions` and do
not create a duplicate memory entry.

## Steps

1. **Locate the authority and the store.** Identify the in-force primary source, its exact scope and
   an existing supported memory store. Do not invent a store, infer authority from a transcript, or
   write through an agent-specific projection. If no supported store exists, return a candidate
   entry without persisting it.
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
4. **Write the candidate in this exact shape:**

   ```yaml
   status: candidate
   statement: <one durable invariant>
   scope: <smallest project, domain, agent or user scope that owns it>
   authority: <ADR, contract, configuration, user decision or official dependency behavior>
   source: <stable locator plus revision, version or status when relevant>
   why_non_derivable: <evidence that ordinary discovery is insufficient or repeatedly costly>
   validated_at: <ISO date and environment in which the source was checked>
   invalidate_when: <observable source, version, configuration or ownership change>
   ```

5. **Validate before persistence.** Read the primary source, confirm its status and scope, remove
   volatile state, secrets, personal data, prompt text and transcript material, then change
   `status` to `validated`. A request to remember authorizes only a candidate that passes this
   contract and targets a known store.
6. **Retrieve narrowly.** Load only entries relevant to the active task. Use a validated entry
   without reopening its source merely because time passed; revalidate when `invalidate_when`
   occurs, the source is unavailable, current evidence contradicts it, or the task raises its
   assurance level. Mark a contradicted entry `invalidated` instead of silently rewriting it.
7. **Route behavior changes separately.** If the candidate would change agent instructions, skills,
   hooks or enforcement, use `harness-reflection`; memory evidence cannot promote itself into
   harness behavior.

## Gotchas

- **Confusing inconvenience with difficult discovery** — easy lookups accumulate into stale duplicate
  memory; reject an entry that a targeted source search resolves cheaply.
- **Saving the successful workaround** — an owned defect becomes doctrine and survives its fix;
  retain only an independently sourced invariant and route the defect to repair or an issue.
- **Using a time-to-live for a durable invariant** — periodic expiry recreates the investigation the
  memory exists to avoid; use an observable `invalidate_when` condition instead.
- **Generalizing from one repository** — a project contract becomes a false global preference;
  choose the smallest scope established by the source.
- **Treating a remembered summary as authority** — drift becomes invisible; the primary source wins
  whenever current evidence conflicts with the entry.

## Constraints

- Never persist a workaround for a defect in code or configuration we own.
- Never let memory replace, amend or outrank its primary source.
- Never persist raw transcripts, private prompts, secrets, credentials or personal data.
- Never admit an entry without `why_non_derivable`, current source validation and an observable
  `invalidate_when` condition.
- Never broaden scope beyond what the authority establishes.
- Never promote memory into harness behavior without the evidence and trial required by
  `harness-reflection`.
