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
const verifiedVerification = {
  state: "verified",
  lastRun: {
    outcome: "passed",
    ranAt: "2026-09-02T00:00:00.000Z",
    environment: "macOS",
  },
};
const oracle = {
  name: "fetch-url-redaction",
  failurePath: "Rejected URLs do not expose credentials.",
  testPath: "tooling/fetch-url-redaction.test.ts",
};

test("rejects candidates that have already been measured", () => {
  const diagnostics = validateInvariantRegistry(
    registry(candidate({ verification: measuredVerification })),
    validationOptions(),
  );

  expect(diagnosticCodes(diagnostics)).toContain("candidate-measured");
});

test("rejects active enforceable invariants without an oracle", () => {
  const diagnostics = validateInvariantRegistry(
    registry(active({ oracle: undefined })),
    validationOptions(),
  );

  expect(diagnosticCodes(diagnostics)).toContain("missing-oracle");
});

test("rejects active enforceable invariants when the oracle path is absent", () => {
  const diagnostics = validateInvariantRegistry(
    registry(active()),
    validationOptions((): boolean => false),
  );

  expect(diagnosticCodes(diagnostics)).toContain("missing-oracle-path");
});

test("checks an enforceable verified oracle path", () => {
  const paths: string[] = [];
  validateInvariantRegistry(
    registry(
      candidate({
        controlKind: "enforceable",
        surface: "hook",
        verification: verifiedVerification,
        oracle,
      }),
    ),
    validationOptions((path): boolean => {
      paths.push(path);
      return true;
    }),
  );

  expect(paths).toEqual(["/repository/tooling/fetch-url-redaction.test.ts"]);
});

test("does not query an oracle path for a candidate", () => {
  let pathChecked = false;
  validateInvariantRegistry(
    registry(candidate({ controlKind: "enforceable", surface: "hook" })),
    validationOptions((): boolean => {
      pathChecked = true;
      return true;
    }),
  );

  expect(pathChecked).toBeFalse();
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
