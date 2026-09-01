import { expect, test } from "bun:test";

import { parseAgentSession } from "./agent-memory-eval-contract.ts";

const cursorCompletionIndex = 3;
const hookFailureExitCode = 7;

test("matches Claude hook response before assistant nonce", () => {
  const expectation = expected("nonce-claude", "2.1.251", "/tmp/runtime");
  const events = claudeEvents(expectation.nonce, expectation.version);
  const observation = traced("claude");
  expect(
    parseAgentSession("claude", jsonLines(events), expectation, observation)
      .contextBeforeModel,
  ).toBe(true);
  const structuredOutput = events.map(
    (event: Readonly<Record<string, unknown>>) => structureHookResponse(event),
  );
  expect(
    parseAgentSession(
      "claude",
      jsonLines(structuredOutput),
      expectation,
      observation,
    ).contextBeforeModel,
  ).toBe(true);
  expectClaudeMutationFails({
    events,
    expectation,
    observation,
    mutation: wrongHookId,
  });
  expectClaudeMutationFails({
    events,
    expectation,
    observation,
    mutation: wrongHookExitCode,
  });
  expect(() =>
    parseAgentSession(
      "claude",
      jsonLines([events[0], events[1], events[3], events[2]]),
      expectation,
      observation,
    ),
  ).toThrow();
});

test("matches Cursor skill and shell completion by call ID", () => {
  const expectation = expected("nonce-cursor", "2026.08.25", "/tmp/runtime");
  const events = cursorEvents(expectation.nonce, expectation.runtime);
  const observation = { ...traced("cursor"), version: expectation.version };
  expect(
    parseAgentSession("cursor", jsonLines(events), expectation, observation)
      .contextBeforeModel,
  ).toBe(true);
  const mutant = [...events];
  mutant[cursorCompletionIndex] = {
    ...mutant[cursorCompletionIndex],
    call_id: "wrong",
  };
  expect(() =>
    parseAgentSession("cursor", jsonLines(mutant), expectation, observation),
  ).toThrow();
});

function expected(
  nonce: string,
  version: string,
  runtime: string,
): {
  cache: string;
  nonce: string;
  runtime: string;
  source: string;
  version: string;
} {
  return {
    cache: "/tmp/store/oracle-cache.json",
    nonce,
    runtime,
    source: "proof.txt",
    version,
  };
}

function jsonLines(events: readonly unknown[]): string {
  return events.map((event) => JSON.stringify(event)).join("\n");
}

function claudeEvents(
  nonce: string,
  version: string,
): Record<string, unknown>[] {
  return [
    { type: "system", subtype: "init", claude_code_version: version },
    {
      type: "system",
      subtype: "hook_started",
      hook_id: "hook-1",
      hook_event: "UserPromptSubmit",
    },
    {
      type: "system",
      subtype: "hook_response",
      hook_id: "hook-1",
      hook_event: "UserPromptSubmit",
      exit_code: 0,
      stdout: '{"source":"proof.txt","verdict_age_milliseconds":0}',
    },
    {
      type: "assistant",
      message: { content: [{ type: "text", text: nonce }] },
    },
  ];
}

function cursorEvents(
  nonce: string,
  runtime: string,
): Record<string, unknown>[] {
  return [
    { type: "system", subtype: "init" },
    {
      type: "tool_call",
      call_id: "read-1",
      tool_call: {
        readToolCall: {
          args: { path: "/tmp/SKILL.md" },
          result: { success: {} },
        },
      },
    },
    {
      type: "tool_call",
      call_id: "shell-1",
      tool_call: {
        shellToolCall: {
          args: { command: `${runtime} retrieve --query-stdin --format json` },
        },
      },
    },
    {
      type: "tool_call",
      call_id: "shell-1",
      tool_call: {
        shellToolCall: {
          result: {
            success: {
              stdout: '{"sources":["proof.txt"],"verdict_age_milliseconds":0}',
            },
          },
        },
      },
    },
    {
      type: "assistant",
      message: { content: [{ type: "text", text: nonce }] },
    },
  ];
}

function traced(agent: "claude" | "cursor"): {
  runtimeTrace: string;
  traceCompletedBeforeModel: boolean;
} {
  return {
    runtimeTrace: jsonLines(evaluationTrace(agent)),
    traceCompletedBeforeModel: true,
  };
}

function evaluationTrace(agent: string): Record<string, unknown>[] {
  return [
    {
      agent,
      command: "hook",
      event: "started",
      exit_class: "started",
      pid: 7,
      timestamp_ms: 10,
    },
    {
      agent,
      command: "hook",
      event: "completed",
      exit_class: "success",
      pid: 7,
      timestamp_ms: 11,
    },
  ];
}

function structureHookResponse(
  event: Readonly<Record<string, unknown>>,
): Record<string, unknown> {
  return event.subtype === "hook_response"
    ? {
        ...event,
        output: {
          hookSpecificOutput: {
            additionalContext:
              '{"source":"proof.txt","verdict_age_milliseconds":0}',
          },
        },
        stdout: undefined,
      }
    : event;
}

function wrongHookId(
  event: Readonly<Record<string, unknown>>,
): Record<string, unknown> {
  return event.subtype === "hook_response"
    ? { ...event, hook_id: "wrong" }
    : event;
}

function wrongHookExitCode(
  event: Readonly<Record<string, unknown>>,
): Record<string, unknown> {
  return event.subtype === "hook_response"
    ? { ...event, exit_code: hookFailureExitCode }
    : event;
}

function expectClaudeMutationFails(
  input: Readonly<{
    events: readonly Readonly<Record<string, unknown>>[];
    expectation: Readonly<{
      cache: string;
      nonce: string;
      runtime: string;
      source: string;
      version: string;
    }>;
    observation: Readonly<{
      runtimeTrace: string;
      traceCompletedBeforeModel: boolean;
    }>;
    mutation: (
      event: Readonly<Record<string, unknown>>,
    ) => Readonly<Record<string, unknown>>;
  }>,
): void {
  const { events, expectation, observation, mutation } = input;
  expect(() =>
    parseAgentSession(
      "claude",
      jsonLines(
        events.map((event: Readonly<Record<string, unknown>>) =>
          mutation(event),
        ),
      ),
      expectation,
      observation,
    ),
  ).toThrow("claude hook response missing applicable context before model");
}
