import {
  active,
  candidate,
  diagnosticCodes,
  firstPullRequest,
  marginalAblation,
  registry,
  secondPullRequest,
  source,
  validateInvariantRegistry,
  validationOptions,
  verifiedVerification,
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

test("rejects active probabilistic controls without marginal ablation", () => {
  const diagnostics = validateInvariantRegistry(
    registry(
      active({
        controlKind: "probabilistic",
        oracle: undefined,
        surface: "always-loaded-instruction",
      }),
    ),
    validationOptions(),
  );

  expect(diagnosticCodes(diagnostics)).toContain("missing-marginal-ablation");
});

test("accepts active probabilistic controls after controlled ablation", () => {
  const diagnostics = validateInvariantRegistry(
    registry(
      active({
        controlKind: "probabilistic",
        marginalAblation,
        oracle: undefined,
        surface: "always-loaded-instruction",
        verification: verifiedVerification,
      }),
    ),
    validationOptions(),
  );

  expect(diagnostics).toEqual([]);
});

test("rejects unverified probabilistic ablation", () => {
  const diagnostics = validateInvariantRegistry(
    registry(
      active({
        controlKind: "probabilistic",
        marginalAblation,
        oracle: undefined,
        surface: "always-loaded-instruction",
      }),
    ),
    validationOptions(),
  );

  expect(diagnosticCodes(diagnostics)).toContain(
    "unverified-marginal-ablation",
  );
});

const ablationCases = [
  {
    name: "different scenarios",
    ablation: {
      ...marginalAblation,
      without: { ...marginalAblation.without, scenarios: ["other scenario"] },
    },
    code: "uncontrolled-marginal-ablation",
  },
  {
    name: "with-only outcome",
    ablation: {
      ...marginalAblation,
      without: {
        ...marginalAblation.without,
        outcomes: marginalAblation.with.outcomes,
      },
    },
    code: "missing-observable-delta",
  },
] as const;
for (const testCase of ablationCases) {
  test(`rejects probabilistic ablation with ${testCase.name}`, () => {
    const diagnostics = validateInvariantRegistry(
      registry(
        active({
          controlKind: "probabilistic",
          marginalAblation: testCase.ablation,
          oracle: undefined,
          surface: "always-loaded-instruction",
          verification: verifiedVerification,
        }),
      ),
      validationOptions(),
    );

    expect(diagnosticCodes(diagnostics)).toContain(testCase.code);
  });
}

test("rejects incomplete ablation replicate outcomes", () => {
  const diagnostics = validateInvariantRegistry(
    registry(
      active({
        controlKind: "probabilistic",
        marginalAblation: {
          ...marginalAblation,
          with: { ...marginalAblation.with, outcomes: ["pass"] },
        },
        oracle: undefined,
        surface: "always-loaded-instruction",
        verification: verifiedVerification,
      }),
    ),
    validationOptions(),
  );

  expect(diagnosticCodes(diagnostics)).toContain(
    "incomplete-ablation-replicates",
  );
});

test("requires effective activation measurement for conditional skills", () => {
  const diagnostics = validateInvariantRegistry(
    registry(
      active({
        controlKind: "probabilistic",
        marginalAblation: {
          ...marginalAblation,
          conditionalSkillActivation: {
            with: { activated: 0, total: 6 },
            without: { activated: 0, total: 6 },
          },
        },
        oracle: undefined,
        surface: "conditional-skill",
        verification: verifiedVerification,
      }),
    ),
    validationOptions(),
  );

  expect(diagnosticCodes(diagnostics)).toContain(
    "ineffective-activation-measurement",
  );
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
            pullRequestUrl:
              "https://github.com/sebastienelet/DOTFILES/pull/206/",
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
