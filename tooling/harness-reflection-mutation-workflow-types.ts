import type { InvariantRecord } from "./invariant-registry-contract.ts";
import { z } from "zod";

const preparedFileSchema = z
  .object({
    contents: z.string(),
    path: z.string(),
    preimage: z.string().nullable(),
  })
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

type DeepReadonly<Value> = Value extends readonly (infer Item)[]
  ? readonly DeepReadonly<Item>[]
  : Value extends object
    ? { readonly [Key in keyof Value]: DeepReadonly<Value[Key]> }
    : Value;
type WorkflowApproval = DeepReadonly<z.output<typeof workflowApprovalSchema>>;
type MutationManifest = WorkflowApproval["manifest"];
type PreparedFile = DeepReadonly<z.output<typeof preparedFileSchema>>;
type MutationKind = "link" | "promotion" | "retirement";
type MutationTransition = Readonly<{
  kind: MutationKind;
  target: InvariantRecord;
}>;
type HarnessMutationRequest = DeepReadonly<
  z.output<typeof harnessMutationRequestSchema>
>;

const parseMutationManifest = (input: unknown): MutationManifest =>
  approvedMutationManifestSchema.parse(input);

const parseHarnessMutationRequest = (input: unknown): HarnessMutationRequest =>
  harnessMutationRequestSchema.parse(input);

export type {
  HarnessMutationRequest,
  MutationKind,
  MutationManifest,
  MutationTransition,
  PreparedFile,
  WorkflowApproval,
};
export { parseHarnessMutationRequest, parseMutationManifest };
