import type {
  HarnessMutationRequest,
  MutationTransition,
  WorkflowApproval,
} from "./harness-reflection-mutation-workflow-types.ts";
import type {
  InvariantRecord,
  InvariantRegistry,
} from "./invariant-registry-contract.ts";
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

const transitionRecords = (
  input: HarnessMutationRequest,
  current: InvariantRegistry,
  proposed: InvariantRegistry,
): Readonly<{
  currentTarget: InvariantRecord | undefined;
  proposedTarget: InvariantRecord | undefined;
}> => ({
  currentTarget: current.invariants.find(
    ({ id }) => id === input.targetInvariantId,
  ),
  proposedTarget: proposed.invariants.find(
    ({ id }) => id === input.targetInvariantId,
  ),
});

const newTransition = (target: InvariantRecord): MutationTransition => {
  if (target.lifecycle === "retired") {
    throw new Error("lifecycle-transition-invalid");
  }
  return {
    kind: target.lifecycle === "active" ? "promotion" : "record-update",
    target,
  };
};

const deriveTransition = (
  input: HarnessMutationRequest,
  current: InvariantRegistry,
  proposed: InvariantRegistry,
): MutationTransition => {
  const { currentTarget, proposedTarget } = transitionRecords(
    input,
    current,
    proposed,
  );
  if (proposedTarget === undefined || currentTarget?.lifecycle === "retired") {
    throw new Error("lifecycle-transition-invalid");
  }
  if (currentTarget === undefined) {
    return newTransition(proposedTarget);
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
  return {
    kind:
      currentTarget.lifecycle === "candidate" &&
      proposedTarget.lifecycle === "active"
        ? "promotion"
        : "record-update",
    target: proposedTarget,
  };
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
  input: HarnessMutationRequest,
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
