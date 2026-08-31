import {
  type NudgeIo,
  missingState,
  parseHookEvent,
  runCodeSearchNudge,
  statePath,
} from "./code-search-nudge-hook.ts";
import { expect, test } from "bun:test";
import { searchThreshold } from "./code-search-nudge.ts";

const environment = { HOME: "/home/agent", XDG_STATE_HOME: "/state" };
const belowThreshold = searchThreshold - 1;

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function hookOutputField(content: string, field: string): string {
  const parsed: unknown = JSON.parse(content);
  if (!isRecord(parsed)) {
    return "";
  }
  const output = parsed.hookSpecificOutput;
  if (!isRecord(output)) {
    return "";
  }
  const value = output[field];
  return typeof value === "string" ? value : "";
}

function memoryIo(): NudgeIo & { readonly files: Map<string, string> } {
  const files = new Map<string, string>();
  return {
    files,
    read: (path) => files.get(path) ?? missingState,
    write: (path, content) => {
      files.set(path, content);
    },
  };
}

function grepEvent(sessionId = "session-1"): string {
  return JSON.stringify({
    session_id: sessionId,
    tool_input: { pattern: "whatever" },
    tool_name: "Grep",
  });
}

function searchUntilNudged(
  io: NudgeIo,
  times: number,
  sessionId?: string,
): string {
  let stdout = "";
  for (let index = 0; index < times; index += 1) {
    stdout =
      runCodeSearchNudge(grepEvent(sessionId), environment, io).stdout ?? "";
  }
  return stdout;
}

test("emits the PreToolUse contract Claude Code understands", () => {
  const stdout = searchUntilNudged(memoryIo(), searchThreshold);

  expect(hookOutputField(stdout, "hookEventName")).toBe("PreToolUse");
  expect(hookOutputField(stdout, "additionalContext")).toContain("code-search");
});

test("persists the count across hook processes", () => {
  const io = memoryIo();
  searchUntilNudged(io, belowThreshold);

  expect(io.files.size).toBe(1);
  expect(searchUntilNudged(io, 1)).not.toBe("");
});

test("keeps sessions independent", () => {
  const io = memoryIo();
  searchUntilNudged(io, belowThreshold, "session-1");

  expect(searchUntilNudged(io, 1, "session-2")).toBe("");
});

test("refuses a session_id that would escape the state directory", () => {
  for (const sessionId of ["../escape", "..", "a/b", ""]) {
    expect(() =>
      parseHookEvent(
        JSON.stringify({
          session_id: sessionId,
          tool_input: {},
          tool_name: "Grep",
        }),
      ),
    ).toThrow();
  }
});

test("never blocks a search when its own input is unusable", () => {
  const io = memoryIo();
  for (const input of ["", "not json", "[]", '{"tool_name":"Grep"}']) {
    const result = runCodeSearchNudge(input, environment, io);

    expect(result.stdout).toBeUndefined();
    expect(result.stderr).toContain("code-search-nudge disabled");
  }
});

test("never blocks a search when the state directory cannot be written", () => {
  const failing: NudgeIo = {
    read: () => missingState,
    write: () => {
      throw new Error("read-only file system");
    },
  };
  const result = runCodeSearchNudge(grepEvent(), environment, failing);

  expect(result.stdout).toBeUndefined();
  expect(result.stderr).toContain("read-only file system");
});

test("falls back to the XDG default when XDG_STATE_HOME is unset", () => {
  expect(statePath({ HOME: "/home/agent" }, "session-1")).toBe(
    "/home/agent/.local/state/code-search-nudge/session-1.json",
  );
});

test("reports a missing HOME instead of writing to a guessed path", () => {
  expect(() => statePath({}, "session-1")).toThrow("missing HOME");
});
