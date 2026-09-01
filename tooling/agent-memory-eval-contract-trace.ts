import type {
  Agent,
  ContractEvent,
} from "./agent-memory-eval-contract-types.ts";
import { parseContractEvents } from "./agent-memory-eval-contract-event.ts";

const EXPECTED_TRACE_EVENT_COUNT = 2;
const EXPECTED_TRACE_KEYS = [
  "agent",
  "command",
  "event",
  "exit_class",
  "pid",
  "timestamp_ms",
];

function parseEvaluationTrace(
  ...[
    agent,
    stream,
    expectedExitClass = "success",
    expectedCommand = "hook",
  ]: readonly [Agent, string, string?, string?]
): boolean {
  const events = parseContractEvents(stream);
  if (events.length !== EXPECTED_TRACE_EVENT_COUNT) {
    throw new Error("runtime trace must contain two events");
  }
  const [started, completed] = events;
  assertTraceEvents(events, agent, expectedCommand);
  assertTraceSequence(started, completed, expectedExitClass);
  return true;
}

function assertTraceEvents(
  events: readonly ContractEvent[],
  agent: Agent,
  expectedCommand: string,
): void {
  for (const event of events) {
    assertTraceKeys(event);
    assertTraceAgent(event, agent);
    assertTraceCommand(event, expectedCommand);
    assertTraceValues(event);
  }
}

function assertTraceKeys(event: ContractEvent): void {
  if (
    Object.keys(event).toSorted().join(",") !== EXPECTED_TRACE_KEYS.join(",")
  ) {
    throw new Error("runtime trace contains unexpected fields");
  }
}

function assertTraceAgent(event: ContractEvent, agent: Agent): void {
  if (event.agent !== agent) {
    throw new Error("runtime trace agent mismatch");
  }
}

function assertTraceCommand(
  event: ContractEvent,
  expectedCommand: string,
): void {
  if (event.command !== expectedCommand) {
    throw new Error("runtime trace command mismatch");
  }
}

function assertTraceValues(event: ContractEvent): void {
  if (!Number.isSafeInteger(event.pid) || Number(event.pid) < 1) {
    throw new Error("runtime trace pid is invalid");
  }
  if (
    !Number.isSafeInteger(event.timestamp_ms) ||
    Number(event.timestamp_ms) < 0
  ) {
    throw new Error("runtime trace timestamp is invalid");
  }
}

function assertTraceSequence(
  started: ContractEvent | undefined,
  completed: ContractEvent | undefined,
  expectedExitClass: string,
): void {
  if (started?.event !== "started" || started.exit_class !== "started") {
    throw new Error("runtime trace start is invalid");
  }
  const expectedEvent = expectedExitClass === "success" ? "completed" : "error";
  if (
    completed?.event !== expectedEvent ||
    completed.exit_class !== expectedExitClass
  ) {
    throw new Error(
      `runtime trace completion is invalid: event=${String(completed?.event)} exit_class=${String(completed?.exit_class)}`,
    );
  }
  if (
    started.pid !== completed.pid ||
    Number(started.timestamp_ms) > Number(completed.timestamp_ms)
  ) {
    throw new Error("runtime trace order is invalid");
  }
}

export { parseEvaluationTrace };
