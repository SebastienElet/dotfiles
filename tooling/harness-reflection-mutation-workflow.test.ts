import {
  type MutationFileAccess,
  type MutationWorkflowInput,
  executeHarnessMutationWorkflow,
} from "./harness-reflection-mutation-workflow.ts";
import {
  candidate,
  firstPullRequest,
  secondPullRequest,
  source,
} from "./invariant-registry-test-support.ts";
import { expect, test } from "bun:test";
import {
  loadHarnessReflectionSources,
  parseHarnessReflectionContract,
} from "./harness-reflection-contract.ts";
import { resolve } from "node:path";

const repositoryRoot = resolve(import.meta.dir, "..");
const registryPath = "harness/invariants/registry.json";
const approvedAt = "2026-09-02T00:00:00.000Z";
const successEventCount = 2;
type ReviewSource = ReturnType<typeof source>;
type RetirementRegistryPair = Readonly<{
  current: string;
  id: string;
  retired: string;
}>;

const isHarnessMutationWorkflow = (
  value: unknown,
): value is typeof executeHarnessMutationWorkflow =>
  value === executeHarnessMutationWorkflow;

const retirementRegistryPair = (
  sources: readonly ReviewSource[] = [source(firstPullRequest)],
): RetirementRegistryPair => {
  const active = candidate({
    approval: { approvedAt, approvedBy: "Reviewer" },
    lifecycle: "active",
    severity: "high",
    sources,
  });
  return {
    id: "prevent-secret-leaks",
    current: JSON.stringify({ invariants: [active], version: 1 }),
    retired: JSON.stringify({
      invariants: [
        {
          ...active,
          lifecycle: "retired",
          retirement: { reason: "Superseded.", retiredAt: approvedAt },
        },
      ],
      version: 1,
    }),
  };
};

const memoryFiles = (
  initial: Readonly<Record<string, string>>,
  beforeCompare?: (path: string, expected: string | undefined) => void,
): MutationFileAccess & { readonly contents: Map<string, string> } => {
  const contents = new Map(Object.entries(initial));
  return {
    contents,
    read: (path) => contents.get(path),
    compareAndSwap: (path, expected, replacement) => {
      beforeCompare?.(path, expected);
      if (contents.get(path) !== expected) {
        return false;
      }
      if (replacement === undefined) {
        contents.delete(path);
      } else {
        contents.set(path, replacement);
      }
      return true;
    },
  };
};

const workflowInput = (
  files: MutationFileAccess,
  retired: string,
): MutationWorkflowInput => ({
  approval: {
    approvedAt,
    approvedBy: "Reviewer",
    source: "human-context",
  },
  files,
  kind: "retirement",
  preparedFiles: [
    { contents: "new surface", path: "surface.md" },
    { contents: retired, path: registryPath },
  ],
  registryPath,
  retirementInvariantId: "prevent-secret-leaks",
  validateAppliedChange: () => Promise.resolve(),
  validatePreparedRegistry: (contents) => {
    expect(contents).toBe(retired);
  },
  validatePreparedSurfaces: () => Promise.resolve(),
});

test("resolves and invokes the retirement workflow through the skill route", async () => {
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
  if (!isHarnessMutationWorkflow(routedWorkflow)) {
    throw new TypeError("retirement-route-unresolved");
  }
  const pair = retirementRegistryPair();
  const files = memoryFiles({
    [registryPath]: pair.current,
    "surface.md": "old surface",
  });

  const result = await routedWorkflow(workflowInput(files, pair.retired));
  expect(result.status).toBe("succeeded");
  expect(result.events.slice(-successEventCount)).toEqual([
    "applied-change-validated",
    "success-rendered",
  ]);
  expect(files.contents.get("surface.md")).toBe("new surface");
  expect(files.contents.get(registryPath)).toBe(pair.retired);
});

test("compensates every applied file when a later CAS detects concurrency", async () => {
  const pair = retirementRegistryPair();
  let conflicted = false;
  const files = memoryFiles(
    { [registryPath]: pair.current, "surface.md": "old surface" },
    (path, expected) => {
      if (!conflicted && path === registryPath && expected === pair.current) {
        conflicted = true;
        files.contents.set(registryPath, "concurrent registry");
      }
    },
  );

  const result = await executeHarnessMutationWorkflow(
    workflowInput(files, pair.retired),
  );

  expect(result.status).toBe("compensated");
  expect(result.events).toEqual([
    "approval-accepted",
    "preimages-captured",
    "retirement-history-validated",
    "prepared-surfaces-validated",
    "prepared-registry-validated",
    "preimages-confirmed",
    "apply-started",
    "apply-conflict",
    "compensation-started",
    "compensation-completed",
  ]);
  expect(files.contents.get("surface.md")).toBe("old surface");
  expect(files.contents.get(registryPath)).toBe("concurrent registry");
});

test("reports an observable best-effort compensation limit", async () => {
  const pair = retirementRegistryPair();
  let applyConflict = false;
  const files = memoryFiles(
    { [registryPath]: pair.current, "surface.md": "old surface" },
    (path, expected) => {
      if (
        !applyConflict &&
        path === registryPath &&
        expected === pair.current
      ) {
        applyConflict = true;
        files.contents.set(registryPath, "concurrent registry");
        files.contents.set("surface.md", "concurrent surface");
      }
    },
  );

  const result = await executeHarnessMutationWorkflow(
    workflowInput(files, pair.retired),
  );

  expect(result.status).toBe("compensation-incomplete");
  expect(result.unresolvedPaths).toEqual(["surface.md"]);
  expect(files.contents.get("surface.md")).toBe("concurrent surface");
});

test("refuses retirement when any historical source is removed", async () => {
  const pair = retirementRegistryPair([
    source(firstPullRequest),
    source(secondPullRequest),
  ]);
  const proposed = retirementRegistryPair([source(firstPullRequest)]).retired;
  const files = memoryFiles({
    [registryPath]: pair.current,
    "surface.md": "old surface",
  });

  const result = await executeHarnessMutationWorkflow(
    workflowInput(files, proposed),
  );

  expect(result.status).toBe("rejected");
  expect(result.reason).toBe("retirement-source-history-changed");
  expect(files.contents.get("surface.md")).toBe("old surface");
  expect(files.contents.get(registryPath)).toBe(pair.current);
});

test("refuses an agent self-asserted approval before reading or mutating files", async () => {
  const pair = retirementRegistryPair();
  let reads = 0;
  const files = memoryFiles({
    [registryPath]: pair.current,
    "surface.md": "old surface",
  });
  const input = workflowInput(
    {
      ...files,
      read: (path) => {
        reads += 1;
        return files.read(path);
      },
    },
    pair.retired,
  );

  const result = await executeHarnessMutationWorkflow({
    ...input,
    approval: {
      approvedAt,
      approvedBy: "agent",
      source: "agent-self-asserted",
    },
  });

  expect(result.status).toBe("rejected");
  expect(result.reason).toBe("human-context-approval-required");
  expect(reads).toBe(0);
});
