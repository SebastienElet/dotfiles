import {
  active,
  candidate,
  diagnosticCodes,
  registry,
  validateInvariantRegistry,
  validationOptions,
} from "./invariant-registry-test-support.ts";
import { expect, test } from "bun:test";

const measuredVerification = {
  state: "measured",
  lastRun: {
    outcome: "passed",
    ranAt: "2026-09-02T00:00:00.000Z",
    environment: "macOS",
  },
};
test("rejects candidates that have already been measured", () => {
  const diagnostics = validateInvariantRegistry(
    registry(candidate({ verification: measuredVerification })),
    validationOptions(),
  );

  expect(diagnosticCodes(diagnostics)).toContain("candidate-measured");
});

test("rejects verified measurements that are not green during parsing", () => {
  expect(() =>
    registry(
      active({
        verification: {
          state: "verified",
          lastRun: {
            outcome: "failed",
            ranAt: "2026-09-02T00:00:00.000Z",
            environment: "macOS",
          },
        },
      }),
    ),
  ).toThrow();
});

test("rejects retired invariants without a retirement record", () => {
  const diagnostics = validateInvariantRegistry(
    registry(candidate({ lifecycle: "retired" })),
    validationOptions(),
  );

  expect(diagnosticCodes(diagnostics)).toContain("missing-retirement");
});

test("rejects replacements that do not identify an invariant", () => {
  const diagnostics = validateInvariantRegistry(
    registry(
      candidate({
        lifecycle: "retired",
        retirement: {
          retiredAt: "2026-09-02T00:00:00.000Z",
          reason: "Replaced by a better control.",
          replacedBy: "missing-invariant",
        },
      }),
    ),
    validationOptions(),
  );

  expect(diagnosticCodes(diagnostics)).toContain("unknown-replacement");
});

test("rejects replacements that identify the retired invariant itself", () => {
  const diagnostics = validateInvariantRegistry(
    registry(
      candidate({
        lifecycle: "retired",
        retirement: {
          retiredAt: "2026-09-02T00:00:00.000Z",
          reason: "Replaced by a better control.",
          replacedBy: "prevent-secret-leaks",
        },
      }),
    ),
    validationOptions(),
  );

  expect(diagnostics).toContainEqual({
    code: "self-replacement",
    path: "invariants.0.retirement.replacedBy",
    message: "Replacement invariant cannot be itself.",
  });
});
