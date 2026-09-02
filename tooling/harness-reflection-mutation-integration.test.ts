import { afterEach, expect, test } from "bun:test";
import {
  cleanup as cleanupCli,
  runRegistryCli,
} from "./invariant-registry-cli.test-support.ts";
import { join, relative, resolve } from "node:path";
import { mkdir, mkdtemp, readFile, rm, writeFile } from "node:fs/promises";
import {
  oraclePath,
  promotionFixture,
  promotionRequest,
  registryPath,
  retirementFixture,
  retirementRequest,
  surfacePath,
} from "./harness-reflection-mutation-integration-test-support.ts";
import { createRepositoryMutationAdapter } from "./harness-reflection-mutation-filesystem.ts";
import { executeHarnessMutationWorkflowCore } from "./harness-reflection-mutation-workflow-core.ts";

const repositoryRoot = resolve(import.meta.dir, "..");
const fixtureRoot = join(repositoryRoot, "tooling/invariant-registry-fixtures");
const temporaryRoots: string[] = [];

const initializeFixture = async (registry: string): Promise<string> => {
  const root = await mkdtemp(join(fixtureRoot, ".workflow-"));
  temporaryRoots.push(root);
  await mkdir(join(root, "harness/invariants"), { recursive: true });
  await mkdir(join(root, "tooling"), { recursive: true });
  await writeFile(join(root, registryPath), registry, "utf8");
  await writeFile(join(root, surfacePath), "old guidance", "utf8");
  await writeFile(
    join(root, oraclePath),
    await readFile(join(repositoryRoot, oraclePath), "utf8"),
    "utf8",
  );
  expect(Bun.spawnSync(["git", "-C", root, "init", "--quiet"]).exitCode).toBe(
    0,
  );
  expect(Bun.spawnSync(["git", "-C", root, "add", oraclePath]).exitCode).toBe(
    0,
  );
  return root;
};

const verifyCliAndOracle = async (root: string): Promise<void> => {
  const cliPath = relative(repositoryRoot, join(root, registryPath));
  expect(await runRegistryCli(cliPath)).toEqual({
    exitCode: 0,
    stderr: "",
    stdout: `Invariant registry passed: ${cliPath}\n`,
  });
  expect(
    Bun.spawnSync(["bun", "test", oraclePath], { cwd: repositoryRoot })
      .exitCode,
  ).toBe(0);
};

afterEach(async () => {
  await cleanupCli();
  await Promise.all(
    temporaryRoots
      .splice(0)
      .map((root) => rm(root, { force: true, recursive: true })),
  );
});

test("runs approved promotion, production mutation, CLI oracle and retirement", async () => {
  const promotion = promotionFixture();
  const root = await initializeFixture(promotion.candidateRegistry);
  const promoted = await executeHarnessMutationWorkflowCore(
    promotionRequest(promotion),
    createRepositoryMutationAdapter(root),
  );
  expect(promoted.status).toBe("succeeded");
  await verifyCliAndOracle(root);

  const retirement = retirementFixture(promotion);
  const retired = await executeHarnessMutationWorkflowCore(
    retirementRequest(promotion, retirement),
    createRepositoryMutationAdapter(root),
  );
  expect(retired.status).toBe("succeeded");
  await verifyCliAndOracle(root);
  expect(retirement.record.sources).toEqual(promotion.activeRecord.sources);
  expect(retirement.record.scope.exceptions).toEqual(
    promotion.activeRecord.scope.exceptions,
  );
  expect(
    JSON.parse(await readFile(join(repositoryRoot, registryPath), "utf8")),
  ).toEqual({ invariants: [], version: 1 });
});
