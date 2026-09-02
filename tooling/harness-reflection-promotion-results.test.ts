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
const evaluatedCommit = "86941ff547dfc6e3bfe32301449f6f791529c124";
type PromotionResults = ReturnType<typeof promotionResultsSchema.parse>;
const expectedRunIdentities = [
  "/root/behavior_eval_13",
  "/root/behavior_eval_14",
  "/root/repo_readiness round behavior_eval_15",
];
const expectedNotCovered: PromotionResults["branchCoverage"]["notCovered"] = [
  "link",
  "propose",
  "approval",
  "retirement",
  "promotion",
  "adr036-ablation",
];
const expectedLimitations: PromotionResults["limitations"] = [
  "concrete-pull-request-urls-not-provided",
  "only-skip-missing-evidence-exercised",
  "link-propose-approval-retirement-and-promotion-not-exercised",
  "no-mutation-manifest-or-approval-produced",
  "no-control-surface-or-effective-oracle-selected",
  "claude-codex-and-cursor-consume-no-new-rule",
  "controlled-marginal-ablation-not-run",
  "accepted-cli-snapshot-is-not-durable-validity",
];
test("records exactly three successful skip-only workflow evaluations", async () => {
  const results = promotionResultsSchema.parse(
    await Bun.file(resultsPath).json(),
  );

  expect(results.status).toBe("recorded");
  expect(
    results.runs.map((run) =>
      "evaluation" in run ? `${run.agent} round ${run.evaluation}` : run.agent,
    ),
  ).toEqual(expectedRunIdentities);
  expect(results.runs.every((run) => run.baseCommit === evaluatedCommit)).toBe(
    true,
  );
  expect(
    results.runs.every((run) => Object.values(run.criteria).every(Boolean)),
  ).toBe(true);
  expect(results.branchCoverage.covered).toEqual(["skip-missing-evidence"]);
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
  ["non-recorded status", { status: "pending" }],
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
      runs: [...results.runs, results.runs[0]],
    }).success,
  ).toBe(false);
});

test("rejects changed provenance or one failed criterion", async () => {
  const results = promotionResultsSchema.parse(
    await Bun.file(resultsPath).json(),
  );

  expect(
    promotionResultsSchema.safeParse({
      ...results,
      runs: [
        { ...results.runs[0], baseCommit: "untrusted" },
        results.runs[1],
        results.runs[2],
      ],
    }).success,
  ).toBe(false);
  expect(
    promotionResultsSchema.safeParse({
      ...results,
      runs: [
        {
          ...results.runs[0],
          criteria: { ...results.runs[0].criteria, "mutation-refused": false },
        },
        results.runs[1],
        results.runs[2],
      ],
    }).success,
  ).toBe(false);
});
