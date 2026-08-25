import { HandoffError } from "./agent-handoff-error.ts";

type Usage = Readonly<{
  agent: "Claude Code" | "Codex";
  used: number;
  window?: number;
}>;

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function parseTokenCount(
  value: unknown,
  field: string,
  fallback?: number,
): number {
  if (value === undefined && fallback !== undefined) {
    return fallback;
  }
  if (typeof value !== "number" || !Number.isSafeInteger(value) || value < 0) {
    throw new HandoffError(`invalid ${field}`, 1);
  }
  return value;
}

function parseClaudeUsage(
  record: Readonly<Record<string, unknown>>,
): Usage | undefined {
  if (record.type !== "assistant") {
    return undefined;
  }
  if (
    record.isSidechain !== undefined &&
    typeof record.isSidechain !== "boolean"
  ) {
    throw new HandoffError("invalid Claude isSidechain", 1);
  }
  if (record.isSidechain === true) {
    return undefined;
  }
  if (!isRecord(record.message) || !isRecord(record.message.usage)) {
    return undefined;
  }
  const { usage } = record.message;
  const input = parseTokenCount(usage.input_tokens, "Claude input_tokens");
  const cacheRead = parseTokenCount(
    usage.cache_read_input_tokens,
    "Claude cache_read_input_tokens",
    0,
  );
  const cacheCreation = parseTokenCount(
    usage.cache_creation_input_tokens,
    "Claude cache_creation_input_tokens",
    0,
  );
  const used = input + cacheRead + cacheCreation;
  if (!Number.isSafeInteger(used)) {
    throw new HandoffError("invalid Claude token total", 1);
  }
  return { agent: "Claude Code", used };
}

function parseCodexUsage(
  record: Readonly<Record<string, unknown>>,
): Usage | undefined {
  if (record.type !== "event_msg" || !isRecord(record.payload)) {
    return undefined;
  }
  if (record.payload.type !== "token_count" || !isRecord(record.payload.info)) {
    return undefined;
  }
  const { info } = record.payload;
  if (!isRecord(info.last_token_usage)) {
    return undefined;
  }
  const window = parseTokenCount(
    info.model_context_window,
    "Codex model_context_window",
  );
  if (window === 0) {
    throw new HandoffError("invalid Codex model_context_window", 1);
  }
  return {
    agent: "Codex",
    used: parseTokenCount(
      info.last_token_usage.input_tokens,
      "Codex input_tokens",
    ),
    window,
  };
}

const retainedTranscriptLineCount = 500;

function parseTranscriptRecord(line: string, index: number): unknown {
  try {
    return JSON.parse(line);
  } catch {
    throw new HandoffError(
      `malformed transcript JSON at retained line ${index + 1}`,
      1,
    );
  }
}

function findLatestUsage(transcript: string): Usage {
  const splitLines = transcript.split("\n");
  const physicalLines =
    splitLines.at(-1) === "" ? splitLines.slice(0, -1) : splitLines;
  const lines = physicalLines.slice(-retainedTranscriptLineCount);
  let latest: Usage | undefined = undefined;
  for (const [index, line] of lines.entries()) {
    if (line.trim() !== "") {
      const record = parseTranscriptRecord(line, index);
      if (isRecord(record)) {
        latest = parseClaudeUsage(record) ?? parseCodexUsage(record) ?? latest;
      }
    }
  }
  if (latest === undefined) {
    throw new HandoffError("no supported usage record in transcript", 1);
  }
  return latest;
}

export { findLatestUsage };
export type { Usage };
