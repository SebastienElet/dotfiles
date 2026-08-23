import { afterEach, beforeEach, describe, expect, test } from "bun:test";
import {
  chmodSync,
  mkdirSync,
  mkdtempSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import {
  claudeEvent,
  claudeUsage,
  codexUsage,
  type HookResult,
  runEntryPoint,
} from "./agent-handoff-test-support.ts";

let testRoot: string;

beforeEach(() => {
  testRoot = mkdtempSync(join(tmpdir(), "agent-handoff-failure-"));
});

afterEach(() => {
  rmSync(testRoot, { force: true, recursive: true });
});

function writeTranscript(name: string, records: readonly string[]): string {
  const path = join(testRoot, name);
  writeFileSync(path, `${records.join("\n")}\n`);
  return path;
}

async function runHook(
  input: string,
  environment: Readonly<Record<string, string>> = {},
): Promise<HookResult> {
  return runEntryPoint(testRoot, input, environment);
}

describe("agent-handoff failures", () => {
  test.each([
    ["malformed event", "not-json", "invalid hook event"],
    [
      "missing session",
      JSON.stringify({ hook_event_name: "Stop", transcript_path: "/tmp/x" }),
      "missing session_id",
    ],
    [
      "missing transcript",
      JSON.stringify({ hook_event_name: "Stop", session_id: "x" }),
      "missing transcript_path",
    ],
    [
      "unsafe session",
      JSON.stringify({
        hook_event_name: "Stop",
        session_id: "../escape",
        transcript_path: "/tmp/x",
      }),
      "invalid session_id",
    ],
    [
      "missing event name",
      JSON.stringify({ session_id: "x", transcript_path: "/tmp/x" }),
      "missing Stop event",
    ],
    [
      "another Claude event",
      JSON.stringify({
        hook_event_name: "UserPromptSubmit",
        session_id: "x",
        transcript_path: "/tmp/x",
      }),
      "unsupported hook event",
    ],
    [
      "another Codex event",
      JSON.stringify({
        event: "UserPromptSubmit",
        session_id: "x",
        transcript_path: "/tmp/x",
      }),
      "unsupported hook event",
    ],
    [
      "conflicting event names",
      JSON.stringify({
        event: "Stop",
        hook_event_name: "UserPromptSubmit",
        session_id: "x",
        transcript_path: "/tmp/x",
      }),
      "unsupported hook event",
    ],
  ])("fails visibly for %s", async (_name, input, diagnostic) => {
    const result = await runHook(input);

    expect(result.exitCode).toBe(1);
    expect(result.stdout).toBe("");
    expect(result.stderr).toContain(diagnostic);
  });

  test.each([
    [
      "unreadable transcript",
      "/missing/transcript",
      {},
      "cannot read transcript",
    ],
    [
      "malformed transcript",
      "malformed.jsonl",
      { records: [claudeUsage(90_000), "not-json"] },
      "malformed transcript JSON",
    ],
    [
      "unsupported agent",
      "unsupported.jsonl",
      { records: [JSON.stringify({ type: "other" })] },
      "no supported usage record",
    ],
    [
      "invalid threshold",
      "threshold.jsonl",
      {
        environment: { HANDOFF_TOKEN_THRESHOLD: "85k" },
        records: [claudeUsage(90_000)],
      },
      "invalid HANDOFF_TOKEN_THRESHOLD",
    ],
    [
      "missing Claude window",
      "window.jsonl",
      {
        environment: { CLAUDE_CODE_AUTO_COMPACT_WINDOW: "" },
        records: [claudeUsage(90_000)],
      },
      "missing context window",
    ],
    [
      "invalid Claude sidechain marker",
      "sidechain-marker.jsonl",
      {
        records: [
          JSON.stringify({
            type: "assistant",
            isSidechain: "true",
            message: { usage: { input_tokens: 90_000 } },
          }),
        ],
      },
      "invalid Claude isSidechain",
    ],
    [
      "zero Codex context window",
      "zero-window.jsonl",
      { records: [codexUsage(90_000, 0)] },
      "invalid Codex model_context_window",
    ],
  ])("fails visibly for %s", async (_name, filename, setup, diagnostic) => {
    const records = "records" in setup ? setup.records : undefined;
    const transcript =
      records === undefined ? filename : writeTranscript(filename, records);
    const environment = "environment" in setup ? setup.environment : {};
    const result = await runHook(
      claudeEvent(transcript, "failure"),
      environment,
    );

    expect(result.exitCode).toBe(1);
    expect(result.stdout).toBe("");
    expect(result.stderr).toContain(diagnostic);
  });

  test("reports sentinel write failure without blocking", async () => {
    const transcript = writeTranscript("write-failure.jsonl", [
      claudeUsage(90_000),
    ]);
    const stateDirectory = join(testRoot, "read-only-state");
    mkdirSync(stateDirectory);
    chmodSync(stateDirectory, 0o500);
    const result = await runHook(claudeEvent(transcript, "write-failure"), {
      XDG_STATE_HOME: stateDirectory,
    });

    expect(result.exitCode).toBe(3);
    expect(result.stdout).toBe("");
    expect(result.stderr).toContain("cannot create handoff sentinel");
  });

  test("does not inspect usage before the latest 500 physical lines", async () => {
    const transcript = writeTranscript("physical-window.jsonl", [
      claudeUsage(90_000),
      ...Array.from({ length: 500 }, () => ""),
    ]);
    const input = claudeEvent(transcript, "physical-window");

    expect(await runHook(input)).toEqual({
      exitCode: 1,
      stderr: "agent-handoff: no supported usage record in transcript\n",
      stdout: "",
    });

    writeFileSync(transcript, `${claudeUsage(90_000)}\n`);
    expect((await runHook(input)).stdout).not.toBe("");
  });
});
