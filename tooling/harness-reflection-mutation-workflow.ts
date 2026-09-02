import type {
  MutationWorkflowInput,
  MutationWorkflowResult,
  PreparedFile,
} from "./harness-reflection-mutation-workflow-types.ts";
import { parseInvariantRegistry } from "./invariant-registry-contract.ts";

type PreparedSnapshot = PreparedFile & Readonly<{ before: string | undefined }>;

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
  input: MutationWorkflowInput,
): Promise<readonly PreparedSnapshot[]> => {
  const paths = input.preparedFiles.map(({ path }) => path);
  if (new Set(paths).size !== paths.length) {
    throw new Error("duplicate-prepared-path");
  }
  const snapshots: PreparedSnapshot[] = [];
  for (const file of input.preparedFiles) {
    const before = await input.files.read(file.path);
    snapshots.push({ ...file, before });
  }
  return snapshots;
};

const validRetirementTransition = (
  before: string,
  after: string,
  retirementInvariantId: string | undefined,
): boolean => {
  const current = parseInvariantRegistry(JSON.parse(before));
  const proposed = parseInvariantRegistry(JSON.parse(after));
  const preservesSources = current.invariants.every((record) => {
    const next = proposed.invariants.find(({ id }) => id === record.id);
    return (
      next !== undefined &&
      JSON.stringify(next.sources) === JSON.stringify(record.sources)
    );
  });
  const currentTarget = current.invariants.find(
    ({ id }) => id === retirementInvariantId,
  );
  const proposedTarget = proposed.invariants.find(
    ({ id }) => id === retirementInvariantId,
  );
  return (
    preservesSources &&
    currentTarget !== undefined &&
    currentTarget.lifecycle !== "retired" &&
    proposedTarget?.lifecycle === "retired"
  );
};

const validateRetirementHistory = (
  input: MutationWorkflowInput,
  snapshots: readonly PreparedSnapshot[],
): void => {
  if (input.kind !== "retirement") {
    return;
  }
  const registry = snapshots.find(({ path }) => path === input.registryPath);
  if (
    registry?.before === undefined ||
    !validRetirementTransition(
      registry.before,
      registry.contents,
      input.retirementInvariantId,
    )
  ) {
    throw new Error("retirement-source-history-changed");
  }
};

const preimagesStillMatch = async (
  input: MutationWorkflowInput,
  snapshots: readonly PreparedSnapshot[],
): Promise<boolean> => {
  const current: (string | undefined)[] = [];
  for (const snapshot of snapshots) {
    current.push(await input.files.read(snapshot.path));
  }
  return snapshots.every(({ before }, index) => before === current[index]);
};

const compensate = async (
  input: MutationWorkflowInput,
  applied: readonly PreparedSnapshot[],
  events: readonly string[],
): Promise<MutationWorkflowResult> => {
  const compensationEvents = [...events, "compensation-started"];
  const unresolvedPaths: string[] = [];
  for (const snapshot of applied.toReversed()) {
    let restored = false;
    try {
      restored = await input.files.compareAndSwap(
        snapshot.path,
        snapshot.contents,
        snapshot.before,
      );
    } catch {
      restored = false;
    }
    if (!restored) {
      unresolvedPaths.push(snapshot.path);
    }
  }
  compensationEvents.push(
    unresolvedPaths.length === 0
      ? "compensation-completed"
      : "compensation-incomplete",
  );
  return {
    events: compensationEvents,
    reason: "apply-or-validation-failed",
    status:
      unresolvedPaths.length === 0 ? "compensated" : "compensation-incomplete",
    unresolvedPaths,
  };
};

const applySnapshots = async (
  input: MutationWorkflowInput,
  snapshots: readonly PreparedSnapshot[],
  events: readonly string[],
): Promise<MutationWorkflowResult> => {
  const applied: PreparedSnapshot[] = [];
  const applyingEvents = [...events, "apply-started"];
  try {
    for (const snapshot of snapshots) {
      const appliedNow = await input.files.compareAndSwap(
        snapshot.path,
        snapshot.before,
        snapshot.contents,
      );
      if (!appliedNow) {
        return await compensate(input, applied, [
          ...applyingEvents,
          "apply-conflict",
        ]);
      }
      applied.push(snapshot);
    }
    await input.validateAppliedChange();
    return {
      events: [
        ...applyingEvents,
        "applied-change-validated",
        "success-rendered",
      ],
      status: "succeeded",
      unresolvedPaths: [],
    };
  } catch {
    return compensate(input, applied, [...applyingEvents, "apply-error"]);
  }
};

type PreparedWorkflow = Readonly<{
  events: readonly string[];
  snapshots: readonly PreparedSnapshot[];
}>;

const prepareWorkflow = async (
  input: MutationWorkflowInput,
  events: readonly string[],
): Promise<MutationWorkflowResult | PreparedWorkflow> => {
  try {
    const snapshots = await capturePreimages(input);
    const capturedEvents = [...events, "preimages-captured"];
    validateRetirementHistory(input, snapshots);
    const historyEvents =
      input.kind === "retirement"
        ? [...capturedEvents, "retirement-history-validated"]
        : capturedEvents;
    const registry = snapshots.find(({ path }) => path === input.registryPath);
    if (registry === undefined) {
      return rejected(historyEvents, "prepared-registry-required");
    }
    await input.validatePreparedSurfaces(
      snapshots.filter(({ path }) => path !== input.registryPath),
    );
    const surfaceEvents = [...historyEvents, "prepared-surfaces-validated"];
    await input.validatePreparedRegistry(registry.contents);
    return {
      events: [...surfaceEvents, "prepared-registry-validated"],
      snapshots,
    };
  } catch (error) {
    return rejected(
      events,
      error instanceof Error ? error.message : "prepared-change-invalid",
    );
  }
};

const executeHarnessMutationWorkflow = async (
  input: MutationWorkflowInput,
): Promise<MutationWorkflowResult> => {
  if (
    input.approval?.source !== "human-context" ||
    input.approval.approvedBy.trim() === "" ||
    !Number.isFinite(Date.parse(input.approval.approvedAt))
  ) {
    return rejected([], "human-context-approval-required");
  }
  const prepared = await prepareWorkflow(input, ["approval-accepted"]);
  if ("status" in prepared) {
    return prepared;
  }
  if (!(await preimagesStillMatch(input, prepared.snapshots))) {
    return rejected(prepared.events, "preimage-conflict-before-apply");
  }
  return applySnapshots(input, prepared.snapshots, [
    ...prepared.events,
    "preimages-confirmed",
  ]);
};

export { executeHarnessMutationWorkflow };
export type {
  MutationFileAccess,
  MutationWorkflowInput,
  MutationWorkflowResult,
} from "./harness-reflection-mutation-workflow-types.ts";
