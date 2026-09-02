import { afterEach, expect, test } from "bun:test";
import {
  approvedAt,
  approvedManifest,
  cleanupTemporaryRoots,
  initializeRepository,
  mutationInput,
  registryPair,
  registryPath,
  surfacePath,
  temporaryRoot,
} from "./harness-reflection-mutation-production-test-support.ts";
import type { MutationWorkflowCoreInput } from "./harness-reflection-mutation-workflow-types.ts";
import type { RegistryPair } from "./harness-reflection-mutation-production-test-support.ts";
import { createRepositoryMutationAdapter } from "./harness-reflection-mutation-filesystem.ts";
import { executeHarnessMutationWorkflowCore } from "./harness-reflection-mutation-workflow-core.ts";
import { join } from "node:path";
import { readFile } from "node:fs/promises";
import { renameSync } from "node:fs";

afterEach(cleanupTemporaryRoots);

type CompetingChange = Readonly<{
  registry: string;
  registryAfter: unknown;
  surface: string;
}>;

const competingInput = (
  pair: RegistryPair,
  change: CompetingChange,
): MutationWorkflowCoreInput =>
  mutationInput(
    approvedManifest(pair, change.registry, {
      registryAfter: change.registryAfter,
      surfaceReplacement: change.surface,
    }),
    [
      { contents: change.surface, path: surfacePath },
      { contents: change.registry, path: registryPath },
    ],
  );

const retiredWithReason = (pair: RegistryPair, reason: string): unknown => ({
  ...pair.retiredRecord,
  retirement: { reason, retiredAt: approvedAt },
});

test("allows exactly one concurrent adapter workflow to win", async () => {
  const root = await temporaryRoot();
  const pair = await registryPair();
  await initializeRepository(root, pair.active);
  const firstRecord = retiredWithReason(pair, "First retirement.");
  const secondRecord = retiredWithReason(pair, "Second retirement.");
  const firstRegistry = JSON.stringify({
    invariants: [firstRecord],
    version: 1,
  });
  const secondRegistry = JSON.stringify({
    invariants: [secondRecord],
    version: 1,
  });
  const [first, second] = await Promise.all([
    executeHarnessMutationWorkflowCore(
      competingInput(pair, {
        registry: firstRegistry,
        registryAfter: firstRecord,
        surface: "first surface",
      }),
      createRepositoryMutationAdapter(root),
    ),
    executeHarnessMutationWorkflowCore(
      competingInput(pair, {
        registry: secondRegistry,
        registryAfter: secondRecord,
        surface: "second surface",
      }),
      createRepositoryMutationAdapter(root),
    ),
  ]);

  expect(
    [first, second]
      .map(({ reason, status }) => `${status}:${reason ?? "none"}`)
      .toSorted(),
  ).toEqual(["rejected:mutation-lock-unavailable", "succeeded:none"]);
  const registryAfter = await readFile(join(root, registryPath), "utf8");
  const surfaceAfter = await readFile(join(root, surfacePath), "utf8");
  expect(
    (registryAfter === firstRegistry && surfaceAfter === "first surface") ||
      (registryAfter === secondRegistry && surfaceAfter === "second surface"),
  ).toBe(true);
});

test("compensates an error on the second file with the production adapter", async () => {
  const root = await temporaryRoot();
  const pair = await registryPair();
  await initializeRepository(root, pair.active);
  let registryRenameFailed = false;
  const adapter = createRepositoryMutationAdapter(root, {
    renameFile: (source, target) => {
      if (!registryRenameFailed && target.endsWith(registryPath)) {
        registryRenameFailed = true;
        throw new Error("injected-registry-rename-failure");
      }
      renameSync(source, target);
    },
  });
  const result = await executeHarnessMutationWorkflowCore(
    mutationInput(approvedManifest(pair), [
      { contents: "new surface", path: surfacePath },
      { contents: pair.retired, path: registryPath },
    ]),
    adapter,
  );

  expect(result.status).toBe("compensated");
  expect(result.unresolvedPaths).toEqual([]);
  expect(await readFile(join(root, surfacePath), "utf8")).toBe("old surface");
  expect(await readFile(join(root, registryPath), "utf8")).toBe(pair.active);
});
