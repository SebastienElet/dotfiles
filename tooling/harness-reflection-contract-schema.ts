import { z } from "zod";

const registryClassSchema = z.enum([
  "not-applied",
  "not-loaded",
  "unknown",
  "blind-spot",
  "judgment",
]);
const decisionSchema = z.enum(["skip", "link", "propose"]);
const probabilisticSurfaceSchema = z.enum([
  "always-loaded-instruction",
  "conditional-skill",
  "project-local-contract",
]);
const enforceableSurfaceSchema = z.enum([
  "hook",
  "permission",
  "lint",
  "type",
  "architectural-test",
]);
const harnessReflectionContractSchema = z
  .object({
    version: z.literal(1),
    initialWorkflowOrder: z.tuple([
      z.literal("identify-equivalent-failure"),
      z.literal("preserve-factual-evidence"),
      z.literal("inspect-current-guidance"),
      z.literal("classify-diagnostic-cause"),
      z.literal("gate-registry-access"),
    ]),
    diagnostic: z
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
      .strict(),
    harnessGapWorkflowOrder: z.tuple([
      z.literal("read-authoritative-reference"),
      z.literal("search-registry"),
      z.literal("record-registry-lookup"),
      z.literal("evaluate-concrete-evidence"),
      z.literal("branch-on-evidence"),
    ]),
    decisionBranches: z
      .object({
        skip: z.tuple([z.literal("render-report")]),
        link: z.tuple([
          z.literal("hold-session-local"),
          z.literal("render-report"),
        ]),
        propose: z.tuple([
          z.literal("hold-session-local"),
          z.literal("render-report"),
        ]),
      })
      .strict(),
    approvedMutationOrder: z.tuple([
      z.literal("require-approval"),
      z.literal("select-control-surface"),
      z.literal("declare-consumers"),
      z.literal("require-oracle"),
      z.literal("run-cli"),
      z.literal("render-report"),
    ]),
    registry: z
      .object({
        path: z.literal("harness/invariants/registry.json"),
        classes: z.tuple([
          registryClassSchema.extract(["not-applied"]),
          registryClassSchema.extract(["not-loaded"]),
          registryClassSchema.extract(["unknown"]),
          registryClassSchema.extract(["blind-spot"]),
          registryClassSchema.extract(["judgment"]),
        ]),
        decisions: z.tuple([
          decisionSchema.extract(["skip"]),
          decisionSchema.extract(["link"]),
          decisionSchema.extract(["propose"]),
        ]),
        judgmentDecision: z.literal("skip"),
        existingInvariantDecision: z.literal("link"),
        linkEffect: z.literal("add-source-without-duplicate-record"),
        missingInvariantDecision: z.literal(
          "propose-if-evidence-threshold-met",
        ),
      })
      .strict(),
    evidence: z
      .object({
        factualPrFeedback: z.literal("immutable"),
        concretePrUrls: z.literal("required"),
        evaluationTiming: z.literal("after-registry-lookup-recorded"),
        missingEvidenceDecision: z.literal("skip"),
        missingEvidenceWorkflow: z.tuple([
          z.literal("choose-skip"),
          z.literal("render-report"),
        ]),
        presentEvidenceWorkflow: z.tuple([
          z.literal("classify-registry-cause"),
          z.literal("choose-decision"),
        ]),
        promotionThreshold: z.literal(
          "two-distinct-pull-requests-or-high-severity",
        ),
        syntheticSources: z.literal("forbidden"),
      })
      .strict(),
    approval: z
      .object({
        requiredBeforeMutation: z.literal(true),
        preApprovalState: z.literal("session-local"),
        timePressureBypass: z.literal(false),
      })
      .strict(),
    controls: z
      .object({
        probabilistic: z.tuple([
          probabilisticSurfaceSchema.extract(["always-loaded-instruction"]),
          probabilisticSurfaceSchema.extract(["conditional-skill"]),
          probabilisticSurfaceSchema.extract(["project-local-contract"]),
        ]),
        enforceable: z.tuple([
          enforceableSurfaceSchema.extract(["hook"]),
          enforceableSurfaceSchema.extract(["permission"]),
          enforceableSurfaceSchema.extract(["lint"]),
          enforceableSurfaceSchema.extract(["type"]),
          enforceableSurfaceSchema.extract(["architectural-test"]),
        ]),
        selectionRequiredAfterApproval: z.literal(true),
      })
      .strict(),
    consumers: z
      .object({
        required: z.tuple([
          z.literal("claude"),
          z.literal("codex"),
          z.literal("cursor"),
        ]),
        declaration: z.literal(
          "independent-supported-mechanism-or-unsupported-reason",
        ),
      })
      .strict(),
    oracle: z
      .object({
        requiredAfterApproval: z.literal(true),
        enforceable: z.literal("executable-failure-path-and-test-path"),
        probabilistic: z.literal("behavioral-trial-with-environment"),
        inapplicable: z.literal("reason-required"),
      })
      .strict(),
    routes: z
      .object({
        skillChange: z.literal("skill-manager"),
        instructionChange: z.literal("agent-instructions"),
      })
      .strict(),
    cli: z
      .object({
        command: z.literal("bun tooling/invariant-registry-cli.ts"),
        timing: z.literal("immediately-before-report"),
        claim: z.literal("accepted-snapshot-read-in-execution-environment"),
        durableValidityClaim: z.literal(false),
      })
      .strict(),
    retirement: z
      .object({
        requiredFields: z.tuple([z.literal("retiredAt"), z.literal("reason")]),
        optionalFields: z.tuple([z.literal("replacedBy")]),
      })
      .strict(),
    proposal: z
      .object({
        requiredFields: z.tuple([
          z.literal("trigger"),
          z.literal("desired-behavior"),
          z.literal("scope"),
          z.literal("supporting-evidence"),
          z.literal("counterexample"),
          z.literal("falsifier"),
          z.literal("expiry-condition"),
          z.literal("cheapest-behavioral-trial"),
        ]),
      })
      .strict(),
    lifecycle: z
      .object({
        promotion: z.literal(
          "three-independent-sessions-without-contradictory-result",
        ),
        rollback: z.tuple([
          z.literal("two-failed-trials"),
          z.literal("one-safety-regression"),
          z.literal("user-veto"),
        ]),
      })
      .strict(),
    report: z
      .object({
        appliesToDecisions: z.tuple([
          decisionSchema.extract(["skip"]),
          decisionSchema.extract(["link"]),
          decisionSchema.extract(["propose"]),
        ]),
        registryLookupAfterHarnessGap: z.literal(
          "required-even-when-evidence-missing",
        ),
        requiredFields: z.tuple([
          z.literal("registry-lookup"),
          z.literal("decision-and-reason"),
          z.literal("control-kind-and-surface"),
          z.literal("sources-evidence-and-missing-proof"),
          z.literal("oracle-or-inapplicability"),
          z.literal("approval-status"),
          z.literal("claude-consumer"),
          z.literal("codex-consumer"),
          z.literal("cursor-consumer"),
          z.literal("cli-snapshot-result-or-not-run-reason"),
        ]),
      })
      .strict(),
  })
  .strict();

type HarnessReflectionContract = z.output<
  typeof harnessReflectionContractSchema
>;

export { harnessReflectionContractSchema, type HarnessReflectionContract };
