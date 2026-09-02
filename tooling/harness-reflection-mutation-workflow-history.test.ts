import {
  approvedAt,
  memoryAdapter,
  registryPath,
  retirementInput,
  retirementRegistryPair,
} from "./harness-reflection-mutation-workflow-test-support.ts";
import { expect, test } from "bun:test";
import {
  firstPullRequest,
  secondPullRequest,
  source,
} from "./invariant-registry-test-support.ts";
import { executeHarnessMutationWorkflowCore } from "./harness-reflection-mutation-workflow-core.ts";

test("refuses retirement when any historical source is removed", async () => {
  const pair = retirementRegistryPair({
    sources: [source(firstPullRequest), source(secondPullRequest)],
  });
  const proposed = retirementRegistryPair({
    sources: [source(firstPullRequest)],
  }).retired;
  const adapter = memoryAdapter({
    [registryPath]: pair.current,
    "surface.md": "old surface",
  });

  const result = await executeHarnessMutationWorkflowCore(
    retirementInput(pair.current, proposed),
    adapter,
  );

  expect(result.status).toBe("rejected");
  expect(result.reason).toBe("retirement-history-changed");
  expect(adapter.contents.get(registryPath)).toBe(pair.current);
});

test("refuses retirement when a scope exception is removed", async () => {
  const pair = retirementRegistryPair({
    scope: {
      exceptions: [{ paths: ["legacy/**"], reason: "Legacy boundary." }],
      kind: "cross-project",
    },
  });
  const proposed = pair.retired.replace(
    '"exceptions":[{"paths":["legacy/**"],"reason":"Legacy boundary."}]',
    '"exceptions":[]',
  );
  const adapter = memoryAdapter({
    [registryPath]: pair.current,
    "surface.md": "old surface",
  });

  const result = await executeHarnessMutationWorkflowCore(
    retirementInput(pair.current, proposed),
    adapter,
  );

  expect(result.status).toBe("rejected");
  expect(result.reason).toBe("retirement-history-changed");
  expect(adapter.contents.get(registryPath)).toBe(pair.current);
});

test("refuses a persisted approval different from the accepted context", async () => {
  const pair = retirementRegistryPair();
  const persisted = pair.retired.replaceAll("Reviewer", "Mallory");
  const adapter = memoryAdapter({
    [registryPath]: pair.current,
    "surface.md": "old surface",
  });

  const result = await executeHarnessMutationWorkflowCore(
    retirementInput(pair.current, persisted, {
      approval: {
        approvedAt,
        approvedBy: "Alice",
        source: "human-context",
      },
    }),
    adapter,
  );

  expect(result.status).toBe("rejected");
  expect(result.reason).toBe("prepared-registry-approval-mismatch");
  expect(adapter.contents.get(registryPath)).toBe(pair.current);
});

test("refuses an agent self-asserted approval before reading files", async () => {
  const pair = retirementRegistryPair();
  let reads = 0;
  const base = memoryAdapter({
    [registryPath]: pair.current,
    "surface.md": "old surface",
  });
  const adapter = {
    ...base,
    read: async (path: string): Promise<string | undefined> => {
      reads += 1;
      const contents = await base.read(path);
      return contents;
    },
  };

  const result = await executeHarnessMutationWorkflowCore(
    retirementInput(pair.current, pair.retired, {
      approval: {
        approvedAt,
        approvedBy: "agent",
        source: "agent-self-asserted",
      },
    }),
    adapter,
  );

  expect(result.status).toBe("rejected");
  expect(result.reason).toBe("human-context-approval-required");
  expect(reads).toBe(0);
});
