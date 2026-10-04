# Behavioral Tests

Adapted from the upstream `tdd/tests.md`; the version and license are recorded in this skill.

Tests describe an observable contract through the interface owning it. Name the production defect
that would make the assertion fail before writing the test. A passing test whose oracle cannot
disagree with the implementation provides no useful protection.

## Independent expected results

Use a known example rather than copying the calculation into the expected value:

```typescript
test("total includes both invoice lines", () => {
  expect(calculateTotal([{ amount: 10 }, { amount: 5 }])).toBe(15);
});
```

The literal `15` is derived from the worked example. Recomputing it with the same reduction as
`calculateTotal` can repeat the same bug. Snapshots are appropriate only when their content has an
independent meaning and the review can detect a wrong result.

## Boundary and assertions

- Test outcomes callers rely on, including rejected inputs, stable error codes and ordering when
  order is part of the contract. A test may need several assertions for one coherent behavior.
- Read through the public interface for ordinary application behavior. When the database owns an
  invariant, direct database inspection or constraint exercise is a legitimate integration boundary.
- Use a real database for uniqueness, transactions and ORM mapping. An in-memory application
  dependency does not establish database semantics.
- Test pure domain behavior directly when the invariant belongs there. Broader integration tests
  do not automatically make focused domain tests redundant.
- Name tests according to the boundary actually assembled. In-process application integration with
  a real database and substituted external services is not automatically end-to-end testing.

## Failures that invalidate the evidence

Setup exceptions, syntax errors and a missing runtime are not the expected behavioral failure.
A test already green before the fix may characterize existing behavior, but it does not prove
that it detects the regression. Rerun the failing scenario after the fix as well as the focused test.

Assertions about internal call counts, private methods or implementation shape usually break
during harmless refactors. Check the observed result instead. For a protocol whose interaction
order is the invariant, test the actual protocol contract rather than a mock's incidental call order.
