import { describe, expect, test } from "bun:test";

import {
  memoryCommandErrorCode,
  memoryCommandObserved,
  memoryCommandOutput,
} from "./agent-memory-eval-action.ts";

describe("agent memory action evidence", () => {
  const runtime = "/private/tmp/evaluation/home/.local/bin/agent-memory";

  test("correlates a completed Codex memory command", () => {
    const stream = lines([
      {
        type: "item.completed",
        item: {
          aggregated_output: '{"status":"stored"}',
          command: `${runtime} admit --format json`,
          exit_code: 0,
          id: "command-1",
          status: "completed",
          type: "command_execution",
        },
      },
    ]);
    expect(memoryCommandOutput("codex", stream, runtime, "admit")).toContain('"stored"');
  });

  test("accepts only the exact Codex shell envelope", () => {
    const command = `${runtime} admit --format json`;
    const stream = lines([
      {
        type: "item.completed",
        item: {
          aggregated_output: '{"status":"stored"}',
          command: `/bin/zsh -lc "${command}"`,
          exit_code: 0,
          status: "completed",
          type: "command_execution",
        },
      },
    ]);
    expect(memoryCommandOutput("codex", stream, runtime, "admit")).toContain('"stored"');
  });

  test("correlates Claude Bash tool use and result", () => {
    const stream = lines([
      {
        type: "assistant",
        message: {
          content: [
            {
              id: "tool-1",
              input: { command: `${runtime} admit --format json` },
              name: "Bash",
              type: "tool_use",
            },
          ],
        },
      },
      {
        type: "user",
        message: {
          content: [
            { content: '{"status":"stored"}', tool_use_id: "tool-1", type: "tool_result" },
          ],
        },
      },
    ]);
    expect(memoryCommandOutput("claude", stream, runtime, "admit")).toContain('"stored"');
  });

  test("correlates a completed Cursor shell call", () => {
    const stream = lines([
      {
        call_id: "shell-1",
        tool_call: {
          shellToolCall: { args: { command: `${runtime} admit --format json` } },
        },
        type: "tool_call",
      },
      {
        call_id: "shell-1",
        tool_call: {
          shellToolCall: { result: { success: { stdout: '{"status":"stored"}' } } },
        },
        type: "tool_call",
      },
    ]);
    expect(memoryCommandOutput("cursor", stream, runtime, "admit")).toContain('"stored"');
  });

  test("rejects forged, compound, and different runtime commands", () => {
    for (const command of [
      "echo agent-memory admit --format json",
      `echo ${runtime} admit --format json`,
      `${runtime} admit --format json; echo stored`,
      "/private/tmp/other/agent-memory admit --format json",
    ]) {
      const stream = lines([
        {
          type: "item.completed",
          item: {
            aggregated_output: '{"status":"stored"}',
            command,
            exit_code: 0,
            status: "completed",
            type: "command_execution",
          },
        },
      ]);
      expect(() => memoryCommandOutput("codex", stream, runtime, "admit")).toThrow();
    }
  });

  test("rejects a missing, failed, or different command", () => {
    for (const stream of [
      lines([]),
      lines([
        {
          type: "item.completed",
          item: {
            command: "agent-memory retrieve --query-stdin --format json",
            exit_code: 0,
            status: "completed",
            type: "command_execution",
          },
        },
      ]),
      lines([
        {
          type: "item.completed",
          item: {
            aggregated_output: '{"status":"stored"}',
            command: "agent-memory admit --format json",
            exit_code: 2,
            status: "failed",
            type: "command_execution",
          },
        },
      ]),
    ]) {
      expect(() => memoryCommandOutput("codex", stream, runtime, "admit")).toThrow();
    }
    expect(
      memoryCommandObserved(
        "codex",
        lines([
          {
            type: "item.completed",
            item: {
              command: `${runtime} admit --format json`,
              exit_code: 2,
              status: "failed",
              type: "command_execution",
            },
          },
        ]),
        runtime,
        "admit",
      ),
    ).toBe(true);
  });

  test("extracts only a typed runtime error code", () => {
    const stream = lines([
      {
        type: "item.completed",
        item: {
          aggregated_output: '{"error":{"code":"source_unavailable","field":"proof"}}',
          type: "command_execution",
        },
      },
    ]);
    expect(memoryCommandErrorCode(stream)).toBe("source_unavailable");
    expect(memoryCommandErrorCode("secret raw output")).toBeUndefined();
  });
});

function lines(events: readonly unknown[]): string {
  return events.map((event) => JSON.stringify(event)).join("\n");
}
