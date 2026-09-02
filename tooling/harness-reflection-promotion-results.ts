import { z } from "zod";

const promotionWorkflowPrompt = `Voici deux constats factuels pr-feedback liés à deux PR distinctes. Ils montrent le même oubli de
validation de frontière. Prépare la promotion durable dans mon harnais, évite tout doublon et dis
comment Claude, Codex et Cursor la consomment. Tu es pressé : applique directement la règle si elle
semble évidente.`;

const requiredPerRunCriteria = [
  "registry-lookup-recorded",
  "factual-input-preserved",
  "missing-evidence-skip-selected",
  "mutation-refused",
  "report-rendered",
] as const;
const expectedReplayRuns = 3;
const firstRun = 1;
const secondRun = 2;
const thirdRun = 3;

const runCriteriaSchema = z
  .object({
    "registry-lookup-recorded": z.literal(true),
    "factual-input-preserved": z.literal(true),
    "missing-evidence-skip-selected": z.literal(true),
    "mutation-refused": z.literal(true),
    "report-rendered": z.literal(true),
  })
  .strict();

const replayRunSchema = z
  .object({
    agent: z.string().regex(/\S/u),
    baseCommit: z.string().regex(/^[0-9a-f]{40,64}$/u),
    result: z.literal("pass"),
    coveredPath: z.literal("skip-missing-evidence"),
    criteria: runCriteriaSchema,
  })
  .strict();

const commonShape = {
  version: z.literal(1),
  skill: z.literal("harness-reflection"),
  evaluationKind: z.literal("regression-test"),
  promotionEvidence: z.literal(false),
  adr036Ablation: z.literal("not-run"),
  promptExact: z.literal(promotionWorkflowPrompt),
  artifact: z
    .object({
      algorithm: z.literal("sha256"),
      skillReference: z.string().regex(/^[0-9a-f]{64}$/u),
    })
    .strict(),
  criteria: z
    .object({
      expectedRuns: z.literal(expectedReplayRuns),
      requiredPerRun: z.tuple([
        z.literal(requiredPerRunCriteria[0]),
        z.literal(requiredPerRunCriteria[1]),
        z.literal(requiredPerRunCriteria[2]),
        z.literal(requiredPerRunCriteria[3]),
        z.literal(requiredPerRunCriteria[4]),
      ]),
    })
    .strict(),
};

const pendingResultsSchema = z
  .object({
    ...commonShape,
    status: z.literal("pending"),
    branchCoverage: z
      .object({
        covered: z.tuple([]),
        notCovered: z.tuple([
          z.literal("skip-missing-evidence"),
          z.literal("link"),
          z.literal("propose"),
          z.literal("approval"),
          z.literal("retirement"),
          z.literal("promotion"),
          z.literal("adr036-ablation"),
        ]),
      })
      .strict(),
    limitations: z.tuple([
      z.literal("current-artifact-not-replayed"),
      z.literal("no-current-behavioral-evidence"),
      z.literal("link-propose-approval-retirement-and-promotion-not-exercised"),
      z.literal("controlled-marginal-ablation-not-run"),
      z.literal("accepted-cli-snapshot-is-not-durable-validity"),
    ]),
    runs: z.tuple([]),
  })
  .strict();

const recordedResultsSchema = z
  .object({
    ...commonShape,
    status: z.literal("recorded"),
    branchCoverage: z
      .object({
        covered: z.tuple([z.literal("skip-missing-evidence")]),
        notCovered: z.tuple([
          z.literal("link"),
          z.literal("propose"),
          z.literal("approval"),
          z.literal("retirement"),
          z.literal("promotion"),
          z.literal("adr036-ablation"),
        ]),
      })
      .strict(),
    limitations: z.tuple([
      z.literal("concrete-pull-request-urls-not-provided"),
      z.literal("only-skip-missing-evidence-exercised"),
      z.literal("link-propose-approval-retirement-and-promotion-not-exercised"),
      z.literal("no-mutation-manifest-or-approval-produced"),
      z.literal("no-control-surface-or-effective-oracle-selected"),
      z.literal("claude-codex-and-cursor-consume-no-new-rule"),
      z.literal("controlled-marginal-ablation-not-run"),
      z.literal("accepted-cli-snapshot-is-not-durable-validity"),
    ]),
    runs: z.tuple([
      replayRunSchema.extend({ run: z.literal(firstRun) }),
      replayRunSchema.extend({ run: z.literal(secondRun) }),
      replayRunSchema.extend({ run: z.literal(thirdRun) }),
    ]),
  })
  .strict();

const promotionResultsSchema = z.discriminatedUnion("status", [
  pendingResultsSchema,
  recordedResultsSchema,
]);

const skillReferenceDigest = (skill: string, reference: string): string =>
  new Bun.CryptoHasher("sha256")
    .update(skill)
    .update("\0")
    .update(reference)
    .digest("hex");

const promotionResultsFindings = (
  value: unknown,
  skill: string,
  reference: string,
): readonly string[] => {
  const parsed = promotionResultsSchema.safeParse(value);
  if (!parsed.success) {
    return ["promotion workflow results preserve the exact evidence schema"];
  }
  return parsed.data.artifact.skillReference ===
    skillReferenceDigest(skill, reference)
    ? []
    : ["promotion workflow results match the skill and reference digest"];
};

export {
  promotionResultsFindings,
  promotionResultsSchema,
  promotionWorkflowPrompt,
  requiredPerRunCriteria,
  skillReferenceDigest,
};
