import { z } from "zod";

const contextSchema = z.object({ source: z.string().min(1) });
const nativeJsonSchema = z.looseObject({
  rawJSON: z.function({ input: [z.string()], output: z.unknown() }),
});

function parsePreservingJson(text: string): unknown {
  const { rawJSON } = nativeJsonSchema.parse(JSON);
  return JSON.parse(
    text,
    (_key: string, value: unknown, context?: unknown): unknown => {
      if (typeof value !== "number") {
        return value;
      }
      const { source } = contextSchema.parse(context);
      return rawJSON(source);
    },
  );
}

export { parsePreservingJson };
