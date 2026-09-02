import {
  type InvariantRecord,
  type InvariantRegistry,
  parseInvariantRegistry,
  validateInvariantRegistry,
} from "./invariant-registry-contract.ts";
import { afterEach, expect, test } from "bun:test";
import {
  applySyntheticSurface,
  cleanup,
  initializeFixture,
  readFixtureState,
  recordSyntheticRegistry,
  retirementRequestFrom,
  runFixtureCli,
  syntheticPromotionRequest,
  validateAppliedFixture,
} from "./harness-reflection-mutation-integration-test-support.ts";
import { readFile } from "node:fs/promises";
import { sourceSchema } from "./invariant-registry-source.ts";
import { validateApprovedHarnessMutation } from "./harness-reflection-mutation-validation.ts";
import { validationOptions } from "./invariant-registry-test-support.ts";

const fixturePath = (name: string): string =>
  `${import.meta.dir}/invariant-registry-fixtures/${name}`;

type SyntheticRequest = ReturnType<typeof syntheticPromotionRequest>;

const readFixtureRegistry = async (name: string): Promise<InvariantRegistry> =>
  parseInvariantRegistry(JSON.parse(await readFile(fixturePath(name), "utf8")));

const proposalRecord = (
  historical: InvariantRecord,
  approvedAt: string,
): InvariantRecord => {
  const withoutRetirement = { ...historical };
  Reflect.deleteProperty(withoutRetirement, "retirement");
  const [candidate] = parseInvariantRegistry({
    version: 1,
    invariants: [
      {
        ...withoutRetirement,
        approval: { approvedAt, approvedBy: "test-attestation" },
        lifecycle: "candidate",
        verification: { state: "unverified" },
      },
    ],
  }).invariants;
  if (candidate === undefined) {
    throw new TypeError("fixture-candidate-missing");
  }
  return candidate;
};

const proposalRequest = (after: InvariantRecord): unknown => {
  const beforeRegistry = JSON.stringify({ invariants: [], version: 1 });
  const afterRegistry = JSON.stringify({ invariants: [after], version: 1 });
  const registryFile = {
    path: "harness/invariants/registry.json",
    preimage: beforeRegistry,
    replacement: afterRegistry,
  };
  return {
    approval: {
      ...after.approval,
      manifest: {
        files: [registryFile],
        registryDelta: {
          after,
          before: null,
          targetInvariantId: after.id,
        },
      },
    },
    preparedFiles: [
      {
        contents: registryFile.replacement,
        path: registryFile.path,
        preimage: registryFile.preimage,
      },
    ],
    targetInvariantId: after.id,
  };
};

const requiredManifestFile = (
  request: Readonly<SyntheticRequest>,
  path: string,
): SyntheticRequest["approval"]["manifest"]["files"][number] => {
  const file = request.approval.manifest.files.find(
    (candidate) => candidate.path === path,
  );
  if (file === undefined) {
    throw new TypeError("fixture-manifest-file-missing");
  }
  return file;
};

const assertCliPassed = async (root: string): Promise<void> => {
  const outcome = await runFixtureCli(root);
  expect(outcome.exitCode).toBe(0);
  expect(outcome.stdout).toContain("Invariant registry passed");
};

const promoteFixture = async (
  root: string,
  request: Readonly<SyntheticRequest>,
): Promise<void> => {
  await applySyntheticSurface(root, request);
  await validateAppliedFixture(root, request);
  await recordSyntheticRegistry(root, request);
  await assertCliPassed(root);
};

const retireFixture = async (
  root: string,
  promotion: Readonly<SyntheticRequest>,
): Promise<SyntheticRequest> => {
  const retirement = retirementRequestFrom(promotion);
  await applySyntheticSurface(root, retirement);
  await validateAppliedFixture(root, retirement);
  const incomplete = await runFixtureCli(root);
  expect(incomplete.exitCode).not.toBe(0);
  expect(incomplete.stderr).toContain("declared oracle failed");
  await recordSyntheticRegistry(root, retirement);
  await assertCliPassed(root);
  return retirement;
};

const expectFixtureState = async (
  root: string,
  request: Readonly<SyntheticRequest>,
  lifecycle: "active" | "retired",
): Promise<void> => {
  const state = await readFixtureState(root);
  const registry = requiredManifestFile(
    request,
    "harness/invariants/registry.json",
  );
  const surface = requiredManifestFile(request, "harness/AGENTS.md");
  expect(state.source).toBe(registry.replacement);
  expect(state.surface).toBe(surface.replacement);
  expect(state.registry.invariants[0]?.lifecycle).toBe(lifecycle);
};

afterEach(cleanup);

test("historical PR 206 and PR 207 fixtures reach distinct candidate proposals", async () => {
  const [pr206, pr207] = await Promise.all([
    readFixtureRegistry("pr-206-secret-redaction.json"),
    readFixtureRegistry("pr-207-invalid-utf8.json"),
  ]);
  const [pr206Record] = pr206.invariants;
  const [pr207Record] = pr207.invariants;
  if (pr206Record === undefined || pr207Record === undefined) {
    throw new TypeError("historical-fixture-record-missing");
  }
  expect(pr206Record.sources[0]).toEqual({
    provider: "github",
    pullRequestUrl: "https://github.com/SebastienElet/dotfiles/pull/206",
    evidenceUrl:
      "https://github.com/SebastienElet/dotfiles/pull/206#issuecomment-5388129552",
  });
  expect(pr207Record.sources[0]).toEqual({
    provider: "github",
    pullRequestUrl: "https://github.com/SebastienElet/dotfiles/pull/207",
    evidenceUrl:
      "https://github.com/SebastienElet/dotfiles/pull/207#issuecomment-5388145825",
  });
  const candidates = [
    proposalRecord(pr206Record, "2026-09-03T10:00:00.000Z"),
    proposalRecord(pr207Record, "2026-09-03T10:01:00.000Z"),
  ];
  expect(
    candidates.map(
      (record) => validateApprovedHarnessMutation(proposalRequest(record)).kind,
    ),
  ).toEqual(["record-update", "record-update"]);
  expect(
    validateInvariantRegistry(
      parseInvariantRegistry({ invariants: candidates, version: 1 }),
      validationOptions(),
    ),
  ).toEqual([]);
});

test("synthetic local fixture completes promotion and retirement", async () => {
  const sourceFixture: unknown = JSON.parse(
    await readFile(fixturePath("synthetic-local-workflow.json"), "utf8"),
  );
  if (typeof sourceFixture !== "object" || sourceFixture === null) {
    throw new TypeError("synthetic-source-fixture-missing");
  }
  expect(Reflect.get(sourceFixture, "evidenceKind")).toBe(
    "synthetic-local-not-historical",
  );
  const request = syntheticPromotionRequest(
    sourceSchema.array().parse(Reflect.get(sourceFixture, "sources")),
  );
  const root = await initializeFixture(request);
  await promoteFixture(root, request);
  await expectFixtureState(root, request, "active");
  const retirement = await retireFixture(root, request);
  await expectFixtureState(root, retirement, "retired");
});
