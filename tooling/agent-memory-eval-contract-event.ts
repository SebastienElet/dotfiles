import type {
  Agent,
  ContractEvent,
} from "./agent-memory-eval-contract-types.ts";

function parseContractEvents(stream: string): readonly ContractEvent[] {
  return stream
    .split("\n")
    .filter((line) => line.trim() !== "")
    .map((line, index) => parseContractEvent(line, index));
}

function parseContractEvent(line: string, index: number): ContractEvent {
  try {
    const parsed: unknown = JSON.parse(line);
    if (!isRecord(parsed)) {
      throw new Error("agent event must be an object");
    }
    return parsed;
  } catch {
    throw new Error(`malformed JSONL at line ${index + 1}`);
  }
}

function modelText(agent: Agent, event: ContractEvent | undefined): string {
  if (agent === "codex") {
    const item = recordField(event, "item");
    return item?.type === "agent_message" && typeof item.text === "string"
      ? item.text
      : "";
  }
  const message = recordField(event, "message");
  const content = message?.content;
  if (!Array.isArray(content)) {
    return "";
  }
  return content
    .map((block) =>
      isRecord(block) && block.type === "text" && typeof block.text === "string"
        ? block.text
        : "",
    )
    .join("\n");
}

function recordField(
  record: ContractEvent | undefined,
  field: string,
): ContractEvent | undefined {
  const value = record?.[field];
  return isRecord(value) ? value : undefined;
}

function isRecord(value: unknown): value is ContractEvent {
  return value !== null && typeof value === "object" && !Array.isArray(value);
}

export { modelText, parseContractEvents, recordField };
