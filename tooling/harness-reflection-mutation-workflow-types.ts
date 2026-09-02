type MaybePromise<Value> = Value | Promise<Value>;
type MutationFileAccess = Readonly<{
  compareAndSwap: (
    path: string,
    expected: string | undefined,
    replacement: string | undefined,
  ) => boolean | Promise<boolean>;
  read: (path: string) => string | undefined | Promise<string | undefined>;
}>;
type WorkflowApproval = Readonly<{
  approvedAt: string;
  approvedBy: string;
  source: "human-context" | "agent-self-asserted";
}>;
type PreparedFile = Readonly<{ contents: string; path: string }>;
type MutationWorkflowInput = Readonly<{
  approval: WorkflowApproval | undefined;
  files: MutationFileAccess;
  kind: "approved-mutation" | "retirement";
  preparedFiles: readonly PreparedFile[];
  registryPath: string;
  retirementInvariantId?: string;
  validateAppliedChange: () => MaybePromise<void>;
  validatePreparedRegistry: (contents: string) => MaybePromise<void>;
  validatePreparedSurfaces: (
    files: readonly PreparedFile[],
  ) => MaybePromise<void>;
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

export type {
  MutationFileAccess,
  MutationWorkflowInput,
  MutationWorkflowResult,
  PreparedFile,
};
