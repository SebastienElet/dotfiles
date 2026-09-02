import { expect, test } from "bun:test";

import {
  parseInvariantRegistry,
  validateInvariantRegistry,
} from "./invariant-registry-contract.ts";

const candidate = (overrides: Record<string, unknown> = {}) => ({
  id: "prevent-secret-leaks",
  statement: "Rejected fetch URLs never expose credentials.",
  lifecycle: "candidate",
  controlKind: "probabilistic",
  causeClass: "unknown",
  severity: "medium",
  sources: [
    {
      pullRequestUrl: "https://github.com/SebastienElet/dotfiles/pull/206",
      evidenceUrl:
        "https://github.com/SebastienElet/dotfiles/pull/206#issuecomment-5388129552",
    },
  ],
  scope: { kind: "cross-project", exceptions: [] },
  surface: "always-loaded-instruction",
  consumers: {
    claude: { state: "supported", mechanism: "always-loaded-instruction" },
    codex: { state: "supported", mechanism: "always-loaded-instruction" },
    cursor: { state: "unsupported", reason: "No managed instruction surface." },
  },
  verification: { state: "unverified" },
  ...overrides,
});

test("rejects unknown registry versions", () => {
  expect(() =>
    parseInvariantRegistry({ version: 2, invariants: [] }),
  ).toThrow();
});

test("rejects unknown lifecycle values and fields", () => {
  expect(() =>
    parseInvariantRegistry({
      version: 1,
      invariants: [{ ...candidate(), lifecycle: "enforced", extra: true }],
    }),
  ).toThrow();
});

test("requires separate Claude, Codex and Cursor declarations", () => {
  const record = candidate();
  const { cursor: _cursor, ...consumers } = record.consumers;

  expect(() =>
    parseInvariantRegistry({
      version: 1,
      invariants: [{ ...record, consumers }],
    }),
  ).toThrow();
});

const source = (number: number) => ({
  pullRequestUrl: `https://github.com/SebastienElet/dotfiles/pull/${number}`,
  evidenceUrl: `https://github.com/SebastienElet/dotfiles/pull/${number}#review`,
});

const active = (overrides: Record<string, unknown> = {}) =>
  candidate({
    lifecycle: "active",
    controlKind: "enforceable",
    surface: "hook",
    approval: {
      approvedBy: "Sebastien",
      approvedAt: "2026-09-02T00:00:00.000Z",
    },
    oracle: {
      name: "fetch-url-redaction",
      failurePath: "Rejected URLs do not expose credentials.",
      testPath: "tooling/fetch-url-redaction.test.ts",
    },
    sources: [source(206), source(207)],
    ...overrides,
  });

const registry = (...invariants: Record<string, unknown>[]) =>
  parseInvariantRegistry({ version: 1, invariants });

const validationOptions = (
  pathExists: (path: string) => boolean = () => true,
) => ({
  repositoryRoot: "/repository",
  pathExists,
});

const diagnosticCodes = (diagnostics: readonly { code: string }[]) =>
  diagnostics.map(({ code }) => code);

test.each([
  {
    causeClass: "unknown",
    name: "one ordinary PR",
    severity: "medium",
    sources: [source(206)],
  },
  {
    causeClass: "judgment",
    name: "judgment",
    severity: "high",
    sources: [source(206), source(207)],
  },
] as const)(
  "refuses active promotion for $name",
  ({ name: _name, ...testCase }) => {
    const diagnostics = validateInvariantRegistry(
      registry(active(testCase)),
      validationOptions(),
    );

    expect(diagnostics).not.toEqual([]);
  },
);

test("accepts two distinct PRs after explicit approval", () => {
  const diagnostics = validateInvariantRegistry(
    registry(active({ sources: [source(206), source(207)] })),
    validationOptions(),
  );

  expect(diagnostics).toEqual([]);
});

test("accepts one high-severity PR after explicit approval", () => {
  const diagnostics = validateInvariantRegistry(
    registry(active({ severity: "high", sources: [source(206)] })),
    validationOptions(),
  );

  expect(diagnostics).toEqual([]);
});

test("rejects duplicate identifiers", () => {
  const diagnostics = validateInvariantRegistry(
    registry(candidate(), candidate({ sources: [source(207)] })),
    validationOptions(),
  );

  expect(diagnosticCodes(diagnostics)).toContain("duplicate-id");
});

test("rejects review sources shared by multiple invariants", () => {
  const diagnostics = validateInvariantRegistry(
    registry(candidate(), candidate({ id: "different-invariant" })),
    validationOptions(),
  );

  expect(diagnosticCodes(diagnostics)).toContain("duplicate-source");
});

test("rejects incompatible control surfaces", () => {
  const diagnostics = validateInvariantRegistry(
    registry(candidate({ surface: "hook" })),
    validationOptions(),
  );

  expect(diagnosticCodes(diagnostics)).toContain("incompatible-surface");
});

test("rejects active invariants without explicit approval", () => {
  const diagnostics = validateInvariantRegistry(
    registry(active({ approval: undefined })),
    validationOptions(),
  );

  expect(diagnosticCodes(diagnostics)).toContain("missing-approval");
});

test("rejects candidates that have already been measured", () => {
  const diagnostics = validateInvariantRegistry(
    registry(
      candidate({
        verification: {
          state: "measured",
          lastRun: {
            outcome: "passed",
            ranAt: "2026-09-02T00:00:00.000Z",
            environment: "macOS",
          },
        },
      }),
    ),
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
    validationOptions(() => false),
  );

  expect(diagnosticCodes(diagnostics)).toContain("missing-oracle-path");
});

test("does not query an oracle path for a candidate", () => {
  let pathChecked = false;
  validateInvariantRegistry(
    registry(candidate({ controlKind: "enforceable", surface: "hook" })),
    validationOptions(() => {
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
