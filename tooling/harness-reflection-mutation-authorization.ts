import type {
  HarnessMutationRequest,
  WorkflowApproval,
} from "./harness-reflection-mutation-workflow-types.ts";
import type {
  InvariantRecord,
  InvariantRegistry,
} from "./invariant-registry-contract.ts";
import { isDeepStrictEqual } from "node:util";

type RegistryPair = Readonly<{
  current: InvariantRegistry;
  proposed: InvariantRegistry;
}>;

const sameValue = (left: unknown, right: unknown): boolean =>
  isDeepStrictEqual(left, right);

const recordsExceptTarget = (
  registry: InvariantRegistry,
  targetInvariantId: string,
): readonly InvariantRecord[] =>
  registry.invariants.filter(({ id }) => id !== targetInvariantId);

const validateApprovedFileRequest = (
  input: HarnessMutationRequest,
  approval: WorkflowApproval,
): void => {
  const { manifest } = approval;
  const matches =
    manifest.registryDelta.targetInvariantId === input.targetInvariantId &&
    manifest.files.length === input.preparedFiles.length &&
    manifest.files.every((file, index) => {
      const prepared = input.preparedFiles[index];
      return (
        prepared?.path === file.path &&
        prepared.contents === file.replacement &&
        prepared.preimage === file.preimage
      );
    });
  if (!matches) {
    throw new Error("approved-manifest-request-mismatch");
  }
};

const validateApprovedRegistryDelta = (
  input: HarnessMutationRequest,
  registries: RegistryPair,
  approval: WorkflowApproval,
): void => {
  const { targetInvariantId } = input;
  const delta = approval.manifest.registryDelta;
  const currentTargets = registries.current.invariants.filter(
    ({ id }) => id === targetInvariantId,
  );
  const proposedTargets = registries.proposed.invariants.filter(
    ({ id }) => id === targetInvariantId,
  );
  if (currentTargets.length !== 1 || proposedTargets.length !== 1) {
    throw new Error("registry-target-count-invalid");
  }
  const [currentTarget] = currentTargets;
  const [proposedTarget] = proposedTargets;
  const matches =
    registries.current.version === registries.proposed.version &&
    sameValue(currentTarget, delta.before) &&
    sameValue(proposedTarget, delta.after) &&
    !sameValue(delta.before, delta.after) &&
    sameValue(
      recordsExceptTarget(registries.current, targetInvariantId),
      recordsExceptTarget(registries.proposed, targetInvariantId),
    );
  if (!matches) {
    throw new Error("approved-registry-delta-mismatch");
  }
};

export { validateApprovedFileRequest, validateApprovedRegistryDelta };
