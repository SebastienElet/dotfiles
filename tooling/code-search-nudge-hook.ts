import {
  type HookToolCall,
  type NudgeState,
  nextNudgeTurn,
  parseStoredState,
} from "./code-search-nudge.ts";
import { dirname, join } from "node:path";
import { mkdirSync, readFileSync, writeFileSync } from "node:fs";

const safeSessionId = /^(?!\.{1,2}$)[A-Za-z0-9._-]+$/u;
const missingState = "";

type HookEnvironment = Readonly<Record<string, string | undefined>>;

type NudgeIo = Readonly<{
  read: (path: string) => string;
  write: (path: string, content: string) => void;
}>;

type NudgeOutcome = Readonly<{ stderr?: string; stdout?: string }>;

const fileSystemIo: NudgeIo = {
  read(path) {
    try {
      return readFileSync(path, "utf8");
    } catch {
      return missingState;
    }
  },
  write(path, content) {
    mkdirSync(dirname(path), { recursive: true });
    writeFileSync(path, content);
  },
};

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function statePath(environment: HookEnvironment, sessionId: string): string {
  const home = environment.HOME;
  if (home === undefined || home.length === 0) {
    throw new Error("missing HOME");
  }
  const stateHome = environment.XDG_STATE_HOME ?? join(home, ".local/state");
  return join(stateHome, "code-search-nudge", `${sessionId}.json`);
}

function parseHookEvent(
  input: string,
): Readonly<{ call: HookToolCall; sessionId: string }> {
  const value: unknown = JSON.parse(input);
  if (!isRecord(value)) {
    throw new Error("expected an object");
  }
  const sessionId = value.session_id;
  if (typeof sessionId !== "string" || !safeSessionId.test(sessionId)) {
    throw new Error("missing or unsafe session_id");
  }
  const toolName = value.tool_name;
  if (typeof toolName !== "string" || toolName.length === 0) {
    throw new Error("missing tool_name");
  }
  return {
    call: {
      toolInput: isRecord(value.tool_input) ? value.tool_input : {},
      toolName,
    },
    sessionId,
  };
}

function renderOutput(additionalContext: string): string {
  return JSON.stringify({
    hookSpecificOutput: { additionalContext, hookEventName: "PreToolUse" },
  });
}

function runCodeSearchNudge(
  input: string,
  environment: HookEnvironment,
  io: NudgeIo = fileSystemIo,
): NudgeOutcome {
  /* Advisory by construction: a hook sees tool calls, never the intent behind
     them, and ADR-039 forbids running ColGrep from a lifecycle hook. It may
     only remind, so every failure below degrades to silence rather than
     blocking a search, and says why on stderr. */
  try {
    const { call, sessionId } = parseHookEvent(input);
    const path = statePath(environment, sessionId);
    const state: NudgeState = parseStoredState(io.read(path));
    const turn = nextNudgeTurn(state, call);
    io.write(path, JSON.stringify(turn.state));
    return turn.additionalContext === undefined
      ? {}
      : { stdout: renderOutput(turn.additionalContext) };
  } catch (error) {
    const reason = error instanceof Error ? error.message : String(error);
    return { stderr: `code-search-nudge disabled for this call: ${reason}` };
  }
}

export {
  type NudgeIo,
  type NudgeOutcome,
  fileSystemIo,
  missingState,
  parseHookEvent,
  runCodeSearchNudge,
  statePath,
};
