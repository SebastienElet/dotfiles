---
name: issue-creation
description: >
  Draft, validate, review, and publish tracker issues across forges. Use when creating or checking
  an issue artifact. Make sure to use this skill whenever a request may publish an issue, even if
  the user names only GitHub, GitLab, Linear, a ticket, or a compatible future forge.
compatibility: Requires tracker read access for search or review; publication also requires write access.
metadata:
  category: dev
---

# Issue Creation

## Overview

Govern the provider-neutral lifecycle of one issue from target discovery through verified
publication. Keep domain content with specialist skills, while this skill owns duplicate search,
coherence, publication authority, the external write, and verification of the stored result.

Drafting, reviewing, and publishing are distinct modes. An explicit request to create or publish is
sufficient authority once the scope is coherent; a request only to draft or review never grants
publication authority.

## Usage

`/issue-creation <draft, existing issue, or publication request>`

Example: "Create this validated incident follow-up as a GitLab issue in group/project." Use the
user's language and the target project's canonical vocabulary and issue conventions.

## Workflow

1. **Select the mode and target.** Classify the request as draft, review, or publish. Identify the
   forge and repository, project, team, or workspace from explicit context or verified local
   configuration. Do not guess a target that could receive the external write. For review, record
   whether the input is a local draft or an existing remote issue.

2. **Verify the available forge path.** Read
   [references/forge-capabilities.md](references/forge-capabilities.md). Probe the current
   purpose-built connector first, then an authenticated official or established forge CLI. Confirm
   the exact search, create, retrieval, and field surfaces before relying on them. Never infer
   capability from a binary name, stale configuration, or historical usage.

3. **Compose specialist work by responsibility.** Activate any domain skill whose trigger applies
   and let it prepare the substantive content. For a Linear product issue, `linear-issue-spec` owns
   product evidence, personas, journey, functional slicing, visible states, and product acceptance
   criteria; this skill still owns the lifecycle gate and external publication. Composition is
   responsibility-based: either skill may activate first, and neither assumes that the other has
   already run.

4. **Search duplicates and adjacent work.** Search open and closed issues in the resolved target
   before every new publication. Use the outcome, primary deliverables, canonical domain terms,
   and neighboring vocabulary rather than only the proposed title. Inspect likely matches and
   record duplicates, overlaps, dependencies, and conflicting work. A probable duplicate blocks
   creation until the user chooses the existing issue or explicitly distinguishes the new outcome;
   continue any independent drafting while that decision is open. This search is a best-effort
   snapshot, not a uniqueness guarantee: it cannot serialize concurrent creators. Use a verified
   provider idempotency or uniqueness mechanism when one exists; otherwise report that concurrent
   duplicate creation remains possible.

5. **Prepare the candidate.** State the title, observable outcome, promised deliverables,
   requirements, established constraints, open questions, deliberate non-goals, and observable
   acceptance criteria. Mark proposals as proposals rather than established facts. Preserve the
   specialist's richer structure when applicable. Reject an empty or placeholder body unless the
   user explicitly requests a title-only issue; record that exception as part of the approved
   candidate.

6. **Run the coherence gate.** For an explicitly approved title-only candidate, verify that the
   title states the requested outcome and that no known contradiction changes its scope, then skip
   checks that require a body. Otherwise, build a short trace from each title promise and deliverable
   to at least one acceptance criterion that proves it. Compare the title, outcome, deliverables,
   requirements, open questions, and non-goals in both directions. Reject the candidate when a
   deliverable lacks proof, a non-goal negates the title, outcome, or deliverable, or an unresolved
   question materially changes scope. Remove limits of the current drafting session, such as not
   implementing during this session, from the implementation issue's non-goals. On contradiction,
   stop publication and ask for the smallest decision that resolves it.

7. **Apply publication authority.** A coherent explicit request to create, open, file, or publish
   the issue authorizes one external creation without a redundant confirmation. A draft-only or
   review-only request authorizes no write. If material scope remains uncertain, return the draft
   and blocking decision even when creation was requested. Reviewing an existing issue does not
   authorize editing it.

8. **Publish once.** Submit the validated title and complete body through the verified provider
   path, using a body file or structured connector fields when supported. Preserve intended labels,
   state, relationships, and target metadata. If the result is ambiguous because the response is
   lost or times out, search the target for the exact candidate and recent creations to recover a
   successful write. A search miss is not proof that the write failed. When no match is visible,
   report the result as indeterminate and stop; retry only through a verified provider idempotency
   or uniqueness mechanism that makes repeating the operation safe.

9. **Retrieve and verify.** Use a fresh read by returned identifier or URL, not the create response.
   Compare the stored title and body with the validated candidate and detect an empty, placeholder,
   or materially truncated body. Verify state, labels, relationships, target, and URL when the
   provider exposes them. Treat a missing issue or material mismatch as publication failure. Name
   every requested field the provider cannot return as `not verified`; never turn absence into a
   successful comparison.

10. **Report and stop.** Return the issue identifier and URL, duplicate-search result, fields
    verified, fields not verified, and any mismatch or provider limitation. In draft or review mode,
    state that nothing was published. Never begin implementing the created or reviewed issue unless
    the user makes a separate implementation request.

## Gotchas

- **Treating a create response as stored truth** — a proxy or provider can alter or drop fields;
  retrieve the issue independently and compare the stored result.
- **Retrying after an ambiguous write** — the first creation may have succeeded but not reached the
  search index; recover a visible match, or stop indeterminate unless a provider guard makes retry
  idempotent.
- **Treating duplicate search as uniqueness** — concurrent creators can both observe no match and
  publish; use a verified provider-side guard when available and report the residual race otherwise.
- **Making session limits into issue scope** — the future implementer receives a false non-goal;
  keep draft-session boundaries in the response and out of the issue body.
- **Letting a specialist publish** — lifecycle ownership becomes duplicated and confirmation rules
  depend on activation order; keep specialist output as content and route publication through this
  workflow.
- **Equating unavailable fields with matching fields** — verification becomes overstated; list each
  unsupported or absent field as not verified.

## Constraints

- Search open and closed issues for duplicates before every new issue publication.
- Never publish a materially contradictory, empty, placeholder, or unproven issue body unless an
  explicit title-only request makes the empty body intentional.
- Never require a second confirmation after a coherent explicit create or publish request.
- Never infer publication authority from drafting, refinement, or review alone.
- Never retry an ambiguous creation without a verified provider-side mechanism that prevents a
  second issue.
- Never claim verification for a field that was not returned by a fresh provider read.
- Never start implementing the issue without a separate request.

## References

- [references/forge-capabilities.md](references/forge-capabilities.md) — runtime capability probes,
  provider operations, and verification limits
