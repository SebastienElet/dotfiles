import {
  approvalBranchesSchema,
  approvedMutationSchema,
  controlsSchema,
  decisionBranchesSchema,
  diagnosticSchema,
  harnessGapWorkflowOrderSchema,
  initialWorkflowOrderSchema,
  lifecycleSchema,
  mutationExecutionSchema,
  oracleSchema,
  retirementSchema,
  workflowRoutesSchema,
} from "./harness-reflection-contract-workflow-schema.ts";
import { z } from "zod";

const registryClassSchema = z.enum([
  "not-applied",
  "not-loaded",
  "unknown",
  "blind-spot",
  "judgment",
]);
const decisionSchema = z.enum(["skip", "link", "propose"]);

const harnessReflectionContractSchema = z
  .object({
    version: z.literal(1),
    initialWorkflowOrder: initialWorkflowOrderSchema,
    diagnostic: diagnosticSchema,
    harnessGapWorkflowOrder: harnessGapWorkflowOrderSchema,
    decisionBranches: decisionBranchesSchema,
    approvalBranches: approvalBranchesSchema,
    workflowRoutes: workflowRoutesSchema,
    mutationExecution: mutationExecutionSchema,
    approvedMutation: approvedMutationSchema,
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
        prFeedbackBoundary: z
          .object({
            input: z.literal("provided-factual-report-only"),
            directForgeIngestion: z.literal("forbidden"),
            historicalReconstruction: z.literal("forbidden"),
            collectionRole: z.literal("none"),
          })
          .strict(),
        syntheticSources: z.literal("forbidden"),
      })
      .strict(),
    approval: z
      .object({
        requiredBeforeMutation: z.literal(true),
        preApprovalState: z.literal("session-local"),
        manifestRequired: z.literal(true),
        manifestTiming: z.literal("present-exact-manifest-before-approval"),
        manifestContents: z.tuple([
          z.literal("kind"),
          z.literal("exact-paths"),
          z.literal("exact-preimages"),
          z.literal("exact-replacements"),
          z.literal("target-invariant-id"),
          z.literal("exact-target-before-and-after"),
        ]),
        timePressureBypass: z.literal(false),
        inputSource: z.literal("human-context"),
        authentication: z.literal("not-performed"),
        registryRecordMeaning: z.literal(
          "provided-context-not-independent-proof",
        ),
        agentSelfAssertion: z.literal("forbidden"),
      })
      .strict(),
    controls: controlsSchema,
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
    oracle: oracleSchema,
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
    retirement: retirementSchema,
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
    lifecycle: lifecycleSchema,
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
