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
      z.literal("prepare-registry-only-proposal"),
      z.literal("prepare-exact-registry-diff"),
      z.literal("await-exact-manifest-approval"),
    ]),
    propose: z.tuple([
      z.literal("select-and-propose-control-surface"),
      z.literal("prepare-route-specific-exact-manifest"),
      z.literal("await-exact-manifest-approval"),
    ]),
  })
  .strict();

const approvalBranchesSchema = z
  .object({
    absent: z.tuple([z.literal("render-report-without-mutation")]),
    granted: z.tuple([z.literal("execute-approved-change-order")]),
  })
  .strict();

const workflowRoutesSchema = z
  .object({
    manifestValidation: z
      .object({
        module: z.literal("tooling/harness-reflection-mutation-validation.ts"),
        export: z.literal("validateAppliedHarnessMutation"),
      })
      .strict(),
    registryValidation: z
      .object({
        command: z.literal("bun tooling/invariant-registry-cli.ts"),
      })
      .strict(),
  })
  .strict();

const approvedChangeOrderSchema = z
  .object({
    registryOnly: z.tuple([
      z.literal("prepare-registry-only-proposal"),
      z.literal("prepare-exact-registry-diff"),
      z.literal("present-exact-manifest-for-contextual-human-approval"),
      z.literal("validate-approved-manifest"),
      z.literal("write-approved-registry-replacement-only"),
      z.literal("run-registry-cli-and-declared-oracles"),
      z.literal("render-report"),
    ]),
    surfaceAndRegistry: z.tuple([
      z.literal("select-and-propose-control-surface"),
      z.literal("prepare-route-specific-exact-manifest"),
      z.literal("present-exact-manifest-for-contextual-human-approval"),
      z.literal("apply-surface-with-required-owner"),
      z.literal("run-required-owner-doctor-and-contracts"),
      z.literal("validate-approved-manifest-and-applied-surface"),
      z.literal("write-approved-registry-replacement-only"),
      z.literal("run-registry-cli-and-declared-oracles"),
      z.literal("render-report"),
    ]),
  })
  .strict();

const surfaceOwnersSchema = z
  .object({
    "always-loaded-instruction": z
      .object({
        owner: z.literal("agent-instructions"),
        path: z.literal("harness/AGENTS.md"),
        verification: z.literal("agent-instructions-contracts"),
      })
      .strict(),
    "conditional-skill": z
      .object({
        owner: z.literal("harness-reflection"),
        path: z.literal("harness/invariants/registry.json"),
        verification: z.literal("registry-cli-and-declared-oracles"),
      })
      .strict(),
    "project-local-contract": z
      .object({
        owner: z.literal("agent-instructions"),
        path: z.literal("AGENTS.md"),
        verification: z.literal("agent-instructions-contracts"),
      })
      .strict(),
  })
  .strict();

const externalControlRoutesSchema = z
  .object({
    application: z.literal(
      "owner-specific-exact-diff-and-contract-before-registry-recording",
    ),
    genericManifestValidator: z.literal("not-applicable"),
    surfaces: z
      .object({
        hook: z.tuple([z.literal("scripts"), z.literal("enforcement-code")]),
        permission: z.tuple([z.literal("enforcement-code")]),
        lint: z.tuple([z.literal("scripts"), z.literal("enforcement-code")]),
        type: z.tuple([z.literal("enforcement-code")]),
        "architectural-test": z.tuple([z.literal("enforcement-code")]),
      })
      .strict(),
  })
  .strict();

const manifestValidationSchema = z
  .object({
    appliesTo: z.tuple([
      z.literal("always-loaded-instruction"),
      z.literal("conditional-skill"),
      z.literal("project-local-contract"),
    ]),
    behavior: z.literal("read-only-no-file-writes"),
    candidateTextRule: z.literal(
      "file-surfaces-add-or-remove-exact-text-and-registry-surface-matches-statement",
    ),
    fileSurfaces: z.tuple([
      z.literal("always-loaded-instruction"),
      z.literal("project-local-contract"),
    ]),
    noOpRule: z.literal("every-approved-replacement-differs-from-preimage"),
    registrySurfaces: z.tuple([z.literal("conditional-skill")]),
    semanticClaim: z.literal(
      "exact-file-text-or-registry-statement-plus-owner-doctor-only",
    ),
    transitionKind: z.literal("derived-from-before-and-after"),
  })
  .strict();

const retirementSchema = z
  .object({
    approval: z.literal("new-exact-attestation-recorded"),
    historicalFields: z.literal(
      "unchanged-except-approval-lifecycle-and-retirement",
    ),
    optionalFields: z.tuple([z.literal("replacedBy")]),
    requiredFields: z.tuple([z.literal("retiredAt"), z.literal("reason")]),
    surfaceText: z.literal(
      "file-surface-text-removed-or-registry-surface-retired",
    ),
  })
  .strict();

export {
  approvalBranchesSchema,
  approvedChangeOrderSchema,
  decisionBranchesSchema,
  diagnosticSchema,
  externalControlRoutesSchema,
  harnessGapWorkflowOrderSchema,
  initialWorkflowOrderSchema,
  manifestValidationSchema,
  retirementSchema,
  surfaceOwnersSchema,
  workflowRoutesSchema,
};
