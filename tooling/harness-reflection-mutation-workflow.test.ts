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

const invalidRegistryRequest = async (): Promise<HarnessMutationRequest> => {
  const surfaceBefore = await readFile(
    resolve(repositoryRoot, surfacePath),
    "utf8",
  );
  const registryBefore = await readFile(
    resolve(repositoryRoot, registryPath),
    "utf8",
  );
  return {
    approval: {
      approvedAt,
      approvedBy: "Reviewer",
      manifest: {
        files: [
          {
            path: surfacePath,
            preimage: surfaceBefore,
            replacement: "new surface",
          },
          {
            path: registryPath,
            preimage: registryBefore,
            replacement: "not-json",
          },
        ],
        kind: "approved-mutation",
        registryDelta: {
          after: null,
          before: null,
          targetInvariantId: "prevent-secret-leaks",
        },
      },
      source: "human-context",
    },
    kind: "approved-mutation",
    preparedFiles: [
      { contents: "new surface", path: surfacePath },
      { contents: "not-json", path: registryPath },
    ],
    targetInvariantId: "prevent-secret-leaks",
  };
};

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
  const result = await executeHarnessMutationWorkflow(
    await invalidRegistryRequest(),
  );
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
    ...(await invalidRegistryRequest()),
    replaceMatching: (): boolean => {
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
  expect(result.reason).toBe("mutation-request-invalid");
  expect(injectedCalls).toBe(0);
  expect(await readFile(resolve(repositoryRoot, surfacePath), "utf8")).toBe(
    surfaceBefore,
  );
  expect(await readFile(resolve(repositoryRoot, registryPath), "utf8")).toBe(
    registryBefore,
  );
});
