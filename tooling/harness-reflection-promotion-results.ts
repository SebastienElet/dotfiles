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
const expectedRuns = 3;
const promotionBaseCommit = "a71390e07546a7169dc2bbe2e7d87104ba89240c";
const firstRun = 1;
const secondRun = 2;
const thirdRun = 3;

const criteriaSchema = z
  .object({
    expectedRuns: z.literal(expectedRuns),
    requiredPerRun: z.tuple([
      z.literal(requiredPerRunCriteria[0]),
      z.literal(requiredPerRunCriteria[1]),
      z.literal(requiredPerRunCriteria[2]),
      z.literal(requiredPerRunCriteria[3]),
      z.literal(requiredPerRunCriteria[4]),
    ]),
  })
  .strict();

const artifactSchema = z
  .object({
    algorithm: z.literal("sha256"),
    skillReference: z.string().regex(/^[0-9a-f]{64}$/u),
  })
  .strict();

const runCriteriaSchema = z
  .object({
    registryLookupRecorded: z.literal(true),
    factualInputPreserved: z.literal(true),
    missingEvidenceSkipSelected: z.literal(true),
    mutationRefused: z.literal(true),
    reportRendered: z.literal(true),
  })
  .strict();

const runBaseSchema = z
  .object({
    baseCommit: z.literal(promotionBaseCommit),
    result: z.enum(["pass", "fail"]),
    registryLookup: z.string().min(firstRun),
    criteria: runCriteriaSchema,
    notes: z.string(),
  })
  .strict();
const firstRunSchema = runBaseSchema.extend({ run: z.literal(firstRun) });
const secondRunSchema = runBaseSchema.extend({ run: z.literal(secondRun) });
const thirdRunSchema = runBaseSchema.extend({ run: z.literal(thirdRun) });

const baseResultsSchema = z.object({
  version: z.literal(1),
  skill: z.literal("harness-reflection"),
  evaluationKind: z.literal("regression-test"),
  coveredPath: z.literal("skip-missing-evidence"),
  promotionEvidence: z.literal(false),
  adr036Ablation: z.literal("not-run"),
  limitations: z.tuple([
    z.literal("The prompt contains no concrete pull request URLs."),
    z.literal("Only the missing-evidence skip branch was exercised."),
    z.literal(
      "No promotion, lifecycle, approval, or ablation claim is supported.",
    ),
  ]),
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
  promptExact: z.literal(promotionWorkflowPrompt),
  artifact: artifactSchema,
  criteria: criteriaSchema,
});

const promotionResultsSchema = z.discriminatedUnion("status", [
  baseResultsSchema
    .extend({
      status: z.literal("pending"),
      runs: z.tuple([]),
    })
    .strict(),
  baseResultsSchema
    .extend({
      status: z.literal("recorded"),
      runs: z.tuple([firstRunSchema, secondRunSchema, thirdRunSchema]),
    })
    .strict(),
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
