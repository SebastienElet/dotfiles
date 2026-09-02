import { afterEach, expect, test } from "bun:test";
import {
  approvedManifest,
  cleanupTemporaryRoots,
  initializeRepository,
  mutationInput,
  registryPair,
  registryPath,
  temporaryRoot,
} from "./harness-reflection-mutation-production-test-support.ts";
import { dirname, join } from "node:path";
import { mkdir, readFile, writeFile } from "node:fs/promises";
import { createRepositoryMutationAdapter } from "./harness-reflection-mutation-filesystem.ts";
import { executeHarnessMutationWorkflowCore } from "./harness-reflection-mutation-workflow-core.ts";
import { parseInvariantRegistry } from "./invariant-registry-contract.ts";

const forbiddenSurfacePaths = [
  "README.md",
  "package.json",
  "tooling/harness-reflection-mutation-workflow-core.ts",
  "harness/skills/harness-reflection/references/invariant-registry.md",
] as const;

afterEach(cleanupTemporaryRoots);

test.each([...forbiddenSurfacePaths])(
  "refuses unsupported mutation destination %s before writing",
  async (path: (typeof forbiddenSurfacePaths)[number]) => {
    const root = await temporaryRoot();
    const pair = await registryPair();
    await initializeRepository(root, pair.active);
    await mkdir(dirname(join(root, path)), { recursive: true });
    await writeFile(join(root, path), "old surface", "utf8");
    const manifest = approvedManifest(pair);
    const [firstFile, registryFile] = manifest.files;
    if (firstFile === undefined || registryFile === undefined) {
      throw new Error("approved-manifest-files-missing");
    }
    const result = await executeHarnessMutationWorkflowCore(
      mutationInput(
        {
          ...manifest,
          files: [{ ...firstFile, path }, registryFile],
        },
        [
          { contents: "new surface", path },
          { contents: pair.retired, path: registryPath },
        ],
      ),
      createRepositoryMutationAdapter(root),
    );

    expect(result.status).toBe("rejected");
    expect(result.reason).toBe("unsupported-control-surface");
    expect(await readFile(join(root, path), "utf8")).toBe("old surface");
    expect(await readFile(join(root, registryPath), "utf8")).toBe(pair.active);
  },
);

test("refuses a consumer adapter that does not match the target surface", async () => {
  const root = await temporaryRoot();
  const pair = await registryPair();
  const active = pair.active.replace(
    "claude-global-instruction",
    "claude-user-skill",
  );
  const retired = pair.retired.replace(
    "claude-global-instruction",
    "claude-user-skill",
  );
  const [activeRecord] = parseInvariantRegistry(JSON.parse(active)).invariants;
  const [retiredRecord] = parseInvariantRegistry(
    JSON.parse(retired),
  ).invariants;
  if (activeRecord === undefined || retiredRecord === undefined) {
    throw new Error("surface-consumer-fixture-missing");
  }
  const mismatchedPair = {
    active,
    activeRecord,
    retired,
    retiredRecord,
  };
  await initializeRepository(root, active);

  const result = await executeHarnessMutationWorkflowCore(
    mutationInput(approvedManifest(mismatchedPair), [
      { contents: "new surface", path: "harness/AGENTS.md" },
      { contents: retired, path: registryPath },
    ]),
    createRepositoryMutationAdapter(root),
  );

  expect(result.status).toBe("rejected");
  expect(result.reason).toBe("mutation-consumer-surface-mismatch");
  expect(await readFile(join(root, "harness/AGENTS.md"), "utf8")).toBe(
    "old surface",
  );
  expect(await readFile(join(root, registryPath), "utf8")).toBe(active);
});
