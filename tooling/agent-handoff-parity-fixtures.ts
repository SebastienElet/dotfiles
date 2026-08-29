import type { Fixture } from "./agent-handoff-parity-support.ts";
import { join } from "node:path";
import { writeFileSync } from "node:fs";

type ParityCase = Readonly<{
  name: string;
  input: (fixture: Fixture) => Uint8Array;
  environment: (fixture: Fixture) => Readonly<Record<string, string>>;
  prepare?: (fixture: Fixture) => void;
}>;

type CaseOptions = Readonly<{
  environment?: ParityCase["environment"];
  prepare?: NonNullable<ParityCase["prepare"]>;
}>;

type EnvironmentOptions = Readonly<{
  claudeWindow?: false | string;
  home?: boolean;
  overrides?: Readonly<Record<string, string>>;
  xdgStateHome?: boolean | string;
}>;

type ClaudeUsageOptions = Readonly<{
  cacheCreationInputTokens?: number;
  cacheReadInputTokens?: number;
  sidechain?: boolean;
}>;

const textEncoder = new TextEncoder();
const defaultWindow = 100_000;
const belowThresholdUsage = 84_999;
const thresholdUsage = 85_000;
const highUsage = 90_000;
const lowUsage = 40_000;
const fractionalTokenCount = 1.5;
const invalidUtf8Byte = 0xff;
const belowRetainedWindowLineCount = 499;
const retainedWindowLineCount = 500;
const aboveRetainedWindowLineCount = 501;
const retainedLineCounts = [
  belowRetainedWindowLineCount,
  retainedWindowLineCount,
  aboveRetainedWindowLineCount,
] as const;

function bytes(value: string): Uint8Array {
  return textEncoder.encode(value);
}

function environmentFor(
  fixture: Fixture,
  options: EnvironmentOptions = {},
): Readonly<Record<string, string>> {
  const includeHome = options.home ?? true;
  const xdgStateHome = options.xdgStateHome ?? true;
  const claudeWindow = options.claudeWindow ?? String(defaultWindow);
  return {
    PATH: process.env.PATH ?? "",
    ...(includeHome ? { HOME: fixture.home } : {}),
    ...(xdgStateHome === false
      ? {}
      : {
          XDG_STATE_HOME:
            typeof xdgStateHome === "string"
              ? xdgStateHome
              : fixture.xdgStateHome,
        }),
    ...(claudeWindow === false
      ? {}
      : { CLAUDE_CODE_AUTO_COMPACT_WINDOW: claudeWindow }),
    ...options.overrides,
  };
}

function standardEnvironment(
  fixture: Fixture,
): Readonly<Record<string, string>> {
  return environmentFor(fixture);
}

function event(
  fixture: Fixture,
  values: Readonly<Record<string, unknown>> = {},
): Uint8Array {
  return bytes(
    JSON.stringify({
      hook_event_name: "Stop",
      session_id: "session",
      transcript_path: fixture.transcriptPath,
      ...values,
    }),
  );
}

function claudeUsage(
  inputTokens: number,
  options: ClaudeUsageOptions = {},
): string {
  return JSON.stringify({
    isSidechain: options.sidechain ?? false,
    message: {
      usage: {
        cache_creation_input_tokens: options.cacheCreationInputTokens ?? 0,
        cache_read_input_tokens: options.cacheReadInputTokens ?? 0,
        input_tokens: inputTokens,
      },
    },
    type: "assistant",
  });
}

function codexUsage(inputTokens: number, window: number): string {
  return JSON.stringify({
    payload: {
      info: {
        last_token_usage: { input_tokens: inputTokens },
        model_context_window: window,
      },
      type: "token_count",
    },
    type: "event_msg",
  });
}

function prepareTranscript(
  lines: readonly string[],
): NonNullable<ParityCase["prepare"]> {
  return (fixture) => {
    writeFileSync(fixture.transcriptPath, `${lines.join("\n")}\n`);
  };
}

function prepareClaude(used = highUsage): NonNullable<ParityCase["prepare"]> {
  return prepareTranscript([claudeUsage(used)]);
}

function parityCase(
  name: string,
  input: ParityCase["input"],
  options: CaseOptions = {},
): ParityCase {
  return {
    name,
    input,
    environment: options.environment ?? standardEnvironment,
    ...(options.prepare === undefined ? {} : { prepare: options.prepare }),
  };
}

function invalidSessionCase(name: string, sessionId?: string): ParityCase {
  const values =
    sessionId === undefined
      ? { session_id: undefined }
      : { session_id: sessionId };
  return parityCase(name, (fixture) => event(fixture, values));
}

function sentinelPath(fixture: Fixture, sessionId: string): string {
  return join(fixture.xdgStateHome, "dotfiles", "handoff", sessionId);
}

export {
  belowThresholdUsage,
  bytes,
  claudeUsage,
  codexUsage,
  defaultWindow,
  environmentFor,
  event,
  fractionalTokenCount,
  highUsage,
  invalidSessionCase,
  invalidUtf8Byte,
  lowUsage,
  parityCase,
  prepareClaude,
  prepareTranscript,
  retainedLineCounts,
  retainedWindowLineCount,
  sentinelPath,
  thresholdUsage,
};
export type { ClaudeUsageOptions, ParityCase };
