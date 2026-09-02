import {
  active,
  candidate,
  diagnosticCodes,
  firstPullRequest,
  registry,
  secondPullRequest,
  source,
  validateInvariantRegistry,
  validationOptions,
} from "./invariant-registry-test-support.ts";
import { expect, test } from "bun:test";

test.each([
  {
    causeClass: "unknown",
    name: "one ordinary PR",
    severity: "medium",
    sources: [source(firstPullRequest)],
  },
  {
    causeClass: "judgment",
    name: "judgment",
    severity: "high",
    sources: [source(firstPullRequest), source(secondPullRequest)],
  },
] as const)("refuses active promotion for $name", (testCase): void => {
  const diagnostics = validateInvariantRegistry(
    registry(
      active({
        causeClass: testCase.causeClass,
        severity: testCase.severity,
        sources: testCase.sources,
      }),
    ),
    validationOptions(),
  );

  expect(diagnostics).not.toEqual([]);
});

test("accepts two distinct PRs after explicit approval", () => {
  const diagnostics = validateInvariantRegistry(
    registry(
      active({
        sources: [source(firstPullRequest), source(secondPullRequest)],
      }),
    ),
    validationOptions(),
  );

  expect(diagnostics).toEqual([]);
});

test("accepts one high-severity PR after explicit approval", () => {
  const diagnostics = validateInvariantRegistry(
    registry(active({ severity: "high", sources: [source(firstPullRequest)] })),
    validationOptions(),
  );

  expect(diagnostics).toEqual([]);
});

test("normalizes GitHub pull request URLs before counting promotion evidence", () => {
  const canonicalSource = source(firstPullRequest);
  const diagnostics = validateInvariantRegistry(
    registry(
      active({
        sources: [
          canonicalSource,
          {
            ...canonicalSource,
            pullRequestUrl: `${canonicalSource.pullRequestUrl}/`,
          },
        ],
      }),
    ),
    validationOptions(),
  );

  expect(diagnostics).toContainEqual({
    code: "insufficient-promotion-evidence",
    path: "invariants.0.sources",
    message: "Active invariants require two pull requests or high severity.",
  });
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
