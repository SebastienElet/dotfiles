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
    mutation: z
      .object({
        module: z.literal("tooling/harness-reflection-mutation-workflow.ts"),
        export: z.literal("executeHarnessMutationWorkflow"),
      })
      .strict(),
  })
  .strict();

const mutationExecutionShape = {
  guarantee: z.literal(
    "cooperative-adapter-lock-with-best-effort-multi-file-compensation-not-atomic",
  ),
  concurrencyScope: z.literal("mutations-through-owned-adapter-only"),
  nonCooperativeWriters: z.literal("outside-guarantee"),
  interruptionLimit: z.literal(
    "hard-interruption-may-leave-lock-temp-or-partial-multi-file-change-without-output",
  ),
  crashRecovery: z.literal(
    "inspect-lock-temp-and-git-before-manual-cleanup-and-retry",
  ),
  applyOrder: z.tuple([
    z.literal("stage-all-replacements-in-same-directories"),
    z.literal("revalidate-all-current-files-under-cooperative-lock"),
    z.literal("atomically-rename-each-file"),
    z.literal("validate-applied-coherent-change"),
  ]),
  onAnyError: z.tuple([
    z.literal("reconcile-ambiguous-file-outcome"),
    z.literal(
      "compensate-applied-files-with-atomic-replacement-when-still-matching",
    ),
    z.literal("report-unresolved-files"),
    z.literal("report-failure"),
  ]),
  successOrder: z.tuple([z.literal("render-report")]),
};
const mutationExecutionSchema = z.object(mutationExecutionShape).strict();

const approvedMutationSchema = z
  .object({
    execution: z.literal("mutationExecution"),
    prepareOrder: z.tuple([
      z.literal("select-supported-control-surface-and-exact-path"),
      z.literal("declare-supported-consumer-mechanisms"),
      z.literal("require-control-oracle"),
      z.literal("prepare-selected-control-surface"),
      z.literal("prepare-registry"),
      z.literal("capture-all-file-preimages-for-approval"),
      z.literal("construct-exact-mutation-manifest"),
      z.literal("await-human-context-approval-for-exact-manifest"),
    ]),
    validationOrder: z.tuple([
      z.literal("validate-request-equals-approved-manifest"),
      z.literal("acquire-owned-cooperative-lock"),
      z.literal("revalidate-approved-preimages-under-lock"),
      z.literal(
        "validate-prepared-selected-control-surface-with-owned-adapter",
      ),
      z.literal("validate-prepared-registry-with-owned-schema-and-policy"),
      z.literal("validate-only-approved-target-registry-delta"),
      z.literal("validate-persisted-approval-matches-accepted-attestation"),
    ]),
  })
  .strict();

const retirementSchema = z
  .object({
    execution: z.literal("mutationExecution"),
    requiredFields: z.tuple([z.literal("retiredAt"), z.literal("reason")]),
    optionalFields: z.tuple([z.literal("replacedBy")]),
    prepareOrder: z.tuple([
      z.literal("lookup-existing-invariant"),
      z.literal("prepare-retired-registry-copy"),
      z.literal("preserve-historical-fields-in-prepared-registry"),
      z.literal("set-retired-at-in-prepared-registry"),
      z.literal("set-retirement-reason-in-prepared-registry"),
      z.literal("handle-optional-replaced-by-in-prepared-registry"),
      z.literal("record-new-approval-attestation-in-prepared-registry"),
      z.literal("prepare-selected-control-surface-copy-if-touched"),
      z.literal("capture-all-file-preimages-for-approval"),
      z.literal("construct-exact-retirement-manifest"),
      z.literal("await-human-context-approval-for-exact-manifest"),
    ]),
    validationOrder: z.tuple([
      z.literal("validate-request-equals-approved-manifest"),
      z.literal("acquire-owned-cooperative-lock"),
      z.literal("revalidate-approved-preimages-under-lock"),
      z.literal(
        "validate-historical-fields-unchanged-except-approval-lifecycle-and-retirement",
      ),
      z.literal(
        "validate-prepared-selected-control-surface-if-touched-with-owned-adapter",
      ),
      z.literal(
        "validate-prepared-retired-registry-with-owned-schema-and-policy",
      ),
      z.literal("validate-only-approved-target-registry-delta"),
      z.literal("validate-persisted-approval-matches-accepted-attestation"),
    ]),
  })
  .strict();

export {
  approvalBranchesSchema,
  approvedMutationSchema,
  decisionBranchesSchema,
  diagnosticSchema,
  harnessGapWorkflowOrderSchema,
  initialWorkflowOrderSchema,
  mutationExecutionSchema,
  retirementSchema,
  workflowRoutesSchema,
};
