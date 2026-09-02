import { expect, test } from "bun:test";
import {
  memoryAdapter,
  registryPath,
  retirementInput,
  retirementRegistryPair,
} from "./harness-reflection-mutation-workflow-test-support.ts";
import type { MutationWorkflowAdapter } from "./harness-reflection-mutation-workflow-types.ts";
import { executeHarnessMutationWorkflowCore } from "./harness-reflection-mutation-workflow-core.ts";

test("reconciles a CAS that writes before throwing", async () => {
  const pair = retirementRegistryPair();
  const base = memoryAdapter({
    [registryPath]: pair.current,
    "surface.md": "old surface",
  });
  let threwAfterWrite = false;
  const adapter: MutationWorkflowAdapter = {
    ...base,
    compareAndSwap: async (path, expected, replacement) => {
      const applied = await base.compareAndSwap(path, expected, replacement);
      if (!threwAfterWrite && path === "surface.md" && applied) {
        threwAfterWrite = true;
        throw new Error("ambiguous-write");
      }
      return applied;
    },
  };

  const result = await executeHarnessMutationWorkflowCore(
    retirementInput(pair.retired),
    adapter,
  );

  expect(result.status).toBe("compensated");
  expect(result.unresolvedPaths).toEqual([]);
  expect(base.contents.get("surface.md")).toBe("old surface");
  expect(base.contents.get(registryPath)).toBe(pair.current);
});

test("reports every path that cannot be restored", async () => {
  const pair = retirementRegistryPair();
  let applyConflict = false;
  const adapter = memoryAdapter(
    { [registryPath]: pair.current, "surface.md": "old surface" },
    (path, expected) => {
      if (
        !applyConflict &&
        path === registryPath &&
        expected === pair.current
      ) {
        applyConflict = true;
        adapter.contents.set(registryPath, "concurrent registry");
        adapter.contents.set("surface.md", "concurrent surface");
      }
    },
  );

  const result = await executeHarnessMutationWorkflowCore(
    retirementInput(pair.retired),
    adapter,
  );

  expect(result.status).toBe("compensation-incomplete");
  expect(result.unresolvedPaths).toEqual([registryPath, "surface.md"]);
  expect(adapter.contents.get("surface.md")).toBe("concurrent surface");
  expect(adapter.contents.get(registryPath)).toBe("concurrent registry");
});

test("compensates files changed before applied validation fails", async () => {
  const pair = retirementRegistryPair();
  let validations = 0;
  const base = memoryAdapter({
    [registryPath]: pair.current,
    "surface.md": "old surface",
  });
  const adapter: MutationWorkflowAdapter = {
    ...base,
    validatePreparedSurfaces: () => {
      validations += 1;
      if (validations > 1) {
        throw new Error("applied-surface-invalid");
      }
    },
  };

  const result = await executeHarnessMutationWorkflowCore(
    retirementInput(pair.retired),
    adapter,
  );

  expect(result.status).toBe("compensated");
  expect(base.contents.get("surface.md")).toBe("old surface");
  expect(base.contents.get(registryPath)).toBe(pair.current);
});
