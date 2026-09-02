import { sourceSchema } from "./invariant-registry-source.ts";
import { z } from "zod";

const semanticStringSchema = z
  .string()
  .regex(/\S/u, "Must contain a non-whitespace character.");
const measurementSchema = z
  .object({
    outcome: z.enum(["passed", "failed"]),
    ranAt: z.iso.datetime(),
    environment: semanticStringSchema,
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
      mechanism: semanticStringSchema,
      lastVerifiedEnvironment: semanticStringSchema.optional(),
    })
    .strict(),
  z
    .object({ state: z.literal("unsupported"), reason: semanticStringSchema })
    .strict(),
]);
const scopeExceptionSchema = z
  .object({
    paths: z.array(semanticStringSchema).min(1),
    reason: semanticStringSchema,
  })
  .strict();
const scopeSchema = z
  .object({
    kind: z.enum(["cross-project", "project-local"]),
    exceptions: z.array(scopeExceptionSchema),
  })
  .strict();
const approvalSchema = z
  .object({ approvedBy: semanticStringSchema, approvedAt: z.iso.datetime() })
  .strict();
const oracleSchema = z
  .object({
    name: semanticStringSchema,
    failurePath: semanticStringSchema,
    testPath: semanticStringSchema,
  })
  .strict();
const ablationConditionSchema = z
  .object({
    scenarios: z.array(semanticStringSchema).min(1),
    environments: z.array(semanticStringSchema).min(1),
    replicates: z.number().int().positive(),
    outcomes: z.array(semanticStringSchema).min(1),
  })
  .strict();
const activationMeasurementSchema = z
  .object({
    activated: z.number().int().nonnegative(),
    total: z.number().int().positive(),
  })
  .strict()
  .refine(({ activated, total }) => activated <= total);
const marginalAblationSchema = z
  .object({
    protocol: z.literal("controlled-marginal-ablation"),
    candidateTextExact: semanticStringSchema,
    with: ablationConditionSchema,
    without: ablationConditionSchema,
    observableDelta: semanticStringSchema,
    conditionalSkillActivation: z
      .object({
        with: activationMeasurementSchema,
        without: activationMeasurementSchema,
      })
      .strict()
      .optional(),
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
    reason: semanticStringSchema,
    replacedBy: semanticStringSchema.optional(),
  })
  .strict();
const invariantSchema = z
  .object({
    id: semanticStringSchema,
    statement: semanticStringSchema,
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
    marginalAblation: marginalAblationSchema.optional(),
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
