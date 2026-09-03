import {
  claudeConsumerMechanisms,
  codexConsumerMechanisms,
  cursorConsumerMechanisms,
} from "./invariant-registry-consumers.ts";
import { sourceSchema } from "./invariant-registry-source.ts";
import { z } from "zod";

const semanticStringSchema = z
  .string()
  .regex(/\S/u, "Must contain a non-whitespace character.");
const invariantSurfaces = [
  "always-loaded-instruction",
  "conditional-skill",
  "project-local-contract",
  "hook",
  "permission",
  "lint",
  "type",
  "architectural-test",
] as const;
const measurementSchema = z
  .object({
    outcome: z.enum(["passed", "failed"]),
    ranAt: z.iso.datetime(),
    environment: semanticStringSchema,
    oracle: z
      .object({
        name: semanticStringSchema,
        testPath: semanticStringSchema,
        invocation: z.array(semanticStringSchema).min(1),
      })
      .strict()
      .optional(),
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
const unsupportedConsumerSchema = z
  .object({ state: z.literal("unsupported"), reason: semanticStringSchema })
  .strict();
const claudeConsumerSchema = z.discriminatedUnion("state", [
  z
    .object({
      state: z.literal("supported"),
      mechanism: z.enum(claudeConsumerMechanisms),
      lastVerifiedEnvironment: semanticStringSchema.optional(),
    })
    .strict(),
  unsupportedConsumerSchema,
]);
const codexConsumerSchema = z.discriminatedUnion("state", [
  z
    .object({
      state: z.literal("supported"),
      mechanism: z.enum(codexConsumerMechanisms),
      lastVerifiedEnvironment: semanticStringSchema.optional(),
    })
    .strict(),
  unsupportedConsumerSchema,
]);
const cursorConsumerSchema = z.discriminatedUnion("state", [
  z
    .object({
      state: z.literal("supported"),
      mechanism: z.enum(cursorConsumerMechanisms),
      lastVerifiedEnvironment: semanticStringSchema.optional(),
    })
    .strict(),
  unsupportedConsumerSchema,
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
    invocation: z.array(semanticStringSchema).min(1),
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
    claude: claudeConsumerSchema,
    codex: codexConsumerSchema,
    cursor: cursorConsumerSchema,
  })
  .strict();
const retirementSchema = z
  .object({
    retiredAt: z.iso.datetime(),
    reason: semanticStringSchema,
    replacedBy: semanticStringSchema.optional(),
  })
  .strict();
const invariantShape = {
  id: semanticStringSchema,
  statement: semanticStringSchema,
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
  surface: z.enum(invariantSurfaces),
  approval: approvalSchema.optional(),
  oracle: oracleSchema.optional(),
  marginalAblation: marginalAblationSchema.optional(),
  consumers: consumersSchema,
  verification: verificationSchema,
};
const invariantSchema = z.discriminatedUnion("lifecycle", [
  z.object({ ...invariantShape, lifecycle: z.literal("candidate") }).strict(),
  z.object({ ...invariantShape, lifecycle: z.literal("active") }).strict(),
  z
    .object({
      ...invariantShape,
      lifecycle: z.literal("retired"),
      retirement: retirementSchema,
    })
    .strict(),
]);
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
type OracleInspection = Readonly<{
  discovered: boolean;
  kind: "missing" | "non-regular" | "regular-file";
  tracked: boolean;
}>;
type ValidationOptions = Readonly<{
  repositoryRoot: string;
  inspectOracle: (
    path: string,
    invocation: readonly string[],
  ) => OracleInspection;
}>;

const parseInvariantRegistry = (input: unknown): InvariantRegistry =>
  registrySchema.parse(input);

export {
  invariantSurfaces,
  parseInvariantRegistry,
  type InvariantRecord,
  type InvariantRegistry,
  type OracleInspection,
  type RegistryDiagnostic,
  type ValidationOptions,
};
