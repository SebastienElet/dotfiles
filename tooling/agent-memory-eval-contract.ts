import type { Agent, AgentText } from "./agent-memory-eval-contract-types.ts";
import {
  modelText,
  parseContractEvents,
} from "./agent-memory-eval-contract-event.ts";
import { claudeVersion } from "./agent-memory-eval-claude.ts";

function parseAgentText(
  agent: Agent,
  stream: string,
  externalVersion?: string,
): AgentText {
  const events = parseContractEvents(stream);
  const version = agent === "claude" ? claudeVersion(events) : externalVersion;
  if (typeof version !== "string" || version.length === 0) {
    throw new Error(`missing ${agent} version metadata`);
  }
  const text = events
    .map((event) => modelText(agent, event))
    .filter((value) => value !== "")
    .join("\n");
  if (text === "") {
    throw new Error(`missing ${agent} model event`);
  }
  return { modelText: text, version };
}

export { modelText } from "./agent-memory-eval-contract-event.ts";
export { parseAgentSession } from "./agent-memory-eval-contract-session.ts";
export { parseEvaluationTrace } from "./agent-memory-eval-contract-trace.ts";
export type {
  AgentText,
  SessionExpectation,
  SessionObservation,
  SessionTrace,
} from "./agent-memory-eval-contract-types.ts";
export { parseAgentText };
