import { afterEach, describe, expect, test } from "bun:test";
import { mkdtemp, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";

import {
  parseAgentSession,
  parseAgentText,
  parseEvaluationTrace,
} from "./agent-memory-eval-contract.ts";
import { normalizeAgentVersion } from "./agent-memory-eval-claude.ts";
import {
  loadEvaluationScenarios,
  scenarioById,
} from "./agent-memory-eval-scenario.ts";
const roots: string[] = [];

afterEach(async () => {
  await Promise.all(roots.splice(0).map((root) => rm(root, { force: true, recursive: true })));
});

describe("agent memory evaluation contract", () => {
  test("normalizes the Claude CLI label to the JSONL version", () => {
    expect(normalizeAgentVersion("claude", "2.1.251 (Claude Code)\n")).toBe("2.1.251");
    expect(normalizeAgentVersion("codex", "codex-cli 0.151.0\n")).toBe("codex-cli 0.151.0");
  });

  test("loads and selects versioned scenarios", async () => {
    const root = await fixtureRoot();
    const path = join(root, "scenarios.json");
    await writeFile(path, JSON.stringify(contract("contract prompt")));
    const loaded = await loadEvaluationScenarios(path);
    expect(scenarioById(loaded, "retrieve").prompt).toBe("contract prompt");
    expect(() => scenarioById(loaded, "missing")).toThrow();
    await rm(path);
    await expect(loadEvaluationScenarios(path)).rejects.toThrow();
  });

  test("rejects malformed scenario contracts", async () => {
    const root = await fixtureRoot();
    const path = join(root, "scenarios.json");
    for (const value of [
      { version: 2, scenarios: [] },
      { version: 1, scenarios: [] },
      { version: 1, scenarios: [{ id: "retrieve", prompt: "query", capabilities: [] }] },
      { version: 1, scenarios: [{ id: "retrieve", prompt: "", capabilities: ["fresh"] }] },
    ]) {
      await writeFile(path, JSON.stringify(value));
      await expect(loadEvaluationScenarios(path)).rejects.toThrow();
    }
  });

  test("strictly parses Codex cache completion before model influence", () => {
    const expectation = expected("nonce-codex", "0.151.0", "/tmp/runtime");
    const stream = jsonLines([
      { type: "thread.started", thread_id: "fixture" },
      { type: "item.completed", item: { id: "1", type: "agent_message", text: expectation.nonce } },
    ]);
    const observation = {
      cacheAbsentBefore: true,
      cacheCompletedBeforeModel: true,
      cachePath: expectation.cache,
      runtimeTrace: jsonLines(evaluationTrace("codex")),
      traceCompletedBeforeModel: true,
      version: expectation.version,
    };
    expect(parseAgentSession("codex", stream, expectation, observation).adapterCompletedBeforeModel).toBe(true);
    for (const mutant of [
      { ...observation, cacheAbsentBefore: false },
      { ...observation, cacheCompletedBeforeModel: false },
      { ...observation, cachePath: "/wrong" },
    ]) {
      expect(() => parseAgentSession("codex", stream, expectation, mutant)).toThrow();
    }
    expect(() => parseAgentSession("codex", "not-json", expectation, observation)).toThrow();
    expect(() => parseAgentSession("codex", stream, expectation, { ...observation, version: "wrong" })).toThrow();
  });

  test("matches Claude hook response before assistant nonce", () => {
    const expectation = expected("nonce-claude", "2.1.251", "/tmp/runtime");
    const events = claudeEvents(expectation.nonce, expectation.version);
    const observation = traced("claude");
    expect(parseAgentSession("claude", jsonLines(events), expectation, observation).contextBeforeModel).toBe(true);
    const structuredOutput = events.map((event) =>
      event.subtype === "hook_response"
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
        : event,
    );
    expect(
      parseAgentSession("claude", jsonLines(structuredOutput), expectation, observation)
        .contextBeforeModel,
    ).toBe(true);
    for (const mutation of [
      (event: Record<string, unknown>) => event.subtype === "hook_response" ? { ...event, hook_id: "wrong" } : event,
      (event: Record<string, unknown>) => event.subtype === "hook_response" ? { ...event, exit_code: 7 } : event,
    ]) {
      expect(() =>
        parseAgentSession("claude", jsonLines(events.map(mutation)), expectation, observation),
      ).toThrow("claude hook response missing applicable context before model");
    }
    expect(() => parseAgentSession("claude", jsonLines([events[0], events[1], events[3], events[2]]), expectation, observation)).toThrow();
  });

  test("matches Cursor skill and shell completion by call ID", () => {
    const expectation = expected("nonce-cursor", "2026.08.25", "/tmp/runtime");
    const events = cursorEvents(expectation.nonce, expectation.runtime);
    const observation = { ...traced("cursor"), version: expectation.version };
    expect(parseAgentSession("cursor", jsonLines(events), expectation, observation).contextBeforeModel).toBe(true);
    const mutant = events.map((event, index) => index === 3 ? { ...event, call_id: "wrong" } : event);
    expect(() => parseAgentSession("cursor", jsonLines(mutant), expectation, observation)).toThrow();
  });

  test("rejects malformed output rather than dropping it", () => {
    const event = { type: "item.completed", item: { id: "1", type: "agent_message", text: "response" } };
    expect(parseAgentText("codex", JSON.stringify(event), "0.151.0").modelText).toBe("response");
    expect(() => parseAgentText("codex", `bad\n${JSON.stringify(event)}`, "0.151.0")).toThrow();
    expect(() => parseAgentText("codex", JSON.stringify(event))).toThrow();
  });

  test("requires a minimal ordered and redacted runtime trace", () => {
    const trace = evaluationTrace("codex");
    expect(parseEvaluationTrace("codex", jsonLines(trace))).toBe(true);
    for (const mutant of [
      [],
      [trace[1], trace[0]],
      [{ ...trace[0], agent: "claude" }, trace[1]],
      [trace[0], { ...trace[1], pid: 8 }],
      [trace[0], { ...trace[1], timestamp_ms: 9 }],
      [{ ...trace[0], query: "private nonce" }, trace[1]],
      [trace[0], { ...trace[1], exit_class: "rejection" }],
    ]) {
      expect(() => parseEvaluationTrace("codex", jsonLines(mutant))).toThrow();
    }
    expect(() =>
      parseEvaluationTrace(
        "codex",
        jsonLines([
          trace[0],
          { ...trace[1], event: "error", exit_class: "unavailable" },
        ]),
      ),
    ).toThrow("event=error exit_class=unavailable");
    expect(
      parseEvaluationTrace(
        "codex",
        jsonLines([
          trace[0],
          { ...trace[1], event: "error", exit_class: "unavailable" },
        ]),
        "unavailable",
      ),
    ).toBe(true);
  });

  test("requires runtime trace completion before model influence", () => {
    const expectation = expected("nonce-codex", "0.151.0", "/tmp/runtime");
    const stream = jsonLines([
      { type: "item.completed", item: { id: "1", type: "agent_message", text: expectation.nonce } },
    ]);
    const valid = {
      cacheAbsentBefore: true,
      cacheCompletedBeforeModel: true,
      cachePath: expectation.cache,
      runtimeTrace: jsonLines(evaluationTrace("codex")),
      traceCompletedBeforeModel: true,
      version: expectation.version,
    };
    expect(parseAgentSession("codex", stream, expectation, valid).adapterCompletedBeforeModel).toBe(true);
    expect(() => parseAgentSession("codex", stream, expectation, { ...valid, runtimeTrace: "" })).toThrow();
    expect(() =>
      parseAgentSession("codex", stream, expectation, {
        ...valid,
        traceCompletedBeforeModel: false,
      }),
    ).toThrow("codex runtime trace completed after model");
  });
});

function contract(prompt: string): unknown {
  return { version: 1, scenarios: [{ id: "retrieve", prompt, capabilities: ["fresh"] }] };
}

function expected(nonce: string, version: string, runtime: string) {
  return { cache: "/tmp/store/oracle-cache.json", nonce, runtime, source: "proof.txt", version };
}

function jsonLines(events: readonly unknown[]): string {
  return events.map((event) => JSON.stringify(event)).join("\n");
}

function claudeEvents(nonce: string, version: string): Record<string, unknown>[] {
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
    { type: "assistant", message: { content: [{ type: "text", text: nonce }] } },
  ];
}

