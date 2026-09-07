import {
  active,
  candidate,
  diagnosticCodes,
  registry,
  secondPullRequest,
  source,
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
  expect(() => registry(candidate({ lifecycle: "retired" }))).toThrow();
});

test.each(["candidate", "active"] as const)(
  "rejects retirement metadata on a %s invariant",
  (lifecycle) => {
    expect(() =>
      registry(
        candidate({
          lifecycle,
          retirement: {
            retiredAt: "2026-09-02T00:00:00.000Z",
            reason: "Invalid early retirement metadata.",
          },
        }),
      ),
    ).toThrow();
  },
);

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

test("rejects a replacement cycle", () => {
  const diagnostics = validateInvariantRegistry(
    registry(
      candidate({
        id: "first-invariant",
        lifecycle: "retired",
        retirement: {
          retiredAt: "2026-09-02T00:00:00.000Z",
          reason: "Replaced.",
          replacedBy: "second-invariant",
        },
      }),
      candidate({
        id: "second-invariant",
        lifecycle: "retired",
        retirement: {
          retiredAt: "2026-09-02T00:00:00.000Z",
          reason: "Replaced.",
          replacedBy: "first-invariant",
        },
        sources: [source(secondPullRequest)],
      }),
    ),
    validationOptions(),
  );

  expect(diagnosticCodes(diagnostics)).toContain("replacement-cycle");
});
