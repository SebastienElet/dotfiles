---
name: tdd
description: >
  Develop owned behavior with test-driven development. Use when implementing features or bug fixes.
  Make sure to use this skill whenever owned executable behavior changes, even if tests are not
  requested. Excludes declarative changes without owned behavior; use native or existing validation.
license: MIT
metadata:
  category: dev
---

# Test-Driven Development

## Overview

Build one observable behavior at a time: red, green, then a small refactor while tests stay green.
Tests cross the public interface that owns the invariant, with expected outcomes independent of the
implementation. Repository instructions and ADRs govern the validation strategy.

Adapted from [tdd by Matt Pocock](https://github.com/mattpocock/skills/tree/d81f3a183412e71a5b1e84ca21bc1a35eea03a60/skills/engineering/tdd).
The upstream license is preserved in [LICENSE](LICENSE).

## Usage

Use `$tdd <behavior>` or `/tdd <behavior>`, or apply it during an authorized implementation.
For example: `Fix duplicate invoice creation and add the regression test first.` Ordinary relevant
tests are part of that authorization; do not ask again when the behavior and test boundary are known.

## Steps

1. Locate the affected behavior, consumers, authoritative requirements and existing tests. Read
   the domain glossary when present. Name the behavior and the public boundary that can detect its
   failure. Use `requirements-clarification` only for an unresolved material policy decision.
2. Choose the smallest relevant existing or native oracle. For configuration without owned
   behavior, use its parser, schema, dry-run or execution instead of a declaration mirror test.
   Consult `codebase-design` when the interface itself needs design, without expanding the scope.
3. Read [references/tests.md](references/tests.md) when choosing assertions and
   [references/dependencies.md](references/dependencies.md) when substituting dependencies.
   Exercise the real database or protocol when it owns the invariant; prefer in-memory application
   dependencies over mocks of internal calls.
4. Write one failing test for the next behavior. Include the affected failure paths. Run it and
   confirm the expected assertion fails because of the missing or broken behavior, rather than a
   setup or syntax error. A test already passing does not demonstrate the regression.
5. Implement only enough to pass that test. Run it again, then the existing checks relevant to
   the changed boundary. Repeat one behavior at a time rather than writing a speculative suite.
6. After green, simplify the code directly touched by the change while preserving its contract.
   Run the affected tests again. Use `code-simplify` for a separately requested refactoring pass.
7. Run the repository's required checks and report their actual environment, outcomes and limits.
   A missing integration environment is missing evidence; a passing unit substitute does not
   prove the database or protocol contract.

## Gotchas

- **Reconfirming an established test boundary** — routine work stalls; reuse the existing contract
  and authorization, asking only when a material decision remains unresolved.
- **Recomputing the expected value** — the test can repeat the same defect; use a worked example,
  requirement or independently known result.
- **Replacing the invariant owner with a mock** — integration failures disappear; run the actual
  database or protocol at the boundary that owns the guarantee.
- **Testing a declaration against itself** — maintenance grows without detecting a real defect;
  use native validation and follow the existing policy before proposing any new validation tooling.

## Constraints

- Observe the relevant failing behavior before changing owned implementation code.
- Never test private methods or internal call counts as a substitute for observable behavior.
- Never require additional permission for already authorized ordinary tests.
- Never add speculative abstractions, unrelated refactors, mirror gates or retries hiding flakes.
- Never claim a passing check covers an environment or boundary it did not exercise.
