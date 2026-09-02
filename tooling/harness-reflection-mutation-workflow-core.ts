import type {
  InvariantRecord,
  InvariantRegistry,
} from "./invariant-registry-contract.ts";
import type {
  MutationWorkflowAdapter,
  MutationWorkflowCoreInput,
  MutationWorkflowResult,
  PreparedSnapshot,
  WorkflowApproval,
} from "./harness-reflection-mutation-workflow-types.ts";
import {
  compensate,
  readMatches,
} from "./harness-reflection-mutation-workflow-compensation.ts";

type PreparedWorkflow = Readonly<{
  events: readonly string[];
  snapshots: readonly PreparedSnapshot[];
}>;

const rejected = (
  events: readonly string[],
  reason: string,
): MutationWorkflowResult => ({
  events,
  reason,
  status: "rejected",
  unresolvedPaths: [],
});

const capturePreimages = async (
  input: MutationWorkflowCoreInput,
  adapter: MutationWorkflowAdapter,
): Promise<readonly PreparedSnapshot[]> => {
  const paths = input.preparedFiles.map(({ path }) => path);
  if (new Set(paths).size !== paths.length) {
    throw new Error("duplicate-prepared-path");
  }
  const snapshots: PreparedSnapshot[] = [];
  for (const file of input.preparedFiles) {
    snapshots.push({ ...file, before: await adapter.read(file.path) });
  }
  return snapshots;
};

const withoutRetirement = (
  record: Readonly<InvariantRecord>,
): Readonly<Record<string, unknown>> =>
  Object.fromEntries(
    Object.entries(record).filter(
      (entry: readonly [string, unknown]) =>
        !["lifecycle", "retirement"].includes(entry[0]),
    ),
  );

const preservesRetirementHistory = (
  current: InvariantRecord,
  proposed: InvariantRecord,
): boolean =>
  JSON.stringify(withoutRetirement(current)) ===
  JSON.stringify(withoutRetirement(proposed));

const approvalMatches = (
  registry: InvariantRegistry,
  targetInvariantId: string,
  approval: WorkflowApproval,
): boolean => {
  const persisted = registry.invariants.find(
    ({ id }) => id === targetInvariantId,
  )?.approval;
  return (
    persisted?.approvedBy === approval.approvedBy &&
    persisted.approvedAt === approval.approvedAt
  );
};

const validateTransition = (
  input: MutationWorkflowCoreInput,
  registries: Readonly<{
    current: InvariantRegistry;
    proposed: InvariantRegistry;
  }>,
  approval: WorkflowApproval,
): void => {
  const { current, proposed } = registries;
  if (!approvalMatches(proposed, input.targetInvariantId, approval)) {
    throw new Error("prepared-registry-approval-mismatch");
  }
  if (input.kind !== "retirement") {
    return;
  }
  const currentTarget = current.invariants.find(
    ({ id }) => id === input.targetInvariantId,
  );
  const proposedTarget = proposed.invariants.find(
    ({ id }) => id === input.targetInvariantId,
  );
  if (
    currentTarget === undefined ||
    currentTarget.lifecycle === "retired" ||
    proposedTarget?.lifecycle !== "retired" ||
    !preservesRetirementHistory(currentTarget, proposedTarget)
  ) {
    throw new Error("retirement-history-changed");
  }
};

const validateApplied = async (
  input: MutationWorkflowCoreInput,
  adapter: MutationWorkflowAdapter,
  snapshots: readonly PreparedSnapshot[],
): Promise<void> => {
  for (const snapshot of snapshots) {
    if (!(await readMatches(adapter, snapshot.path, snapshot.contents))) {
      throw new Error("applied-content-mismatch");
    }
  }
  const registry = snapshots.find(({ path }) => path === input.registryPath);
  if (registry === undefined) {
    throw new Error("prepared-registry-required");
  }
  adapter.validatePreparedRegistry(registry.contents);
  await adapter.validatePreparedSurfaces(
    snapshots.filter(({ path }) => path !== input.registryPath),
    input.kind,
  );
};

const applySnapshots = async (
  input: MutationWorkflowCoreInput,
  adapter: MutationWorkflowAdapter,
  prepared: PreparedWorkflow,
): Promise<MutationWorkflowResult> => {
  const { events, snapshots } = prepared;
  const attempted: PreparedSnapshot[] = [];
  const applyingEvents = [...events, "apply-started"];
  for (const snapshot of snapshots) {
    attempted.push(snapshot);
    try {
      if (
        !(await adapter.compareAndSwap(
          snapshot.path,
          snapshot.before,
          snapshot.contents,
        ))
      ) {
        return await compensate(adapter, attempted, [
          ...applyingEvents,
          "apply-conflict",
        ]);
      }
    } catch {
      return compensate(adapter, attempted, [...applyingEvents, "apply-error"]);
    }
  }
  try {
    await validateApplied(input, adapter, snapshots);
  } catch {
    return compensate(adapter, attempted, [
      ...applyingEvents,
      "applied-validation-error",
    ]);
  }
  return {
    events: [...applyingEvents, "applied-change-validated", "success-rendered"],
    status: "succeeded",
    unresolvedPaths: [],
  };
};

const prepareWorkflow = async (
  input: MutationWorkflowCoreInput,
  adapter: MutationWorkflowAdapter,
  approval: WorkflowApproval,
): Promise<MutationWorkflowResult | PreparedWorkflow> => {
  try {
    const snapshots = await capturePreimages(input, adapter);
    const registry = snapshots.find(({ path }) => path === input.registryPath);
    if (registry?.before === undefined) {
      return rejected(["approval-accepted"], "prepared-registry-required");
    }
    const surfaces = snapshots.filter(
      ({ path }) => path !== input.registryPath,
    );
    await adapter.validatePreparedSurfaces(surfaces, input.kind);
    const current = adapter.validatePreparedRegistry(registry.before);
    const proposed = adapter.validatePreparedRegistry(registry.contents);
    validateTransition(input, { current, proposed }, approval);
    return {
      events: [
        "approval-accepted",
        "preimages-captured",
        "prepared-surfaces-validated",
        "prepared-registry-validated",
      ],
      snapshots,
    };
  } catch (error) {
    return rejected(
      ["approval-accepted"],
      error instanceof Error ? error.message : "prepared-change-invalid",
    );
  }
};

const executeHarnessMutationWorkflowCore = async (
  input: MutationWorkflowCoreInput,
  adapter: MutationWorkflowAdapter,
): Promise<MutationWorkflowResult> => {
  const { approval } = input;
  if (
    approval?.source !== "human-context" ||
    approval.approvedBy.trim() === "" ||
    !Number.isFinite(Date.parse(approval.approvedAt))
  ) {
    return rejected([], "human-context-approval-required");
  }
  const prepared = await prepareWorkflow(input, adapter, approval);
  if ("status" in prepared) {
    return prepared;
  }
  for (const snapshot of prepared.snapshots) {
    if (!(await readMatches(adapter, snapshot.path, snapshot.before))) {
      return rejected(prepared.events, "preimage-conflict-before-apply");
    }
  }
  return applySnapshots(input, adapter, {
    events: [...prepared.events, "preimages-confirmed"],
    snapshots: prepared.snapshots,
  });
};

export { executeHarnessMutationWorkflowCore };
