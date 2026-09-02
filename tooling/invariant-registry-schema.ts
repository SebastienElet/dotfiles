import { z } from "zod";

const measurementSchema = z
  .object({
    outcome: z.enum(["passed", "failed"]),
    ranAt: z.iso.datetime(),
    environment: z.string().min(1),
  })
  .strict();
const verifiedMeasurementSchema = measurementSchema.extend({
  outcome: z.literal("passed"),
});
const verificationSchema = z.discriminatedUnion("state", [
  z.object({ state: z.literal("unverified") }).strict(),
  z
    .object({ state: z.literal("measured"), lastRun: measurementSchema })
    .strict(),
  z
    .object({
      state: z.literal("verified"),
      lastRun: verifiedMeasurementSchema,
    })
    .strict(),
]);
const consumerSchema = z.discriminatedUnion("state", [
  z
    .object({
      state: z.literal("supported"),
      mechanism: z.string().min(1),
      lastVerifiedEnvironment: z.string().min(1).optional(),
    })
    .strict(),
  z
    .object({ state: z.literal("unsupported"), reason: z.string().min(1) })
    .strict(),
]);
const sourceSchema = z
  .object({ pullRequestUrl: z.url(), evidenceUrl: z.url() })
  .strict();
const scopeExceptionSchema = z
  .object({
    paths: z.array(z.string().min(1)).min(1),
    reason: z.string().min(1),
  })
  .strict();
const scopeSchema = z
  .object({
    kind: z.enum(["cross-project", "project-local"]),
    exceptions: z.array(scopeExceptionSchema),
  })
  .strict();
const approvalSchema = z
  .object({ approvedBy: z.string().min(1), approvedAt: z.iso.datetime() })
  .strict();
const oracleSchema = z
  .object({
    name: z.string().min(1),
    failurePath: z.string().min(1),
    testPath: z.string().min(1),
  })
  .strict();
const consumersSchema = z
  .object({
    claude: consumerSchema,
    codex: consumerSchema,
    cursor: consumerSchema,
  })
  .strict();
const retirementSchema = z
  .object({
    retiredAt: z.iso.datetime(),
    reason: z.string().min(1),
    replacedBy: z.string().min(1).optional(),
  })
  .strict();
const invariantSchema = z
  .object({
    id: z.string().min(1),
    statement: z.string().min(1),
    lifecycle: z.enum(["candidate", "active", "retired"]),
    controlKind: z.enum(["probabilistic", "enforceable"]),
    causeClass: z.enum([
      "not-applied",
      "not-loaded",
      "unknown",
      "blind-spot",
      "judgment",
    ]),
    severity: z.enum(["low", "medium", "high", "critical"]),
    sources: z.array(sourceSchema).min(1),
    scope: scopeSchema,
    surface: z.enum([
      "always-loaded-instruction",
      "conditional-skill",
      "project-local-contract",
      "hook",
      "permission",
      "lint",
      "type",
      "architectural-test",
    ]),
    approval: approvalSchema.optional(),
    oracle: oracleSchema.optional(),
    consumers: consumersSchema,
    verification: verificationSchema,
    retirement: retirementSchema.optional(),
  })
  .strict();
const registrySchema = z
  .object({ version: z.literal(1), invariants: z.array(invariantSchema) })
  .strict();

type DeepReadonly<Value> = Value extends readonly (infer Item)[]
  ? readonly DeepReadonly<Item>[]
  : Value extends object
    ? { readonly [Key in keyof Value]: DeepReadonly<Value[Key]> }
    : Value;
type InvariantRecord = DeepReadonly<z.output<typeof invariantSchema>>;
type InvariantRegistry = DeepReadonly<z.output<typeof registrySchema>>;
type RegistryDiagnostic = Readonly<{
  code: string;
  path: string;
  message: string;
}>;
type ValidationOptions = Readonly<{
  repositoryRoot: string;
  pathExists: (path: string) => boolean;
}>;

const parseInvariantRegistry = (input: unknown): InvariantRegistry =>
  registrySchema.parse(input);

export {
  parseInvariantRegistry,
  type InvariantRecord,
  type InvariantRegistry,
  type RegistryDiagnostic,
  type ValidationOptions,
};
