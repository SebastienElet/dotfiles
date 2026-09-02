import {
  type HarnessMutationRequest,
  executeHarnessMutationWorkflow,
} from "./harness-reflection-mutation-workflow.ts";
import { expect, test } from "bun:test";
import {
  loadHarnessReflectionSources,
  parseHarnessReflectionContract,
} from "./harness-reflection-contract.ts";
import { readFile } from "node:fs/promises";
import { resolve } from "node:path";

const repositoryRoot = resolve(import.meta.dir, "..");
const registryPath = "harness/invariants/registry.json";
const surfacePath =
  "docs/superpowers/specs/2026-09-02-registre-invariants-harnais-design.md";
const approvedAt = "2026-09-02T00:00:00.000Z";

const invalidRegistryRequest = (): HarnessMutationRequest => ({
  approval: {
    approvedAt,
    approvedBy: "Reviewer",
    source: "human-context",
  },
  kind: "approved-mutation",
  preparedFiles: [
    { contents: "new surface", path: surfacePath },
    { contents: "not-json", path: registryPath },
  ],
  targetInvariantId: "prevent-secret-leaks",
});

test("resolves and invokes the production workflow through the skill route", async () => {
  const sources = await loadHarnessReflectionSources(repositoryRoot);
  const contract = parseHarnessReflectionContract(sources.reference);
  const route = contract.workflowRoutes.retirement;
  const workflowModule: unknown = await import(
    resolve(repositoryRoot, route.module)
  );
  if (typeof workflowModule !== "object" || workflowModule === null) {
    throw new TypeError("retirement-module-unresolved");
  }
  const routedWorkflow: unknown = Reflect.get(workflowModule, route.export);
  expect(routedWorkflow).toBe(executeHarnessMutationWorkflow);
  if (routedWorkflow !== executeHarnessMutationWorkflow) {
    throw new TypeError("retirement-route-unresolved");
  }
  const result = await executeHarnessMutationWorkflow(invalidRegistryRequest());
  expect(result.status).toBe("rejected");
});

test("rejects malformed prepared registry without writing any file", async () => {
  const surfaceBefore = await readFile(
    resolve(repositoryRoot, surfacePath),
    "utf8",
  );
  const registryBefore = await readFile(
    resolve(repositoryRoot, registryPath),
    "utf8",
  );

  let injectedCalls = 0;
  const unsafeRequest = {
    ...invalidRegistryRequest(),
    compareAndSwap: (): boolean => {
      injectedCalls += 1;
      return true;
    },
    validatePreparedRegistry: (): Readonly<Record<string, never>> => {
      injectedCalls += 1;
      return {};
    },
    validatePreparedSurfaces: (): void => {
      injectedCalls += 1;
    },
  };

  const result = await executeHarnessMutationWorkflow(unsafeRequest);

  expect(result.status).toBe("rejected");
  expect(result.reason).toBe("invariant registry must be valid JSON");
  expect(injectedCalls).toBe(0);
  expect(await readFile(resolve(repositoryRoot, surfacePath), "utf8")).toBe(
    surfaceBefore,
  );
  expect(await readFile(resolve(repositoryRoot, registryPath), "utf8")).toBe(
    registryBefore,
  );
});
