import type {
  InvariantRecord,
  InvariantRegistry,
} from "./invariant-registry-contract.ts";
import type {
  MutationTransition,
  MutationWorkflowCoreInput,
  WorkflowApproval,
} from "./harness-reflection-mutation-workflow-types.ts";
import { isDeepStrictEqual } from "node:util";
import { validateApprovedRegistryDelta } from "./harness-reflection-mutation-authorization.ts";

const withoutRetirementChanges = (
  record: Readonly<InvariantRecord>,
): Readonly<Record<string, unknown>> =>
  Object.fromEntries(
    Object.entries(record).filter(
      (entry: readonly [string, unknown]) =>
        !["approval", "lifecycle", "retirement"].includes(entry[0]),
    ),
  );

const preservesRetirementHistory = (
  current: InvariantRecord,
  proposed: InvariantRecord,
): boolean =>
  isDeepStrictEqual(
    withoutRetirementChanges(current),
    withoutRetirementChanges(proposed),
  );

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

const deriveTransition = (
  input: MutationWorkflowCoreInput,
  current: InvariantRegistry,
  proposed: InvariantRegistry,
): MutationTransition => {
  const currentTarget = current.invariants.find(
    ({ id }) => id === input.targetInvariantId,
  );
  const proposedTarget = proposed.invariants.find(
    ({ id }) => id === input.targetInvariantId,
  );
  if (proposedTarget === undefined || currentTarget?.lifecycle === "retired") {
    throw new Error("lifecycle-transition-invalid");
  }
  if (currentTarget === undefined) {
    if (proposedTarget.lifecycle === "retired") {
      throw new Error("lifecycle-transition-invalid");
    }
    return { kind: "approved-mutation", target: proposedTarget };
  }
  if (
    currentTarget.lifecycle === "active" &&
    proposedTarget.lifecycle === "retired"
  ) {
    return { kind: "retirement", target: proposedTarget };
  }
  if (
    proposedTarget.lifecycle === "retired" ||
    (currentTarget.lifecycle === "active" &&
      proposedTarget.lifecycle === "candidate")
  ) {
    throw new Error("lifecycle-transition-invalid");
  }
  if (
    currentTarget.lifecycle !== proposedTarget.lifecycle &&
    (currentTarget.lifecycle !== "candidate" ||
      proposedTarget.lifecycle !== "active")
  ) {
    throw new Error("lifecycle-transition-invalid");
  }
  return { kind: "approved-mutation", target: proposedTarget };
};

const validateHistoricalFields = (
  transition: MutationTransition,
  current: InvariantRegistry,
  targetInvariantId: string,
): void => {
  if (transition.kind !== "retirement") {
    return;
  }
  const currentTarget = current.invariants.find(
    ({ id }) => id === targetInvariantId,
  );
  if (
    currentTarget === undefined ||
    !preservesRetirementHistory(currentTarget, transition.target)
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
): MutationTransition => {
  validateApprovedRegistryDelta(input, registries, approval);
  if (
    !approvalMatches(registries.proposed, input.targetInvariantId, approval)
  ) {
    throw new Error("prepared-registry-approval-mismatch");
  }
  const transition = deriveTransition(
    input,
    registries.current,
    registries.proposed,
  );
  validateHistoricalFields(
    transition,
    registries.current,
    input.targetInvariantId,
  );
  return transition;
};

export { validateTransition };
