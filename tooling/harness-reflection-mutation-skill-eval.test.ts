import { expect, test } from "bun:test";
import {
  promotionApproval,
  promotionRecords,
  record,
  request,
} from "./harness-reflection-mutation-test-support.ts";
import {
  promotionResultsSchema,
  skillReferenceDigest,
} from "./harness-reflection-promotion-results.ts";
import { resolve } from "node:path";
import { validateApprovedHarnessMutation } from "./harness-reflection-mutation-validation.ts";

const repositoryRoot = resolve(import.meta.dir, "..");
const skillPath = "harness/skills/harness-reflection/SKILL.md";
const referencePath =
  "harness/skills/harness-reflection/references/invariant-registry.md";
const evaluationPath =
  "harness/skills/harness-reflection/evals/promotion-workflow-results.json";
const sha256HexLength = 64;
const skillPreimage = await Bun.file(resolve(repositoryRoot, skillPath)).text();
const reference = await Bun.file(resolve(repositoryRoot, referencePath)).text();
const recordedResults: unknown = await Bun.file(
  resolve(repositoryRoot, evaluationPath),
).json();
const skillReplacement = `${skillPreimage}\nAlways validate external input before domain use.\n`;
const pendingResults = promotionResultsSchema.parse({
  ...(typeof recordedResults === "object" && recordedResults !== null
    ? recordedResults
    : {}),
  artifact: {
    algorithm: "sha256",
    skillReference: skillReferenceDigest(skillReplacement, reference),
  },
  branchCoverage: {
    covered: [],
    notCovered: [
      "skip-missing-evidence",
      "link",
      "propose",
      "approval",
      "retirement",
      "promotion",
      "adr036-ablation",
    ],
  },
  limitations: [
    "current-artifact-not-replayed",
    "no-current-behavioral-evidence",
    "link-propose-approval-retirement-and-promotion-not-exercised",
    "controlled-marginal-ablation-not-run",
    "accepted-cli-snapshot-is-not-durable-validity",
  ],
  runs: [],
  status: "pending",
});

const conditionalSkillRequest = (): ReturnType<typeof request> => {
  const records = promotionRecords();
  const consumers = {
    claude: {
      mechanism: "claude-user-skill" as const,
      state: "supported" as const,
    },
    codex: {
      mechanism: "codex-user-skill" as const,
      state: "supported" as const,
    },
    cursor: {
      mechanism: "cursor-user-skill" as const,
      state: "supported" as const,
    },
  };
  const before = record({
    ...records.before,
    consumers,
    surface: "conditional-skill",
  });
  const after = record({
    ...records.after,
    consumers,
    surface: "conditional-skill",
  });
  return request({
    after,
    approval: promotionApproval,
    before,
    files: [
      {
        path: skillPath,
        preimage: skillPreimage,
        replacement: skillReplacement,
      },
      {
        path: evaluationPath,
        preimage: JSON.stringify(recordedResults),
        replacement: JSON.stringify(pendingResults),
      },
    ],
  });
};

test("accepts a conditional-skill promotion with an exact pending eval reset", () => {
  expect(validateApprovedHarnessMutation(conditionalSkillRequest()).kind).toBe(
    "promotion",
  );
});

test("rejects a conditional-skill change without its eval reset", () => {
  const input = conditionalSkillRequest();
  const files = input.approval.manifest.files.filter(
    ({ path }) => path !== evaluationPath,
  );
  const preparedFiles = input.preparedFiles.filter(
    ({ path }) => path !== evaluationPath,
  );

  expect(() =>
    validateApprovedHarnessMutation({
      ...input,
      approval: {
        ...input.approval,
        manifest: { ...input.approval.manifest, files },
      },
      preparedFiles,
    }),
  ).toThrow("skill-evaluation-reset-required");
});

test("rejects a conditional-skill companion that remains recorded", () => {
  const input = conditionalSkillRequest();
  const stillRecorded = JSON.stringify({
    ...(typeof recordedResults === "object" && recordedResults !== null
      ? recordedResults
      : {}),
    artifact: {
      algorithm: "sha256",
      skillReference: "0".repeat(sha256HexLength),
    },
  });
  const files = input.approval.manifest.files.map((file) =>
    file.path === evaluationPath
      ? { ...file, replacement: stillRecorded }
      : file,
  );
  const preparedFiles = input.preparedFiles.map((file) =>
    file.path === evaluationPath ? { ...file, contents: stillRecorded } : file,
  );

  expect(() =>
    validateApprovedHarnessMutation({
      ...input,
      approval: {
        ...input.approval,
        manifest: { ...input.approval.manifest, files },
      },
      preparedFiles,
    }),
  ).toThrow("skill-evaluation-reset-required");
});
