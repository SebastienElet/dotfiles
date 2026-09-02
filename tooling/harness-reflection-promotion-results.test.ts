import { expect, test } from "bun:test";
import {
  loadHarnessReflectionSources,
  validateHarnessReflectionContract,
} from "./harness-reflection-contract.ts";
import { promotionResultsSchema } from "./harness-reflection-promotion-results.ts";
import { resolve } from "node:path";

const repositoryRoot = resolve(import.meta.dir, "..");
const resultsPath = resolve(
  repositoryRoot,
  "harness/skills/harness-reflection/evals/promotion-workflow-results.json",
);
const sha256HexLength = 64;
type PromotionResults = ReturnType<typeof promotionResultsSchema.parse>;
const expectedNotCovered: PromotionResults["branchCoverage"]["notCovered"] = [
  "skip-missing-evidence",
  "link",
  "propose",
  "approval",
  "retirement",
  "promotion",
  "adr036-ablation",
];
const expectedLimitations: PromotionResults["limitations"] = [
  "evaluation-invalidated-by-skill-or-reference-change",
  "no-current-agent-workflow-runs",
  "no-current-behavioral-branch-coverage",
  "controlled-marginal-ablation-not-run",
  "accepted-cli-snapshot-is-not-durable-validity",
];
test("marks changed skill evaluation evidence pending with no runs", async () => {
  const results = promotionResultsSchema.parse(
    await Bun.file(resultsPath).json(),
  );

  expect(results.status).toBe("pending");
  expect(results.runs).toEqual([]);
  expect(results.branchCoverage.covered).toEqual([]);
  expect(results.branchCoverage.notCovered).toEqual(expectedNotCovered);
  expect(results.limitations).toEqual(expectedLimitations);
  expect(results.promotionEvidence).toBe(false);
  expect(results.adr036Ablation).toBe("not-run");
  expect(results.artifact.skillReference).toHaveLength(sha256HexLength);
  expect(Object.keys(results).toSorted()).toEqual([
    "adr036Ablation",
    "artifact",
    "branchCoverage",
    "criteria",
    "evaluationKind",
    "limitations",
    "promotionEvidence",
    "promptExact",
    "runs",
    "skill",
    "status",
    "version",
  ]);
});

test.each([
  [
    "wrong digest",
    {
      artifact: {
        algorithm: "sha256",
        skillReference: "0".repeat(sha256HexLength),
      },
    },
  ],
  ["inexact prompt", { promptExact: "changed prompt" }],
  ["weakened criteria", { criteria: { expectedRuns: 3, requiredPerRun: [] } }],
  ["non-pending status", { status: "recorded" }],
  ["top-level run provenance", { baseCommit: "untrusted" }],
  ["top-level covered path", { coveredPath: "skip-missing-evidence" }],
] as const)("rejects promotion results with %s", async (_name, patch) => {
  const sources = await loadHarnessReflectionSources(repositoryRoot);
  const promotionResults = structuredClone(sources.promotionResults);
  if (typeof promotionResults !== "object" || promotionResults === null) {
    throw new TypeError("promotion results missing");
  }
  Object.assign(promotionResults, patch);

  expect(
    validateHarnessReflectionContract({ ...sources, promotionResults }),
  ).not.toEqual([]);
});

test("rejects broader branch coverage or weakened limitations", async () => {
  const results = promotionResultsSchema.parse(
    await Bun.file(resultsPath).json(),
  );

  expect(
    promotionResultsSchema.safeParse({
      ...results,
      branchCoverage: { ...results.branchCoverage, covered: ["link"] },
    }).success,
  ).toBe(false);
  expect(
    promotionResultsSchema.safeParse({ ...results, limitations: [] }).success,
  ).toBe(false);
  expect(
    promotionResultsSchema.safeParse({
      ...results,
      runs: [{ result: "pass" }],
    }).success,
  ).toBe(false);
});
