import {
  type HookResult,
  claudeEvent,
  claudeUsage,
  codexUsage,
  runEntryPoint,
} from "./agent-handoff-test-support.ts";
import { afterEach, beforeEach, expect, test } from "bun:test";
import {
  chmodSync,
  mkdirSync,
  mkdtempSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { join } from "node:path";
import { tmpdir } from "node:os";

let testRoot = "";

const highUsage = 90_000;
const restrictedDirectoryMode = 0o500;
const unexpectedFailureExitCode = 3;

type FailureSetup = Readonly<{
  environment?: Readonly<Record<string, string>>;
  records?: readonly string[];
}>;

beforeEach(() => {
  testRoot = mkdtempSync(join(tmpdir(), "agent-handoff-failure-"));
});

afterEach(() => {
  if (testRoot !== "") {
    rmSync(testRoot, { force: true, recursive: true });
    testRoot = "";
  }
});

function writeTranscript(name: string, records: readonly string[]): string {
  const path = join(testRoot, name);
  writeFileSync(path, `${records.join("\n")}\n`);
  return path;
}

function runHook(
  input: string,
  environment: Readonly<Record<string, string>> = {},
): Promise<HookResult> {
  return runEntryPoint(testRoot, input, environment);
}

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
    { records: [claudeUsage(highUsage), "not-json"] },
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
      records: [claudeUsage(highUsage)],
    },
    "invalid HANDOFF_TOKEN_THRESHOLD",
  ],
  [
    "missing Claude window",
    "window.jsonl",
    {
      environment: { CLAUDE_CODE_AUTO_COMPACT_WINDOW: "" },
      records: [claudeUsage(highUsage)],
    },
    "missing context window",
  ],
  [
    "invalid Claude sidechain marker",
    "sidechain-marker.jsonl",
    {
      records: [
        JSON.stringify({
          isSidechain: "true",
          message: { usage: { input_tokens: highUsage } },
          type: "assistant",
        }),
      ],
    },
    "invalid Claude isSidechain",
  ],
  [
    "zero Codex context window",
    "zero-window.jsonl",
    { records: [codexUsage(highUsage, 0)] },
    "invalid Codex model_context_window",
  ],
])(
  "fails visibly for %s",
  async (
    ...[_name, filename, setup, diagnostic]: readonly [
      string,
      string,
      FailureSetup,
      string,
    ]
  ) => {
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
  },
);

test("reports sentinel write failure without blocking", async () => {
  const transcript = writeTranscript("write-failure.jsonl", [
    claudeUsage(highUsage),
  ]);
  const stateDirectory = join(testRoot, "read-only-state");
  mkdirSync(stateDirectory);
  chmodSync(stateDirectory, restrictedDirectoryMode);
  const result = await runHook(claudeEvent(transcript, "write-failure"), {
    XDG_STATE_HOME: stateDirectory,
  });

  expect(result.exitCode).toBe(unexpectedFailureExitCode);
  expect(result.stdout).toBe("");
  expect(result.stderr).toContain("cannot create handoff sentinel");
});

test("retains exactly the latest 500 physical lines", async () => {
  const boundary = writeTranscript("physical-window-boundary.jsonl", [
    claudeUsage(highUsage),
    ...Array.from({ length: 499 }, () => ""),
  ]);
  const boundaryResult = await runHook(
    claudeEvent(boundary, "physical-window-boundary"),
  );
  expect(boundaryResult.stdout).not.toBe("");

  const outside = writeTranscript("physical-window-outside.jsonl", [
    claudeUsage(highUsage),
    ...Array.from({ length: 500 }, () => ""),
  ]);
  const input = claudeEvent(outside, "physical-window-outside");

  expect(await runHook(input)).toEqual({
    exitCode: 1,
    stderr: "agent-handoff: no supported usage record in transcript\n",
    stdout: "",
  });

  writeFileSync(outside, `${claudeUsage(highUsage)}\n`);
  const replay = await runHook(input);
  expect(replay.stdout).not.toBe("");
});
