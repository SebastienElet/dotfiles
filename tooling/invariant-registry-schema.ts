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
const invariantSurfaceSchema = z.enum(invariantSurfaces);
const nonConditionalSurfaceSchema = invariantSurfaceSchema.exclude([
  "conditional-skill",
]);
const targetSkillPathSchema = z
  .string()
  .regex(/^harness\/skills\/[a-z0-9]+(?:-[a-z0-9]+)*\/SKILL\.md$/u);
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
  approval: approvalSchema.optional(),
  oracle: oracleSchema.optional(),
  marginalAblation: marginalAblationSchema.optional(),
  consumers: consumersSchema,
  verification: verificationSchema,
};
const lifecycleSchemas = <Shape extends z.ZodRawShape>(shape: Shape) =>
  [
    z.object({ ...shape, lifecycle: z.literal("candidate") }).strict(),
    z.object({ ...shape, lifecycle: z.literal("active") }).strict(),
    z
      .object({
        ...shape,
        lifecycle: z.literal("retired"),
        retirement: retirementSchema,
      })
      .strict(),
  ] as const;
const invariantSchema = z.union([
  ...lifecycleSchemas({
    ...invariantShape,
    surface: z.literal("conditional-skill"),
    targetSkillPath: targetSkillPathSchema,
  }),
  ...lifecycleSchemas({
    ...invariantShape,
    surface: nonConditionalSurfaceSchema,
  }),
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
const parseInvariantRegistry = (input: unknown): InvariantRegistry =>
  registrySchema.parse(input);

export {
  invariantSurfaces,
  parseInvariantRegistry,
  type InvariantRecord,
  type InvariantRegistry,
  type RegistryDiagnostic,
};
export type {
  OracleInspection,
  SkillTargetInspection,
  ValidationOptions,
} from "./invariant-registry-validation-options.ts";
