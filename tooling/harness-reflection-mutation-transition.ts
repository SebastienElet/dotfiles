import type {
  InvariantRecord,
  InvariantRegistry,
} from "./invariant-registry-contract.ts";
import type {
  MutationWorkflowCoreInput,
  WorkflowApproval,
} from "./harness-reflection-mutation-workflow-types.ts";
import { validateApprovedRegistryDelta } from "./harness-reflection-mutation-authorization.ts";

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

const validateRetirement = (
  input: MutationWorkflowCoreInput,
  current: InvariantRegistry,
  proposed: InvariantRegistry,
): void => {
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

const validateTransition = (
  input: MutationWorkflowCoreInput,
  registries: Readonly<{
    current: InvariantRegistry;
    proposed: InvariantRegistry;
  }>,
  approval: WorkflowApproval,
): void => {
  validateApprovedRegistryDelta(input, registries, approval);
  if (
    !approvalMatches(registries.proposed, input.targetInvariantId, approval)
  ) {
    throw new Error("prepared-registry-approval-mismatch");
  }
  validateRetirement(input, registries.current, registries.proposed);
};

export { validateTransition };