function cursorEvents(nonce: string, runtime: string): Record<string, unknown>[] {
  return [
    { type: "system", subtype: "init" },
    { type: "tool_call", call_id: "read-1", tool_call: { readToolCall: { args: { path: "/tmp/SKILL.md" }, result: { success: {} } } } },
    { type: "tool_call", call_id: "shell-1", tool_call: { shellToolCall: { args: { command: `${runtime} retrieve --query-stdin --format json` } } } },
    { type: "tool_call", call_id: "shell-1", tool_call: { shellToolCall: { result: { success: { stdout: '{"sources":["proof.txt"],"verdict_age_milliseconds":0}' } } } } },
    { type: "assistant", message: { content: [{ type: "text", text: nonce }] } },
  ];
}

function evaluationTrace(agent: string): Record<string, unknown>[] {
  return [
    { agent, command: "hook", event: "started", exit_class: "started", pid: 7, timestamp_ms: 10 },
    { agent, command: "hook", event: "completed", exit_class: "success", pid: 7, timestamp_ms: 11 },
  ];
}

function traced(agent: "codex" | "claude" | "cursor") {
  return {
    runtimeTrace: jsonLines(evaluationTrace(agent)),
    traceCompletedBeforeModel: true,
  };
}

async function fixtureRoot(): Promise<string> {
  const root = await mkdtemp(join(tmpdir(), "agent-memory-contract-test-"));
  roots.push(root);
  return root;
}
