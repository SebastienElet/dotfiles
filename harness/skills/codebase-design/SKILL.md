---
name: codebase-design
description: >
  Design cohesive modules with simple public interfaces. Use when deciding an interface, dependency
  boundary or test surface. Make sure to use this skill whenever another skill needs module-design
  guidance, even if architecture is unnamed. Excludes routine edits and separately requested cleanup.
license: MIT
metadata:
  category: dev
---

# Codebase Design

## Overview

A useful module hides meaningful complexity behind an interface callers can understand. Evaluate
depth by the behavior and decisions that interface provides, rather than line counts. Preserve
cohesion, local reasoning, testability and every boundary established by an ADR.

Adapted from [codebase-design by Matt Pocock](https://github.com/mattpocock/skills/tree/d81f3a183412e71a5b1e84ca21bc1a35eea03a60/skills/engineering/codebase-design).
The upstream license is preserved in [LICENSE](LICENSE).

## Usage

Use `$codebase-design <interface>` or `/codebase-design <interface>`, or consult it from `tdd`
when the test boundary requires design. For example: `Design the invoice repository interface
around the uniqueness invariant.` This is design guidance, not permission to reorganize a codebase.

## Steps

1. Locate the consumers, affected domain contract and ADRs in force. Bound the interface under
   discussion. Use the project's canonical vocabulary; module, interface, adapter and test boundary
   are explanatory terms, not mandatory replacements for domain or framework names.
2. Describe what callers must know: inputs, results, invariants, error modes, ordering, required
   configuration and relevant performance constraints. The type signature alone is not the contract.
3. Apply the deletion test: if the module disappeared, would its decisions spread across callers
   or would unnecessary delegation disappear? Keep functions and adapters with a real responsibility
   even when they have one caller or one implementation.
4. Keep the public surface small and specific to actual use cases. Hide cohesive implementation
   details, accept dependencies with effects at the appropriate boundary and prefer pure transformations
   for the owned core. Avoid hypothetical ports, boolean switches and generic operation handlers.
5. Read [references/dependencies.md](references/dependencies.md) when considering dependency
   boundaries or consolidation. State where each invariant is enforced and test it there. Do not
   collapse domain, transport or transaction boundaries merely to make a module deeper.
6. Compare alternatives only when a consequential design choice remains. State the gain in
   locality, caller effort and testability; separate a contract change from a behavior-preserving
   restructuring. Use `requirements-clarification` for unresolved material policy, and
   `code-simplify` for an authorized cleanup implementation.
7. Before replacing tests or interfaces, inventory existing guarantees and independent consumers.
   Preserve their coverage until replacement checks exercise the same required behavior, including
   failures. Follow repository size limits rather than expanding one file without a cohesion limit.

## Gotchas

- **Depth measured by implementation size** — padding looks like progress; judge the useful
  behavior exposed and complexity removed from callers.
- **One implementation mistaken for a useless boundary** — an architectural invariant is lost;
  inspect the ADR and responsibility before proposing removal.
- **Replacing domain words with a skill glossary** — terminology diverges; retain the project's
  canonical names and explain design concepts in that vocabulary.
- **Deleting unit tests as soon as integration tests exist** — uncovered invariants disappear;
  compare behavioral coverage before retiring the old checks.

## Constraints

- An ADR in force and the domain contract take precedence over depth heuristics.
- Never introduce an abstraction solely for hypothetical future consumers or adapters.
- Never merge responsibilities solely to reduce the number of files or methods.
- Never erase a test boundary or its coverage solely because a broader test now exists.
- Design discussion does not authorize unrelated implementation or architectural migration.
