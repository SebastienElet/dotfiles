import type {
  InvariantRecord,
  InvariantRegistry,
} from "./invariant-registry-contract.ts";
import { z } from "zod";

type MaybePromise<Value> = Value | Promise<Value>;
const preparedFileSchema = z
  .object({ contents: z.string(), path: z.string() })
  .strict();
const approvedFileMutationSchema = z
  .object({
    path: z.string(),
    preimage: z.string().nullable(),
    replacement: z.string(),
  })
  .strict();
const registryRecordValueSchema = z.record(z.string(), z.json());
const approvedMutationManifestSchema = z
  .object({
    files: z.array(approvedFileMutationSchema).min(1),
    registryDelta: z
      .object({
        after: registryRecordValueSchema.nullable(),
        before: registryRecordValueSchema.nullable(),
        targetInvariantId: z.string(),
      })
      .strict(),
  })
  .strict();
const workflowApprovalSchema = z
  .object({
    approvedAt: z.iso.datetime(),
    approvedBy: z.string().regex(/\S/u),
    manifest: approvedMutationManifestSchema,
  })
  .strict();
const harnessMutationRequestShape = {
  approval: workflowApprovalSchema.optional(),
  preparedFiles: z.array(preparedFileSchema),
  targetInvariantId: z.string(),
};
const harnessMutationRequestSchema = z
  .object(harnessMutationRequestShape)
  .strict();
const mutationWorkflowCoreInputSchema = z
  .object({ ...harnessMutationRequestShape, registryPath: z.string() })
  .strict();

type DeepReadonly<Value> = Value extends readonly (infer Item)[]
  ? readonly DeepReadonly<Item>[]
  : Value extends object
    ? { readonly [Key in keyof Value]: DeepReadonly<Value[Key]> }
    : Value;
type WorkflowApproval = DeepReadonly<z.output<typeof workflowApprovalSchema>>;
type MutationManifest = WorkflowApproval["manifest"];
type PreparedFile = DeepReadonly<z.output<typeof preparedFileSchema>>;
type PreparedSnapshot = PreparedFile & Readonly<{ before: string | undefined }>;
type MutationKind = "approved-mutation" | "retirement";
type MutationTransition = Readonly<{
  kind: MutationKind;
  target: InvariantRecord;
}>;
type HarnessMutationRequest = DeepReadonly<
  z.output<typeof harnessMutationRequestSchema>
>;
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
  applyMatchingBatch: (
    snapshots: readonly PreparedSnapshot[],
    onAttempt: (snapshot: PreparedSnapshot) => void,
  ) => MaybePromise<boolean>;
  replaceMatching: (
    path: string,
    expected: string | undefined,
    replacement: string | undefined,
  ) => MaybePromise<boolean>;
  read: (path: string) => MaybePromise<string | undefined>;
  withMutationLock: <Value>(
    action: () => MaybePromise<Value>,
  ) => Promise<Value>;
  validatePreparedRegistry: (contents: string) => InvariantRegistry;
  validatePreparedSurfaces: (
    files: readonly PreparedFile[],
    transition: MutationTransition,
  ) => MaybePromise<void>;
}>;
type MutationWorkflowCoreInput = HarnessMutationRequest &
  Readonly<{ registryPath: string }>;

const parseMutationWorkflowCoreInput = (
  input: unknown,
): MutationWorkflowCoreInput => mutationWorkflowCoreInputSchema.parse(input);

const parseMutationManifest = (input: unknown): MutationManifest =>
  approvedMutationManifestSchema.parse(input);

export type {
  HarnessMutationRequest,
  MutationKind,
  MutationManifest,
  MutationTransition,
  MutationWorkflowAdapter,
  MutationWorkflowCoreInput,
  MutationWorkflowResult,
  PreparedFile,
  PreparedSnapshot,
  WorkflowApproval,
};
export { parseMutationManifest, parseMutationWorkflowCoreInput };
