import { z } from "zod";

const evaluationDocumentSchema = z
  .object({
    queries: z
      .array(
        z
          .object({
            query: z.string(),
            reason: z.string(),
            should_activate: z.boolean(),
          })
          .strict(),
      )
      .min(1),
    skill: z.literal("obsidian-retrieval"),
    version: z.string().min(1),
  })
  .strict();

export const validateEvaluations = (value: unknown): string[] => {
  const result = evaluationDocumentSchema.safeParse(value);
  if (!result.success) {
    return [z.prettifyError(result.error)];
  }
  const { queries } = result.data;
  const texts = queries.map((query) => query.query);
  const reasons = queries.map((query) => query.reason);
  const activations = new Set(queries.map((query) => query.should_activate));
  const coverage = [
    activations.has(true) && activations.has(false),
    texts.some((text) => /exact|title|identifier|tag/iu.test(text)),
    texts.some((text) => /concept|idea|theme/iu.test(text)),
    texts.some((text) => /backlink|property|properties|task|base/iu.test(text)),
    texts.some((text) => /web|weather/iu.test(text)),
    texts.some((text) => /write|create|edit/iu.test(text)),
    reasons.some((reason) => /missing|unavailable/iu.test(reason)),
    reasons.some((reason) => /empty|no match/iu.test(reason)),
  ];
  return coverage.every(Boolean)
    ? []
    : ["evaluations do not cover the acceptance cases"];
};
