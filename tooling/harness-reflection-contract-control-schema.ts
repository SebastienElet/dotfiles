import { z } from "zod";

const probabilisticPromotionSchema = z
  .object({
    protocol: z.literal("controlled-marginal-ablation"),
    conditions: z.tuple([
      z.literal("with-exact-candidate-text"),
      z.literal("without-candidate-text"),
    ]),
    controlledConstants: z.tuple([
      z.literal("scenarios"),
      z.literal("environments"),
      z.literal("replicates"),
    ]),
    observableDelta: z.literal("required"),
    withOnlyRuns: z.literal("never-sufficient"),
    activationMeasurementForConditionalSkill: z.literal("required"),
  })
  .strict();

const controlsSchema = z
  .object({
    probabilistic: z.tuple([
      z.literal("always-loaded-instruction"),
      z.literal("conditional-skill"),
      z.literal("project-local-contract"),
    ]),
    enforceable: z.tuple([
      z.literal("hook"),
      z.literal("permission"),
      z.literal("lint"),
      z.literal("type"),
      z.literal("architectural-test"),
    ]),
    probabilisticPromotion: probabilisticPromotionSchema,
    selectionRequiredBeforeApproval: z.literal(true),
  })
  .strict();

const oracleSchema = z
  .object({
    requiredBeforeApproval: z.literal(true),
    enforceable: z.literal("executable-failure-path-and-test-path"),
    probabilistic: z.literal(
      "controlled-marginal-ablation-with-observable-delta",
    ),
    inapplicable: z.literal("reason-required"),
  })
  .strict();

const lifecycleSchema = z
  .object({
    promotion: z.literal("control-kind-specific-green-oracle-required"),
    independentWithOnlySessions: z.literal(
      "never-sufficient-for-probabilistic-control",
    ),
    allowedTransitions: z.tuple([
      z.literal("new-to-candidate"),
      z.literal("new-to-active"),
      z.literal("candidate-to-candidate"),
      z.literal("candidate-to-active"),
      z.literal("active-to-active"),
      z.literal("active-to-retired"),
    ]),
    retiredTerminal: z.literal(true),
    rollback: z.tuple([
      z.literal("two-failed-trials"),
      z.literal("one-safety-regression"),
      z.literal("user-veto"),
    ]),
  })
  .strict();

export { controlsSchema, lifecycleSchema, oracleSchema };
