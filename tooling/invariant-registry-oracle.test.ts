import {
  active,
  candidate,
  diagnosticCodes,
  firstPullRequest,
  oracle,
  registry,
  source,
  validateInvariantRegistry,
  validationOptions,
} from "./invariant-registry-test-support.ts";
import { expect, test } from "bun:test";

const verifiedVerification = {
  state: "verified",
  lastRun: {
    outcome: "passed",
    ranAt: "2026-09-02T00:00:00.000Z",
    environment: "macOS",
    oracle: {
      name: oracle.name,
      invocation: oracle.invocation,
      testPath: oracle.testPath,
    },
  },
};

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
    validationOptions(() => ({
      discovered: false,
      kind: "missing",
      tracked: false,
    })),
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
    validationOptions((path) => {
      paths.push(path);
      return { discovered: true, kind: "regular-file", tracked: true };
    }),
  );

  expect(paths).toEqual(["/repository/tooling/fetch-url-redaction.test.ts"]);
});

test("does not query an oracle path for a candidate", () => {
  let pathChecked = false;
  validateInvariantRegistry(
    registry(candidate({ controlKind: "enforceable", surface: "hook" })),
    validationOptions(() => {
      pathChecked = true;
      return { discovered: true, kind: "regular-file", tracked: true };
    }),
  );

  expect(pathChecked).toBeFalse();
});

test.each([
  { name: "absolute", testPath: "/outside/oracle.test.ts" },
  { name: "parent", testPath: "../outside/oracle.test.ts" },
  { name: "nested parent", testPath: "tooling/../../../oracle.test.ts" },
  { name: "Windows absolute", testPath: "C:\\outside\\oracle.test.ts" },
  { name: "Windows parent", testPath: "..\\outside\\oracle.test.ts" },
  { name: "Windows drive-relative", testPath: "C:outside\\oracle.test.ts" },
] as const)("rejects $name oracle path before probing", (testCase): void => {
  let pathChecked = false;
  const diagnostics = validateInvariantRegistry(
    registry(
      active({
        approval: undefined,
        sources: [source(firstPullRequest)],
        oracle: { ...oracle, testPath: testCase.testPath },
      }),
    ),
    validationOptions(() => {
      pathChecked = true;
      return { discovered: true, kind: "regular-file", tracked: true };
    }),
  );

  expect(pathChecked).toBeFalse();
  expect(diagnostics).toEqual([
    {
      code: "missing-approval",
      path: "invariants.0.approval",
      message: "Active invariants require explicit approval.",
    },
    {
      code: "insufficient-promotion-evidence",
      path: "invariants.0.sources",
      message: "Active invariants require two pull requests or high severity.",
    },
    {
      code: "invalid-oracle-path",
      path: "invariants.0.oracle.testPath",
      message: "Oracle test path must stay within the repository.",
    },
  ]);
});

test("translates oracle path check failures without dropping diagnostics", () => {
  const diagnostics = validateInvariantRegistry(
    registry(
      active({
        approval: undefined,
        sources: [source(firstPullRequest)],
        oracle,
      }),
    ),
    validationOptions(() => {
      throw new Error("filesystem unavailable");
    }),
  );

  expect(diagnostics).toEqual([
    {
      code: "missing-approval",
      path: "invariants.0.approval",
      message: "Active invariants require explicit approval.",
    },
    {
      code: "insufficient-promotion-evidence",
      path: "invariants.0.sources",
      message: "Active invariants require two pull requests or high severity.",
    },
    {
      code: "oracle-path-check-failed",
      path: "invariants.0.oracle.testPath",
      message: "Oracle test path could not be checked.",
    },
  ]);
});

test.each([
  {
    code: "non-regular-oracle-path",
    inspection: { discovered: false, kind: "non-regular", tracked: true },
    name: "directory",
  },
  {
    code: "untracked-oracle-path",
    inspection: { discovered: true, kind: "regular-file", tracked: false },
    name: "untracked file",
  },
  {
    code: "undiscovered-oracle-path",
    inspection: { discovered: false, kind: "regular-file", tracked: true },
    name: "non-test file",
  },
] as const)("rejects an oracle backed by a $name", (testCase) => {
  const diagnostics = validateInvariantRegistry(
    registry(active()),
    validationOptions(() => testCase.inspection),
  );

  expect(diagnosticCodes(diagnostics)).toContain(testCase.code);
});

test.each([
  {
    name: "oracle name",
    measurement: { name: "different-oracle" },
  },
  {
    name: "test path",
    measurement: { testPath: "tooling/different.test.ts" },
  },
  {
    name: "invocation",
    measurement: { invocation: ["bun", "test", "tooling/different.test.ts"] },
  },
] as const)(
  "rejects verified measurement for a different $name",
  (testCase) => {
    const diagnostics = validateInvariantRegistry(
      registry(
        active({
          verification: {
            ...verifiedVerification,
            lastRun: {
              ...verifiedVerification.lastRun,
              oracle: {
                ...verifiedVerification.lastRun.oracle,
                ...testCase.measurement,
              },
            },
          },
        }),
      ),
      validationOptions(),
    );

    expect(diagnosticCodes(diagnostics)).toContain(
      "oracle-measurement-mismatch",
    );
  },
);

test("rejects an invocation that does not run the declared oracle path", () => {
  const diagnostics = validateInvariantRegistry(
    registry(
      active({
        oracle: {
          ...oracle,
          invocation: ["bun", "test", "tooling/different.test.ts"],
        },
      }),
    ),
    validationOptions(),
  );

  expect(diagnosticCodes(diagnostics)).toContain("invalid-oracle-invocation");
});
