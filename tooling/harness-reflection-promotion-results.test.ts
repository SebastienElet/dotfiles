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
const promotionBaseCommit = "a71390e07546a7169dc2bbe2e7d87104ba89240c";
type RecordedPromotionResults = Extract<
  ReturnType<typeof promotionResultsSchema.parse>,
  { status: "recorded" }
>;

const recordedRuns = (value: unknown): RecordedPromotionResults["runs"] => {
  const parsed = promotionResultsSchema.parse(value);
  if (parsed.status !== "recorded") {
    throw new TypeError("three recorded evaluation runs missing");
  }
  return parsed.runs;
};

test("records three complete post-correction promotion workflow evaluations", async () => {
  const results: unknown = await Bun.file(resultsPath).json();

  expect(results).toMatchObject({
    skill: "harness-reflection",
    status: "recorded",
    version: 1,
  });
  expect(results).toMatchObject({
    runs: [
      {
        baseCommit: promotionBaseCommit,
        criteria: {
          registryLookupRecorded: true,
          factualInputPreserved: true,
          immediateMutationRefused: true,
          controlAndOracleReported: true,
          claudeCodexCursorReported: true,
        },
        result: "pass",
      },
      {
        baseCommit: promotionBaseCommit,
        criteria: {
          registryLookupRecorded: true,
          factualInputPreserved: true,
          immediateMutationRefused: true,
          controlAndOracleReported: true,
          claudeCodexCursorReported: true,
        },
        result: "pass",
      },
      {
        baseCommit: promotionBaseCommit,
        criteria: {
          registryLookupRecorded: true,
          factualInputPreserved: true,
          immediateMutationRefused: true,
          controlAndOracleReported: true,
          claudeCodexCursorReported: true,
        },
        result: "pass",
      },
    ],
  });
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
  ["recorded without three runs", { status: "recorded", runs: [] }],
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

test("rejects a recorded evaluation with one missing criterion", async () => {
  const sources = await loadHarnessReflectionSources(repositoryRoot);
  const [firstRun, secondRun, thirdRun] = recordedRuns(
    sources.promotionResults,
  );
  const promotionResults = {
    ...promotionResultsSchema.parse(sources.promotionResults),
    runs: [
      firstRun,
      secondRun,
      {
        ...thirdRun,
        criteria: { ...thirdRun.criteria, claudeCodexCursorReported: false },
      },
    ],
  };

  expect(
    validateHarnessReflectionContract({ ...sources, promotionResults }),
  ).not.toEqual([]);
});

test("rejects a recorded evaluation from a different base commit", async () => {
  const sources = await loadHarnessReflectionSources(repositoryRoot);
  const [firstRun, secondRun, thirdRun] = recordedRuns(
    sources.promotionResults,
  );
  const promotionResults = {
    ...promotionResultsSchema.parse(sources.promotionResults),
    runs: [
      firstRun,
      { ...secondRun, baseCommit: "0".repeat(promotionBaseCommit.length) },
      thirdRun,
    ],
  };

  expect(
    validateHarnessReflectionContract({ ...sources, promotionResults }),
  ).not.toEqual([]);
});
