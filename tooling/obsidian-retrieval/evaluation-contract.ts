import { z } from "zod";

const evaluationDocumentSchema = z
  .object({
    skill: z.literal("obsidian-retrieval"),
    version: z.string().min(1),
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
  })
  .strict();

export const validateEvaluations = (value: unknown): string[] => {
  const result = evaluationDocumentSchema.safeParse(value);
  if (!result.success) return [z.prettifyError(result.error)];
  const queries = result.data.queries;
  const texts = queries.map((query) => String(query.query));
  const reasons = queries.map((query) => String(query.reason));
  const activations = queries.map((query) => Boolean(query.should_activate));
  const coverage = [
    activations.includes(true) && activations.includes(false),
    texts.some((text) => /exact|title|identifier|tag/i.test(text)),
    texts.some((text) => /concept|idea|theme/i.test(text)),
    texts.some((text) => /backlink|propert|task|base/i.test(text)),
    texts.some((text) => /web|weather/i.test(text)),
    texts.some((text) => /write|create|edit/i.test(text)),
    reasons.some((reason) => /missing|unavailable/i.test(reason)),
    reasons.some((reason) => /empty|no match/i.test(reason)),
  ];
  return coverage.every(Boolean)
    ? []
    : ["evaluations do not cover the acceptance cases"];
};
