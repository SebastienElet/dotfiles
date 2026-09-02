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

const promotionResultsSchema = z
  .object({
    version: z.literal(1),
    skill: z.literal("harness-reflection"),
    status: z.literal("pending"),
    evaluationKind: z.literal("regression-test"),
    promotionEvidence: z.literal(false),
    adr036Ablation: z.literal("not-run"),
    promptExact: z.literal(promotionWorkflowPrompt),
    artifact: artifactSchema,
    criteria: criteriaSchema,
    runs: z.tuple([]),
  })
  .strict();

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
