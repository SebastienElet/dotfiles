import type {
  Agent,
  ContractEvent,
  SessionExpectation,
  SessionObservation,
  SessionTrace,
} from "./agent-memory-eval-contract-types.ts";
import {
  claudeHookContext,
  claudeVersion,
} from "./agent-memory-eval-claude.ts";
import {
  modelText,
  parseContractEvents,
  recordField,
} from "./agent-memory-eval-contract-event.ts";
import { parseEvaluationTrace } from "./agent-memory-eval-contract-trace.ts";

function parseAgentSession(
  ...[agent, stream, expectation, observation]: readonly [
    Agent,
    string,
    SessionExpectation,
    SessionObservation,
  ]
): SessionTrace {
  const events = parseContractEvents(stream);
  assertSessionVersion(agent, events, expectation.version, observation.version);
  const modelIndex = nonceModelIndex(agent, events, expectation.nonce);
  const eventsBeforeModel = events.slice(0, modelIndex);
  const failure = adapterFailure(
    agent,
    eventsBeforeModel,
    expectation,
    observation,
  );
  if (failure !== undefined) {
    throw new Error(failure);
  }
  return {
    adapterCompletedBeforeModel: true,
    contextBeforeModel: contextObserved(
      agent,
      eventsBeforeModel,
      expectation.source,
    ),
    modelText: modelText(agent, events[modelIndex]),
    version: expectation.version,
  };
}

function assertSessionVersion(
  ...[agent, events, expectedVersion, observedVersion]: readonly [
    Agent,
    readonly ContractEvent[],
    string,
    string | undefined,
  ]
): void {
  const version = agent === "claude" ? claudeVersion(events) : observedVersion;
  if (version !== expectedVersion) {
    throw new Error(`missing ${agent} version metadata`);
  }
}

function nonceModelIndex(
  agent: Agent,
  events: readonly ContractEvent[],
  nonce: string,
): number {
  const modelIndex = events.findIndex((event) =>
    modelText(agent, event).includes(nonce),
  );
  if (modelIndex === -1) {
    throw new Error(`missing ${agent} nonce model event`);
  }
  return modelIndex;
}

function adapterFailure(
  ...[agent, events, expectation, observation]: readonly [
    Agent,
    readonly ContractEvent[],
    SessionExpectation,
    SessionObservation,
  ]
): string | undefined {
  if (observation.traceCompletedBeforeModel !== true) {
    return `${agent} runtime trace completed after model`;
  }
  if (!adapterCompleted(agent, events, expectation, observation)) {
    return adapterFailureMessage(agent, events, expectation);
  }
  return undefined;
}

function adapterCompleted(
  ...[agent, events, expectation, observation]: readonly [
    Agent,
    readonly ContractEvent[],
    SessionExpectation,
    SessionObservation,
  ]
): boolean {
  parseEvaluationTrace(agent, observation.runtimeTrace ?? "");
  if (agent === "codex") {
    return (
      observation.cacheAbsentBefore === true &&
      observation.cacheCompletedBeforeModel === true &&
      observation.cachePath === expectation.cache
    );
  }
  if (agent === "claude") {
    return claudeHookContext(events, expectation.source) !== "";
  }
  return cursorAdapterOrder(events, expectation.runtime);
}

function adapterFailureMessage(
  agent: Agent,
  events: readonly ContractEvent[],
  expectation: SessionExpectation,
): string {
  if (agent === "codex") {
    return "codex cache was not completed before model";
  }
  if (
    agent === "claude" &&
    claudeHookContext(events, expectation.source) === ""
  ) {
    return "claude hook response missing applicable context before model";
  }
  if (agent === "cursor" && !cursorAdapterOrder(events, expectation.runtime)) {
    return "cursor runtime command did not complete before model";
  }
  return `${agent} adapter did not complete before model`;
}

function contextObserved(
  agent: Agent,
  events: readonly ContractEvent[],
  source: string,
): boolean {
  if (agent === "codex") {
    return true;
  }
  if (agent === "claude") {
    return claudeHookContext(events, source) !== "";
  }
  return (
    cursorAdapterOrder(events, "") ||
    events.some((event) => JSON.stringify(event).includes(source))
  );
}

function cursorAdapterOrder(
  events: readonly ContractEvent[],
  runtime: string,
): boolean {
  const skill = events.findIndex((event) => cursorSkillReadCompleted(event));
  const command = events.findIndex((event) =>
    cursorRuntimeCommand(event, runtime),
  );
  const commandEvent = command === -1 ? undefined : events[command];
  const callId = commandEvent?.call_id;
  const completed = events.findIndex(
    (event, index) =>
      index > command && event.call_id === callId && shellCallSucceeded(event),
  );
  return skill !== -1 && command > skill && completed > command;
}

function cursorSkillReadCompleted(event: ContractEvent): boolean {
  const read = recordField(recordField(event, "tool_call"), "readToolCall");
  const path = recordField(read, "args")?.path;
  const success = recordField(recordField(read, "result"), "success");
  return (
    typeof path === "string" &&
    path.endsWith("/SKILL.md") &&
    success !== undefined
  );
}

function cursorRuntimeCommand(event: ContractEvent, runtime: string): boolean {
  if (runtime === "") {
    return false;
  }
  const shell = recordField(recordField(event, "tool_call"), "shellToolCall");
  const command = recordField(shell, "args")?.command;
  return (
    typeof command === "string" &&
    command === `${runtime} retrieve --query-stdin --format json`
  );
}

function shellCallSucceeded(event: ContractEvent): boolean {
  const shell = recordField(recordField(event, "tool_call"), "shellToolCall");
  return (
    shell?.result !== undefined &&
    recordField(recordField(shell, "result"), "success") !== undefined
  );
}

export { parseAgentSession };
