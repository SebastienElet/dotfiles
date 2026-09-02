import type { InvariantRegistry } from "./invariant-registry-contract.ts";

type MaybePromise<Value> = Value | Promise<Value>;
type WorkflowApproval = Readonly<{
  approvedAt: string;
  approvedBy: string;
  source: "human-context" | "agent-self-asserted";
}>;
type PreparedFile = Readonly<{ contents: string; path: string }>;
type PreparedSnapshot = PreparedFile & Readonly<{ before: string | undefined }>;
type HarnessMutationRequest = Readonly<{
  approval: WorkflowApproval | undefined;
  kind: "approved-mutation" | "retirement";
  preparedFiles: readonly PreparedFile[];
  targetInvariantId: string;
}>;
type MutationWorkflowStatus =
  | "succeeded"
  | "rejected"
  | "compensated"
  | "compensation-incomplete";
type MutationWorkflowResult = Readonly<{
  events: readonly string[];
  reason?: string;
  status: MutationWorkflowStatus;
  unresolvedPaths: readonly string[];
}>;
type MutationWorkflowAdapter = Readonly<{
  compareAndSwap: (
    path: string,
    expected: string | undefined,
    replacement: string | undefined,
  ) => MaybePromise<boolean>;
  read: (path: string) => MaybePromise<string | undefined>;
  validatePreparedRegistry: (contents: string) => InvariantRegistry;
  validatePreparedSurfaces: (
    files: readonly PreparedFile[],
    kind: HarnessMutationRequest["kind"],
  ) => MaybePromise<void>;
}>;
type MutationWorkflowCoreInput = HarnessMutationRequest &
  Readonly<{ registryPath: string }>;

export type {
  HarnessMutationRequest,
  MutationWorkflowAdapter,
  MutationWorkflowCoreInput,
  MutationWorkflowResult,
  PreparedFile,
  PreparedSnapshot,
  WorkflowApproval,
};
