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
import { parseMutationManifest } from "./harness-reflection-mutation-workflow-types.ts";

const registryPath = "harness/invariants/registry.json";
const approvedAt = "2026-09-02T00:00:00.000Z";
const reviewerApproval: Omit<WorkflowApproval, "manifest"> = {
  approvedAt,
  approvedBy: "Reviewer",
};
type RetirementRegistryPair = Readonly<{
  current: string;
  id: string;
  retired: string;
}>;
type MemoryAdapter = MutationWorkflowAdapter &
  Readonly<{ contents: Map<string, string> }>;
type RetirementInputOverrides = Omit<
  Partial<MutationWorkflowCoreInput>,
  "approval"
> &
  Readonly<{ approval?: Partial<WorkflowApproval> }>;

const withMemoryLock: MutationWorkflowAdapter["withMutationLock"] = async (
  action,
) => {
  const value = await action();
  return value;
};

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
  beforeReplace?: (
    path: string,
    expected: string | undefined,
    replacement: string | undefined,
  ) => void,
): MemoryAdapter => {
  const contents = new Map(Object.entries(initial));
  const replaceMatching: MutationWorkflowAdapter["replaceMatching"] = (
    path,
    expected,
    replacement,
  ) => {
    beforeReplace?.(path, expected, replacement);
    if (contents.get(path) !== expected) {
      return false;
    }
    if (replacement === undefined) {
      contents.delete(path);
    } else {
      contents.set(path, replacement);
    }
    return true;
  };
  return {
    applyMatchingBatch: async (snapshots, onAttempt) => {
      for (const snapshot of snapshots) {
        onAttempt(snapshot);
        if (
          !(await replaceMatching(
            snapshot.path,
            snapshot.before,
            snapshot.contents,
          ))
        ) {
          return false;
        }
      }
      return true;
    },
    contents,
    withMutationLock: withMemoryLock,
    read: (path) => contents.get(path),
    replaceMatching,
    validatePreparedRegistry: (value) =>
      parseInvariantRegistry(JSON.parse(value)),
    validatePreparedSurfaces: () => Promise.resolve(),
  };
};

const retirementInput = (
  current: string,
  retired: string,
  overrides: RetirementInputOverrides = {},
): MutationWorkflowCoreInput => {
  const [currentRecord] = parseInvariantRegistry(
    JSON.parse(current),
  ).invariants;
  const [retiredRecord] = parseInvariantRegistry(
    JSON.parse(retired),
  ).invariants;
  if (currentRecord === undefined || retiredRecord === undefined) {
    throw new Error("retirement-registry-target-missing");
  }
  const approval: WorkflowApproval = {
    ...reviewerApproval,
    ...overrides.approval,
    manifest: parseMutationManifest({
      files: [
        {
          path: "surface.md",
          preimage: "old surface",
          replacement: "new surface",
        },
        { path: registryPath, preimage: current, replacement: retired },
      ],
      registryDelta: {
        after: retiredRecord,
        before: currentRecord,
        targetInvariantId: "prevent-secret-leaks",
      },
    }),
  };
  return {
    preparedFiles: [
      { contents: "new surface", path: "surface.md" },
      { contents: retired, path: registryPath },
    ],
    registryPath,
    targetInvariantId: "prevent-secret-leaks",
    ...overrides,
    approval,
  };
};

export {
  approvedAt,
  memoryAdapter,
  registryPath,
  retirementInput,
  retirementRegistryPair,
  reviewerApproval,
  type MemoryAdapter,
};
