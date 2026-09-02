import { z } from "zod";

const initialWorkflowOrderSchema = z.tuple([
  z.literal("identify-equivalent-failure"),
  z.literal("preserve-factual-evidence"),
  z.literal("inspect-current-guidance"),
  z.literal("classify-diagnostic-cause"),
  z.literal("gate-registry-access"),
]);

const diagnosticSchema = z
  .object({
    classes: z.tuple([
      z.literal("task-specific"),
      z.literal("owned-defect"),
      z.literal("external-transient"),
      z.literal("missing-capability"),
      z.literal("harness-gap"),
    ]),
    harnessGap: z.literal("execute-harness-gap-workflow"),
    other: z.literal("skip-with-reason-and-next-diagnostic-action"),
    registryAccessForOther: z.literal("forbidden"),
  })
  .strict();

const harnessGapWorkflowOrderSchema = z.tuple([
  z.literal("read-authoritative-reference"),
  z.literal("search-registry"),
  z.literal("record-registry-lookup"),
  z.literal("evaluate-concrete-evidence"),
  z.literal("branch-on-evidence"),
]);

const decisionBranchesSchema = z
  .object({
    skip: z.tuple([z.literal("render-report")]),
    link: z.tuple([
      z.literal("hold-session-local"),
      z.literal("await-explicit-approval"),
    ]),
    propose: z.tuple([
      z.literal("hold-session-local"),
      z.literal("await-explicit-approval"),
    ]),
  })
  .strict();

const approvalBranchesSchema = z
  .object({
    absent: z.tuple([z.literal("render-report-without-mutation")]),
    granted: z.tuple([z.literal("execute-approved-compensated-mutation")]),
  })
  .strict();

const workflowRoutesSchema = z
  .object({
    approvedMutation: z
      .object({
        module: z.literal("tooling/harness-reflection-mutation-workflow.ts"),
        export: z.literal("executeHarnessMutationWorkflow"),
        mode: z.literal("approved-mutation"),
      })
      .strict(),
    retirement: z
      .object({
        module: z.literal("tooling/harness-reflection-mutation-workflow.ts"),
        export: z.literal("executeHarnessMutationWorkflow"),
        mode: z.literal("retirement"),
      })
      .strict(),
  })
  .strict();

const approvedMutationSchema = z
  .object({
    guarantee: z.literal("best-effort-compensation-not-atomic"),
    interruptionLimit: z.literal(
      "hard-interruption-has-no-output-or-recovery-guarantee",
    ),
    crashRecovery: z.literal("inspect-git-and-reconcile-before-retry"),
    prepareOrder: z.tuple([
      z.literal("select-control-surface"),
      z.literal("declare-consumers"),
      z.literal("require-control-oracle"),
      z.literal("prepare-selected-control-surface"),
      z.literal("prepare-registry"),
      z.literal("capture-all-file-preimages"),
    ]),
    validationOrder: z.tuple([
      z.literal(
        "validate-prepared-selected-control-surface-with-owned-adapter",
      ),
      z.literal("validate-prepared-registry-with-owned-schema-and-policy"),
      z.literal("validate-persisted-approval-matches-human-context"),
    ]),
    applyOrder: z.tuple([
      z.literal("confirm-all-file-preimages"),
      z.literal("apply-each-file-with-compare-and-swap"),
      z.literal("validate-applied-coherent-change"),
    ]),
    onAnyError: z.tuple([
      z.literal("reconcile-ambiguous-file-outcome"),
      z.literal("compensate-applied-files-with-compare-and-swap"),
      z.literal("report-unresolved-files"),
      z.literal("report-failure"),
    ]),
    successOrder: z.tuple([z.literal("render-report")]),
  })
  .strict();

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
    selectionRequiredAfterApproval: z.literal(true),
  })
  .strict();

const oracleSchema = z
  .object({
    requiredAfterApproval: z.literal(true),
    enforceable: z.literal("executable-failure-path-and-test-path"),
    probabilistic: z.literal(
      "controlled-marginal-ablation-with-observable-delta",
    ),
    inapplicable: z.literal("reason-required"),
  })
  .strict();

const retirementSchema = z
  .object({
    guarantee: z.literal("best-effort-compensation-not-atomic"),
    interruptionLimit: z.literal(
      "hard-interruption-has-no-output-or-recovery-guarantee",
    ),
    crashRecovery: z.literal("inspect-git-and-reconcile-before-retry"),
    requiredFields: z.tuple([z.literal("retiredAt"), z.literal("reason")]),
    optionalFields: z.tuple([z.literal("replacedBy")]),
    prepareOrder: z.tuple([
      z.literal("require-approval"),
      z.literal("lookup-existing-invariant"),
      z.literal("prepare-retired-registry-copy"),
      z.literal("preserve-complete-record-history-in-prepared-registry"),
      z.literal("set-retired-at-in-prepared-registry"),
      z.literal("set-retirement-reason-in-prepared-registry"),
      z.literal("handle-optional-replaced-by-in-prepared-registry"),
      z.literal("prepare-selected-control-surface-copy-if-touched"),
      z.literal("capture-all-file-preimages"),
    ]),
    validationOrder: z.tuple([
      z.literal("validate-complete-record-history-unchanged"),
      z.literal(
        "validate-prepared-selected-control-surface-if-touched-with-owned-adapter",
      ),
      z.literal(
        "validate-prepared-retired-registry-with-owned-schema-and-policy",
      ),
      z.literal("validate-persisted-approval-matches-human-context"),
    ]),
    applyOrder: z.tuple([
      z.literal("confirm-all-file-preimages"),
      z.literal("apply-each-file-with-compare-and-swap"),
      z.literal("validate-applied-coherent-change"),
    ]),
    onAnyError: z.tuple([
      z.literal("reconcile-ambiguous-file-outcome"),
      z.literal("compensate-applied-files-with-compare-and-swap"),
      z.literal("report-unresolved-files"),
      z.literal("report-failure"),
    ]),
    successOrder: z.tuple([z.literal("render-report")]),
  })
  .strict();

const lifecycleSchema = z
  .object({
    promotion: z.literal("control-kind-specific-green-oracle-required"),
    independentWithOnlySessions: z.literal(
      "never-sufficient-for-probabilistic-control",
    ),
    rollback: z.tuple([
      z.literal("two-failed-trials"),
      z.literal("one-safety-regression"),
      z.literal("user-veto"),
    ]),
  })
  .strict();

export {
  approvalBranchesSchema,
  approvedMutationSchema,
  controlsSchema,
  decisionBranchesSchema,
  diagnosticSchema,
  harnessGapWorkflowOrderSchema,
  initialWorkflowOrderSchema,
  lifecycleSchema,
  oracleSchema,
  retirementSchema,
  workflowRoutesSchema,
};
