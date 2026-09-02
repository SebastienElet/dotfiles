import type {
  InvariantRecord,
  InvariantRegistry,
} from "./invariant-registry-contract.ts";
import type {
  MutationWorkflowCoreInput,
  PreparedSnapshot,
  WorkflowApproval,
} from "./harness-reflection-mutation-workflow-types.ts";
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
  input: MutationWorkflowCoreInput,
  approval: WorkflowApproval,
): void => {
  const { manifest } = approval;
  const matches =
    manifest.kind === input.kind &&
    manifest.registryDelta.targetInvariantId === input.targetInvariantId &&
    manifest.files.length === input.preparedFiles.length &&
    manifest.files.every((file, index) => {
      const prepared = input.preparedFiles[index];
      return (
        prepared?.path === file.path && prepared.contents === file.replacement
      );
    });
  if (!matches) {
    throw new Error("approved-manifest-request-mismatch");
  }
};

const validateApprovedPreimages = (
  snapshots: readonly PreparedSnapshot[],
  approval: WorkflowApproval,
): void => {
  const matches = snapshots.every((snapshot, index) => {
    const approved = approval.manifest.files[index];
    return (
      approved?.path === snapshot.path &&
      (approved.preimage ?? undefined) === snapshot.before
    );
  });
  if (!matches) {
    throw new Error("approved-manifest-preimage-mismatch");
  }
};

const validateApprovedRegistryDelta = (
  input: MutationWorkflowCoreInput,
  registries: RegistryPair,
  approval: WorkflowApproval,
): void => {
  const { targetInvariantId } = input;
  const delta = approval.manifest.registryDelta;
  const currentTarget = registries.current.invariants.find(
    ({ id }) => id === targetInvariantId,
  );
  const proposedTarget = registries.proposed.invariants.find(
    ({ id }) => id === targetInvariantId,
  );
  const matches =
    registries.current.version === registries.proposed.version &&
    sameValue(currentTarget ?? null, delta.before) &&
    sameValue(proposedTarget ?? null, delta.after) &&
    !sameValue(delta.before, delta.after) &&
    sameValue(
      recordsExceptTarget(registries.current, targetInvariantId),
      recordsExceptTarget(registries.proposed, targetInvariantId),
    );
  if (!matches) {
    throw new Error("approved-registry-delta-mismatch");
  }
};

export {
  validateApprovedFileRequest,
  validateApprovedPreimages,
  validateApprovedRegistryDelta,
};
