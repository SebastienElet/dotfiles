import type { Run } from "./report-schema.ts";
import { z } from "zod";

const eventSchema = z.looseObject({ type: z.string() }).readonly();
const usageSchema = z.object({
  usage: z.object({
    input_tokens: z.int().nonnegative(),
    cached_input_tokens: z.int().nonnegative(),
    output_tokens: z.int().nonnegative(),
  }),
});
const itemSchema = z.object({
  item: z.object({ id: z.string(), type: z.string() }).readonly(),
});

type CodexMetrics = Readonly<{
  tokens: NonNullable<Run["tokens"]>;
  toolCalls: number;
}>;

function parseCodexEvents(output: string): CodexMetrics {
  const events = output
    .split("\n")
    .filter(Boolean)
    .map((line) => eventSchema.parse(JSON.parse(line)));
  if (
    events.some(
      (event) => event.type === "error" || event.type === "turn.failed",
    )
  ) {
    throw new Error("Agent failure event");
  }
  const completed = events.filter((event) => event.type === "turn.completed");
  if (completed.length !== 1) {
    throw new Error("One completed turn required");
  }
  const { usage } = usageSchema.parse(completed[0]);
  const calls = events
    .filter((event) => event.type === "item.completed")
    .map((event) => itemSchema.parse(event).item)
    .filter((item) =>
      [
        "command_execution",
        "mcp_tool_call",
        "web_search",
        "file_change",
      ].includes(item.type),
    );
  return {
    tokens: {
      input: usage.input_tokens,
      cachedInput: usage.cached_input_tokens,
      output: usage.output_tokens,
    },
    toolCalls: new Set(calls.map((item) => item.id)).size,
  };
}

export { type CodexMetrics, parseCodexEvents };
