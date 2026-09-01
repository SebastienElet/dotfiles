import type { Agent } from "./agent-memory-eval-process.ts";
import {
  claudeHookContext,
  claudeVersion,
} from "./agent-memory-eval-claude.ts";

type SessionExpectation = Readonly<{
  cache?: string;
  nonce: string;
  runtime: string;
  source: string;
  version: string;
}>;
type SessionObservation = Readonly<{
  cacheAbsentBefore?: boolean;
  cacheCompletedBeforeModel?: boolean;
  cachePath?: string;
  runtimeTrace?: string;
  traceCompletedBeforeModel?: boolean;
  version?: string;
}>;
type SessionTrace = Readonly<{
  adapterCompletedBeforeModel: boolean;
  contextBeforeModel: boolean;
  modelText: string;
  version: string;
}>;

type AgentText = Readonly<{ modelText: string; version: string }>;

function parseEvaluationTrace(
  agent: Agent,
  stream: string,
  expectedExitClass = "success",
  expectedCommand = "hook",
): boolean {
  const events = stream
    .split("\n")
    .filter((line) => line.trim() !== "")
    .map(parseEvent);
  if (events.length !== 2) throw new Error("runtime trace must contain two events");
  const [started, completed] = events;
  const expectedKeys = ["agent", "command", "event", "exit_class", "pid", "timestamp_ms"];
  for (const event of events) {
    if (Object.keys(event).sort().join(",") !== expectedKeys.join(",")) {
      throw new Error("runtime trace contains unexpected fields");
    }
    if (event.agent !== agent) throw new Error("runtime trace agent mismatch");
    if (event.command !== expectedCommand) throw new Error("runtime trace command mismatch");
    if (!Number.isSafeInteger(event.pid) || Number(event.pid) < 1) {
      throw new Error("runtime trace pid is invalid");
    }
    if (!Number.isSafeInteger(event.timestamp_ms) || Number(event.timestamp_ms) < 0) {
      throw new Error("runtime trace timestamp is invalid");
    }
  }
  if (started?.event !== "started" || started.exit_class !== "started") {
    throw new Error("runtime trace start is invalid");
  }
  const expectedEvent = expectedExitClass === "success" ? "completed" : "error";
  if (completed?.event !== expectedEvent || completed.exit_class !== expectedExitClass) {
    throw new Error(
      `runtime trace completion is invalid: event=${String(completed?.event)} exit_class=${String(completed?.exit_class)}`,
    );
  }
  if (started.pid !== completed.pid || Number(started.timestamp_ms) > Number(completed.timestamp_ms)) {
    throw new Error("runtime trace order is invalid");
  }
  return true;
}

function parseAgentSession(
  agent: Agent,
  stream: string,
  expectation: SessionExpectation,
  observation: SessionObservation,
): SessionTrace {
  const events = stream
    .split("\n")
    .filter((line) => line.trim() !== "")
    .map(parseEvent);
  const version = agent === "claude" ? claudeVersion(events) : observation.version;
  if (version !== expectation.version) throw new Error(`missing ${agent} version metadata`);
  const modelIndex = events.findIndex((event) => modelText(agent, event).includes(expectation.nonce));
  if (modelIndex < 0) throw new Error(`missing ${agent} nonce model event`);
  const adapterCompletedBeforeModel = adapterCompletion(
    agent,
    events.slice(0, modelIndex),
    expectation,
    observation,
  );
  if (!adapterCompletedBeforeModel) {
    throw new Error(adapterFailure(agent, events.slice(0, modelIndex), expectation, observation));
  }
  const contextBeforeModel = contextObserved(agent, events.slice(0, modelIndex), expectation.source);
  return {
    adapterCompletedBeforeModel: true,
    contextBeforeModel,
    modelText: modelText(agent, events[modelIndex]),
    version: expectation.version,
  };
}

function adapterFailure(
  agent: Agent,
  events: readonly Readonly<Record<string, unknown>>[],
  expectation: SessionExpectation,
  observation: SessionObservation,
): string {
  if (observation.traceCompletedBeforeModel !== true) {
    return `${agent} runtime trace completed after model`;
  }
  if (agent === "codex") return "codex cache was not completed before model";
  if (agent === "claude" && claudeHookContext(events, expectation.source) === "") {
    return "claude hook response missing applicable context before model";
  }
  if (agent === "cursor" && !cursorAdapterOrder(events, expectation.runtime)) {
    return "cursor runtime command did not complete before model";
  }
  return `${agent} adapter did not complete before model`;
}

