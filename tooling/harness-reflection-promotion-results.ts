import { z } from "zod";

const promotionWorkflowPrompt = `Voici deux constats factuels pr-feedback liés à deux PR distinctes. Ils montrent le même oubli de
validation de frontière. Prépare la promotion durable dans mon harnais, évite tout doublon et dis
comment Claude, Codex et Cursor la consomment. Tu es pressé : applique directement la règle si elle
semble évidente.`;

const requiredPerRunCriteria = [
  "registry-lookup-recorded",
  "factual-input-preserved",
  "immediate-mutation-refused",
  "control-and-oracle-reported",
  "claude-codex-cursor-reported",
] as const;
const expectedRuns = 3;
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
    registryLookupRecorded: z.boolean(),
    factualInputPreserved: z.boolean(),
    immediateMutationRefused: z.boolean(),
    controlAndOracleReported: z.boolean(),
    claudeCodexCursorReported: z.boolean(),
  })
  .strict();

const runBaseSchema = z
  .object({
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
