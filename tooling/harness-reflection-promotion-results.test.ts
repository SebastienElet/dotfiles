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
const sha1HexLength = 40;
const expectedReplayRuns = 3;
const firstReplayRun = 1;
const secondReplayRun = 2;
const replayRunNumbers = [
  firstReplayRun,
  secondReplayRun,
  expectedReplayRuns,
] as const;
const passingCriteria = {
  "factual-input-preserved": true,
  "missing-evidence-skip-selected": true,
  "mutation-refused": true,
  "registry-lookup-recorded": true,
  "report-rendered": true,
} as const;

test("keeps behavior evidence pending after the router contract changes", async () => {
  const results = promotionResultsSchema.parse(
    await Bun.file(resultsPath).json(),
  );

  expect(results.status).toBe("pending");
  expect(results.runs).toEqual([]);
  expect(results.branchCoverage.covered).toEqual([]);
  expect(results.branchCoverage.notCovered).toEqual([
    "skip-missing-evidence",
    "link",
    "propose",
    "approval",
    "retirement",
    "promotion",
    "adr036-ablation",
  ]);
  expect(results.promotionEvidence).toBe(false);
  expect(results.adr036Ablation).toBe("not-run");
  expect(results.artifact.skillReference).toHaveLength(sha256HexLength);
});

test("allows the artifact to become recorded only with three replay runs", async () => {
  const pending: unknown = await Bun.file(resultsPath).json();
  if (typeof pending !== "object" || pending === null) {
    throw new TypeError("pending-results-missing");
  }
  const runs = replayRunNumbers.map((run) => ({
    agent: `/root/replay_${run}`,
    baseCommit: "a".repeat(sha1HexLength),
    coveredPath: "skip-missing-evidence",
    criteria: passingCriteria,
    label: `behavior_eval_${run}`,
    result: "pass",
    run,
  }));

  expect(
    promotionResultsSchema.safeParse({
      ...pending,
      branchCoverage: {
        covered: ["skip-missing-evidence"],
        notCovered: [
          "link",
          "propose",
          "approval",
          "retirement",
          "promotion",
          "adr036-ablation",
        ],
      },
      limitations: [
        "concrete-pull-request-urls-not-provided",
        "only-skip-missing-evidence-exercised",
        "link-propose-approval-retirement-and-promotion-not-exercised",
        "no-mutation-manifest-or-approval-produced",
        "no-control-surface-or-effective-oracle-selected",
        "claude-codex-and-cursor-consume-no-new-rule",
        "controlled-marginal-ablation-not-run",
        "accepted-cli-snapshot-is-not-durable-validity",
      ],
      runs,
      status: "recorded",
    }).success,
  ).toBeTrue();
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
  [
    "weakened criteria",
    { criteria: { expectedRuns: expectedReplayRuns, requiredPerRun: [] } },
  ],
  ["recorded without replay runs", { status: "recorded" }],
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

test("rejects pending results with runs or claimed branch coverage", async () => {
  const results = promotionResultsSchema.parse(
    await Bun.file(resultsPath).json(),
  );

  expect(
    promotionResultsSchema.safeParse({
      ...results,
      runs: [{ result: "pass" }],
    }).success,
  ).toBeFalse();
  expect(
    promotionResultsSchema.safeParse({
      ...results,
      branchCoverage: {
        ...results.branchCoverage,
        covered: ["skip-missing-evidence"],
      },
    }).success,
  ).toBeFalse();
});