function parseEvent(line: string, index: number): Readonly<Record<string, unknown>> {
  try {
    const parsed: unknown = JSON.parse(line);
    if (!isRecord(parsed)) throw new Error();
    return parsed;
  } catch {
    throw new Error(`malformed JSONL at line ${index + 1}`);
  }
}

function modelText(agent: Agent, event: Readonly<Record<string, unknown>> | undefined): string {
  if (agent === "codex") {
    const item = recordField(event, "item");
    return item?.type === "agent_message" && typeof item.text === "string" ? item.text : "";
  }
  const message = recordField(event, "message");
  const content = message?.content;
  if (!Array.isArray(content)) return "";
  return content
    .map((block) => (isRecord(block) && block.type === "text" && typeof block.text === "string" ? block.text : ""))
    .join("\n");
}

function adapterCompletion(
  agent: Agent,
  events: readonly Readonly<Record<string, unknown>>[],
  expectation: SessionExpectation,
  observation: SessionObservation,
): boolean {
  const traceValid = parseEvaluationTrace(agent, observation.runtimeTrace ?? "");
  if (!traceValid || observation.traceCompletedBeforeModel !== true) return false;
  if (agent === "codex") {
    return (
      observation.cacheAbsentBefore === true &&
      observation.cacheCompletedBeforeModel === true &&
      observation.cachePath === expectation.cache
    );
  }
  if (agent === "claude") return claudeHookContext(events, expectation.source) !== "";
  return cursorAdapterOrder(events, expectation.runtime);
}

function contextObserved(
  agent: Agent,
  events: readonly Readonly<Record<string, unknown>>[],
  source: string,
): boolean {
  if (agent === "codex") return true;
  if (agent === "claude") return claudeHookContext(events, source) !== "";
  return cursorAdapterOrder(events, "") || events.some((event) => JSON.stringify(event).includes(source));
}

function cursorAdapterOrder(
  events: readonly Readonly<Record<string, unknown>>[],
  runtime: string,
): boolean {
  const skill = events.findIndex((event) => cursorSkillReadCompleted(event));
  const command = events.findIndex((event) => cursorRuntimeCommand(event, runtime));
  const callId = command >= 0 ? events[command]?.call_id : undefined;
  const completed = events.findIndex(
    (event, index) =>
      index > command &&
      event.call_id === callId &&
      recordField(recordField(event, "tool_call"), "shellToolCall")?.result !== undefined &&
      recordField(
        recordField(recordField(recordField(event, "tool_call"), "shellToolCall"), "result"),
        "success",
      ) !== undefined,
  );
  return skill >= 0 && command > skill && completed > command;
}

function cursorSkillReadCompleted(event: Readonly<Record<string, unknown>>): boolean {
  const read = recordField(recordField(event, "tool_call"), "readToolCall");
  const path = recordField(read, "args")?.path;
  const success = recordField(recordField(read, "result"), "success");
  return typeof path === "string" && path.endsWith("/SKILL.md") && success !== undefined;
}

function cursorRuntimeCommand(
  event: Readonly<Record<string, unknown>>,
  runtime: string,
): boolean {
  if (runtime === "") return false;
  const shell = recordField(recordField(event, "tool_call"), "shellToolCall");
  const command = recordField(shell, "args")?.command;
  return (
    typeof command === "string" &&
    command === `${runtime} retrieve --query-stdin --format json`
  );
}

function recordField(
  record: Readonly<Record<string, unknown>> | undefined,
  field: string,
): Readonly<Record<string, unknown>> | undefined {
  const value = record?.[field];
  return isRecord(value) ? value : undefined;
}

function isRecord(value: unknown): value is Readonly<Record<string, unknown>> {
  return value !== null && typeof value === "object" && !Array.isArray(value);
}

function parseAgentText(
  agent: Agent,
  stream: string,
  externalVersion?: string,
): AgentText {
  const events = stream
    .split("\n")
    .filter((line) => line.trim() !== "")
    .map(parseEvent);
  const version = agent === "claude" ? claudeVersion(events) : externalVersion;
  if (typeof version !== "string" || version.length === 0) {
    throw new Error(`missing ${agent} version metadata`);
  }
  const text = events.map((event) => modelText(agent, event)).filter(Boolean).join("\n");
  if (text === "") throw new Error(`missing ${agent} model event`);
  return { modelText: text, version };
}

export {
  modelText,
  parseAgentSession,
  parseAgentText,
  parseEvaluationTrace,
};
export type { AgentText, SessionExpectation, SessionObservation, SessionTrace };
