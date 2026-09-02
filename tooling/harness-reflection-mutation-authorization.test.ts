import { afterEach, expect, test } from "bun:test";
import {
  approvedManifest,
  cleanupTemporaryRoots,
  initializeRepository,
  mutationInput,
  registryPair,
  registryPath,
  surfacePath,
  temporaryRoot,
} from "./harness-reflection-mutation-production-test-support.ts";
import {
  candidate,
  secondPullRequest,
  source,
} from "./invariant-registry-test-support.ts";
import { link, readFile, rm, writeFile } from "node:fs/promises";
import { createRepositoryMutationAdapter } from "./harness-reflection-mutation-filesystem.ts";
import { executeHarnessMutationWorkflowCore } from "./harness-reflection-mutation-workflow-core.ts";
import { join } from "node:path";

afterEach(cleanupTemporaryRoots);

test("refuses a path outside the approved manifest before writing", async () => {
  const root = await temporaryRoot();
  const pair = await registryPair();
  await initializeRepository(root, pair.active);
  const adapter = createRepositoryMutationAdapter(root);
  const result = await executeHarnessMutationWorkflowCore(
    mutationInput(approvedManifest(pair), [
      { contents: "unrelated after", path: "unrelated.txt" },
      { contents: pair.retired, path: registryPath },
    ]),
    adapter,
  );

  expect(result.status).toBe("rejected");
  expect(await readFile(join(root, "unrelated.txt"), "utf8")).toBe(
    "unrelated before",
  );
  expect(await readFile(join(root, registryPath), "utf8")).toBe(pair.active);
});

test("refuses a second invariant outside the approved registry delta", async () => {
  const root = await temporaryRoot();
  const pair = await registryPair();
  await initializeRepository(root, pair.active);
  const secondInvariant = candidate({
    id: "second-invariant",
    sources: [source(secondPullRequest)],
    statement: "A second invariant remains unchanged.",
  });
  const registryWithSecondInvariant = JSON.stringify({
    invariants: [pair.retiredRecord, secondInvariant],
    version: 1,
  });
  const adapter = createRepositoryMutationAdapter(root);
  const result = await executeHarnessMutationWorkflowCore(
    mutationInput(approvedManifest(pair, registryWithSecondInvariant), [
      { contents: "new surface", path: surfacePath },
      { contents: registryWithSecondInvariant, path: registryPath },
    ]),
    adapter,
  );

  expect(result.status).toBe("rejected");
  expect(await readFile(join(root, surfacePath), "utf8")).toBe("old surface");
  expect(await readFile(join(root, registryPath), "utf8")).toBe(pair.active);
});

test("refuses an internal hardlink without changing its external inode", async () => {
  const root = await temporaryRoot();
  const externalRoot = await temporaryRoot();
  const pair = await registryPair();
  await initializeRepository(root, pair.active);
  const externalPath = join(externalRoot, "external.md");
  await writeFile(externalPath, "external contents", "utf8");
  await rm(join(root, surfacePath));
  await link(externalPath, join(root, surfacePath));

  const result = await executeHarnessMutationWorkflowCore(
    mutationInput(approvedManifest(pair), [
      { contents: "new surface", path: surfacePath },
      { contents: pair.retired, path: registryPath },
    ]),
    createRepositoryMutationAdapter(root),
  );

  expect(result.status).toBe("rejected");
  expect(result.reason).toBe("mutation-path-hard-linked");
  expect(await readFile(externalPath, "utf8")).toBe("external contents");
  expect(await readFile(join(root, registryPath), "utf8")).toBe(pair.active);
});
