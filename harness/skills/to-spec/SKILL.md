---
name: to-spec
description: >
  Synthesize the current conversation into a functional specification. Use only when the user
  explicitly invokes `$to-spec` or `/to-spec`; never select it implicitly from an implementation,
  interview or issue-publication request.
license: MIT
disable-model-invocation: true
metadata:
  category: product
---

# Conversation to Specification

## Overview

Turn the decisions already discussed into a functional specification with observable acceptance
criteria. Preserve the user's intended outcome and established constraints while leaving technical
discovery to the implementation agent. Drafting, saving and publishing remain distinct actions.

Adapted from [to-spec by Matt Pocock](https://github.com/mattpocock/skills/tree/d81f3a183412e71a5b1e84ca21bc1a35eea03a60/skills/engineering/to-spec).
The upstream license is preserved in [LICENSE](LICENSE).

## Usage

Invoke `$to-spec` in Codex or `/to-spec` in Claude Code and Cursor after discussing the change,
optionally with a requested destination. For example: `/to-spec Draft the invoice export spec
from our decisions.` The default output is a draft in the conversation. Saving requires a requested
file destination; tracker publication follows the authority and lifecycle in `issue-creation`.

## Steps

1. Extract the problem, intended outcome, users, decisions and unresolved points from the current
   conversation. Read linked product evidence, the domain glossary and relevant ADRs. Inspect
   existing behavior only to verify claims and terminology, not to prescribe implementation.
2. Synthesize what is established without starting another interview. Mark assumptions as proposals
   and missing material decisions as open questions. An unresolved contradiction affecting the
   outcome or scope prevents a final or implementation-ready spec; return the useful draft and
   identify the blocker instead of inventing a choice.
3. For a Linear development issue, use `linear-issue-spec` for product content, evidence, journey,
   visible states and functional slicing. Pass the settled conversation and mark unresolved points;
   do not duplicate its preparation procedure or turn synthesis into a new grilling session.
4. For other targets, use the requested or established project format. Otherwise write:

   ```text
   Title
   Problem and observable outcome
   Users and meaningful journey
   Functional requirements and established constraints
   Relevant states, failures, permissions and compatibility
   Observable acceptance criteria
   Dependencies and deliberate out of scope
   Open questions and proposed assumptions
   ```

   Size user stories to the actual feature. Cite established sources and distinguish requested,
   established and proposed requirements. Each acceptance criterion must describe observable
   behavior rather than a module, test seam or implementation step.

5. Check that the outcome, scope, requirements and acceptance criteria agree. Preserve architectural
   constraints already in force, but leave file paths, classes, libraries, module designs and testing
   plans to the future implementer. Never select one test boundary as an acceptance criterion.
6. Return the draft in the user's language. When a file destination is explicitly requested, save
   there using its existing documentation conventions. When an issue artifact or publication is
   requested, compose with `issue-creation`: it owns duplicate search, coherence, publication
   authority, writing and retrieval. Passing a draft does not grant publication permission or
   authorize a readiness label. Report the actual draft, saved file or verified issue state.

## Gotchas

- **Restarting the interview during synthesis** — settled decisions are re-opened; draft from the
  conversation and expose only the material unresolved questions.
- **Turning code discovery into implementation decisions** — the spec anchors a fresh agent to
  stale choices; state observable behavior and existing constraints instead.
- **Publishing on a bare command invocation** — a draft request creates an external issue; retain
  the draft default and let `issue-creation` evaluate the user's publication authority.
- **Copying Linear content preparation here** — two specification procedures diverge; compose with
  `linear-issue-spec` for that target and keep this skill focused on conversation synthesis.

## Constraints

- Activate only on explicit `$to-spec` or `/to-spec` invocation.
- Never invent a material decision or present an unresolved spec as implementation-ready.
- Never prescribe implementation architecture, files, classes, libraries or test topology.
- Never publish, apply readiness labels or bypass the `issue-creation` lifecycle from draft authority.
- Never start a new design interview or implement the specification from this invocation.
