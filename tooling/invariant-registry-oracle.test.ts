import {
  active,
  candidate,
  diagnosticCodes,
  firstPullRequest,
  registry,
  source,
  validateInvariantRegistry,
  validationOptions,
} from "./invariant-registry-test-support.ts";
import { expect, test } from "bun:test";

const oracle = {
  name: "fetch-url-redaction",
  failurePath: "Rejected URLs do not expose credentials.",
  testPath: "tooling/fetch-url-redaction.test.ts",
};
const verifiedVerification = {
  state: "verified",
  lastRun: {
    outcome: "passed",
    ranAt: "2026-09-02T00:00:00.000Z",
    environment: "macOS",
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
    validationOptions((): boolean => {
      pathChecked = true;
      return true;
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
    validationOptions((): boolean => {
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
