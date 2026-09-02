import type {
  MutationWorkflowAdapter,
  MutationWorkflowCoreInput,
  WorkflowApproval,
} from "./harness-reflection-mutation-workflow-types.ts";
import {
  candidate,
  firstPullRequest,
  source,
} from "./invariant-registry-test-support.ts";
import { parseInvariantRegistry } from "./invariant-registry-contract.ts";

const registryPath = "harness/invariants/registry.json";
const approvedAt = "2026-09-02T00:00:00.000Z";
const reviewerApproval: WorkflowApproval = {
  approvedAt,
  approvedBy: "Reviewer",
  source: "human-context",
};
type RetirementRegistryPair = Readonly<{
  current: string;
  id: string;
  retired: string;
}>;
type MemoryAdapter = MutationWorkflowAdapter &
  Readonly<{ contents: Map<string, string> }>;

const retirementRegistryPair = (
  recordOverrides: Readonly<Record<string, unknown>> = {},
): RetirementRegistryPair => {
  const currentRecord = candidate({
    approval: { approvedAt, approvedBy: "Reviewer" },
    lifecycle: "active",
    severity: "high",
    sources: [source(firstPullRequest)],
    ...recordOverrides,
  });
  return {
    id: "prevent-secret-leaks",
    current: JSON.stringify({ invariants: [currentRecord], version: 1 }),
    retired: JSON.stringify({
      invariants: [
        {
          ...currentRecord,
          lifecycle: "retired",
          retirement: { reason: "Superseded.", retiredAt: approvedAt },
        },
      ],
      version: 1,
    }),
  };
};

const memoryAdapter = (
  initial: Readonly<Record<string, string>>,
  beforeCompare?: (
    path: string,
    expected: string | undefined,
    replacement: string | undefined,
  ) => void,
): MemoryAdapter => {
  const contents = new Map(Object.entries(initial));
  return {
    contents,
    read: (path) => contents.get(path),
    compareAndSwap: (path, expected, replacement) => {
      beforeCompare?.(path, expected, replacement);
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
    validatePreparedRegistry: (value) =>
      parseInvariantRegistry(JSON.parse(value)),
    validatePreparedSurfaces: () => Promise.resolve(),
  };
};

const retirementInput = (
  retired: string,
  overrides: Partial<MutationWorkflowCoreInput> = {},
): MutationWorkflowCoreInput => ({
  approval: reviewerApproval,
  kind: "retirement",
  preparedFiles: [
    { contents: "new surface", path: "surface.md" },
    { contents: retired, path: registryPath },
  ],
  registryPath,
  targetInvariantId: "prevent-secret-leaks",
  ...overrides,
});

export {
  approvedAt,
  memoryAdapter,
  registryPath,
  retirementInput,
  retirementRegistryPair,
  reviewerApproval,
  type MemoryAdapter,
};
