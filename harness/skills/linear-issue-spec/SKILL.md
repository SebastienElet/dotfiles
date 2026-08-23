---
name: linear-issue-spec
description: >
  Prepare implementation-ready Linear development issues as functional specifications. Use when
  scoping, drafting, splitting, or refining product work before implementation. Make sure to use
  this skill whenever an agent will receive a Linear issue, even if the user only asks to write or
  clean up the ticket.
compatibility: Requires access to the relevant Linear workspace and product or design sources.
metadata:
  category: product
---

# Linear Issue Preparation

## Overview

Prepare a development issue that defines a functionally complete product increment while leaving
technical discovery and implementation design to a fresh implementation agent. Establish the user
journey and product evidence before choosing scope, then optimize the slice for recognizable user
value and coordination cost rather than the smallest code change.

The deliverable is proposed issue content, not an implementation plan. Publishing or modifying the
issue in Linear is a separate external write and requires explicit authorization.

## Usage

`/linear-issue-spec <request, draft, or Linear issue>`

Examples: "Prepare the issue for adding invoice export", "refine ENG-482 before handing it to an
implementation agent", or "should these two adjacent onboarding tickets be merged?"

Provide the draft in the user's language. Preserve canonical product terms from their sources.

## Workflow

1. **Anchor the request and evidence.** Read the request or existing issue and identify its stated
   outcome, constraints, links, and unresolved questions. Locate the relevant personas, journey
   maps, specifications, decisions, research, analytics, and release plans. Record each source's
   status and recency; a proposal, draft, planned issue, shipped screen, and decision in force do not
   carry the same authority. If a required source is inaccessible, report that limitation instead
   of reconstructing its contents from hints.

2. **Identify the persona and journey.** State who experiences the problem, their goal, the entry
   point, the meaningful intermediate steps, and the observable end state. Follow the journey far
   enough in both directions to detect prerequisites, follow-up-only slices, and impacts on another
   persona. If the persona or intended outcome cannot be established, list it as missing information
   before drafting the issue.

3. **Map adjacent planned work.** Search open, planned, and recently completed Linear issues that
   affect the same journey, including issues using neighboring product vocabulary. Record overlaps,
   dependencies, sequencing assumptions, and potential duplicates. Treat issue descriptions as
   claims to reconcile with current product decisions, not as authority by themselves.

4. **Inspect the product surface.** Examine relevant designs, prototypes, design-system guidance,
   reusable components, shipped screens, and existing behavior elsewhere in the product. Use the
   current code only as evidence of observable behavior, product vocabulary, and existing product
   surfaces. Do not turn repository exploration into file recommendations, architecture, or a
   technical solution for the future implementer.

5. **Build an evidence ledger.** Classify every material requirement as one of:
   - `Requested`: explicitly stated in the current request or issue.
   - `Established`: supported by an existing design, specification, decision, or shipped product
     behavior; cite the source and its status.
   - `Proposed`: an assumption or recommendation introduced to close a gap; state why it is useful
     and keep it open to product confirmation.

   Surface contradictions, missing decisions, inaccessible evidence, and product dependencies
   before the proposed issue. A contradiction that changes the user-visible outcome or scope blocks
   finalization; ask for the smallest product decision that resolves it while continuing any
   independent preparation.

6. **Define functional completeness.** Describe the observable result, information and data users
   need to see, expected interactions, and relevant states. Explicitly assess initial, loading,
   empty, no-data, success, partial, error, retry, unavailable, and permission states, marking
   irrelevant ones as such rather than silently omitting them. Name applicable designs and existing
   design-system patterns without specifying how to implement them.

7. **Choose and challenge the slice.** Compare the proposed scope with adjacent issues and the full
   journey. Merge neighboring work when that removes meaningful coordination or review overhead
   without making the outcome difficult to understand or verify. Split when increments are
   independently usable, independently verifiable, and recognizable as intended product value.
   Before finalizing, answer:
   - Can a target user complete something meaningful with this increment?
   - Does it include known journey behavior rather than deferring it merely to shrink the ticket?
   - Would merging a neighboring issue materially reduce coordination and review cost?
   - Could the product owner recognize this as a coherent part of the expected product?

8. **Present the preparation in this order.** Start with `Missing information and contradictions`,
   including source status and impact. Then provide the issue draft with the structure below, and
   finish with `Why this scope`. Omit an empty preliminary section only after explicitly verifying
   that no material gap or contradiction remains.

   ```text
   Title

   Context and observable outcome
   Persona and user journey
   Scope
   Functional requirements, each labeled Requested, Established, or Proposed
   Required data and visible information
   User interactions
   States and permissions
   Design and existing product alignment
   Dependencies and related issues
   Deliberate out of scope, with product rationale
   Functional acceptance criteria
   Open product questions

   Why this scope
   ```

9. **Verify the handoff.** Ensure every acceptance criterion is observable by a user, product owner,
   or tester and collectively proves the increment complete. Remove implementation steps, file
   paths, architecture, classes, functions, library choices, and speculative technical constraints.
   Leave the implementation agent responsible for rediscovering the current code and selecting the
   technical approach at implementation time.

## Gotchas

- **Treating the initial ticket as the complete product truth** — adjacent journey requirements are
  missed and the slice becomes useful only to the next ticket. Reconcile it with personas, designs,
  shipped behavior, and neighboring work before choosing scope.
- **Converting code exploration into a prescribed solution** — the handoff anchors a fresh agent to
  stale file or architecture assumptions. Translate findings into observable product behavior and
  cite the evidence, leaving technical discovery out of the issue.
- **Collapsing empty, no-data, and error into one state** — distinct user situations receive an
  unverifiable fallback. Assess each important state explicitly and state when one is irrelevant.
- **Deferring known behavior to keep the diff small** — review happens twice while neither issue
  delivers recognizable value alone. Challenge the boundary against the whole journey and merge
  adjacent work when coordination savings are material.
- **Presenting a proposal as established fact** — product assumptions silently become acceptance
  criteria. Label provenance on every material requirement and expose unresolved proposals.
- **Publishing during preparation** — an external issue changes before the user reviews the scope.
  Return the draft first and write to Linear only after explicit authorization.

## Constraints

- Never prescribe architecture, files, classes, functions, libraries, detailed implementation
  steps, or a technical solution inferred from repository exploration.
- Never finalize a scope-changing contradiction as an implementation assumption.
- Never omit provenance for a material functional requirement: label it `Requested`, `Established`,
  or `Proposed` and cite established sources.
- Never optimize a slice solely for minimal code change; require independently usable and
  verifiable product value.
- Never defer a known journey state without an explicit product rationale or dependency.
- Never claim inaccessible Linear, design, or specification evidence was inspected.
- Never publish or modify a Linear issue unless the user explicitly authorizes that external write.
