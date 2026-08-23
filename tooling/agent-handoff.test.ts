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
  claudeUsage,
  codexUsage,
  event,
  type HookResult,
  runEntryPoint,
} from "./agent-handoff-test-support.ts";

let testRoot: string;

beforeEach(() => {
  testRoot = mkdtempSync(join(tmpdir(), "agent-handoff-"));
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

describe("agent-handoff entry point", () => {
  test("keeps Claude below the boundary and blocks at the boundary", async () => {
    const below = writeTranscript("claude-below.jsonl", [claudeUsage(84_999)]);
    const boundary = writeTranscript("claude-boundary.jsonl", [
      claudeUsage(85_000),
    ]);

    expect(await runHook(event(below, "claude-below"))).toEqual({
      exitCode: 0,
      stderr: "",
      stdout: "",
    });

    const result = await runHook(event(boundary, "claude-boundary"));
    expect(result.exitCode).toBe(0);
    expect(result.stderr).toBe("");
    expect(JSON.parse(result.stdout)).toEqual({
      decision: "block",
      reason:
        "Context is at 85k tokens, past the 85k handoff threshold. Start no new work. Use /handoff to emit the resume prompt for a fresh session, then stop.",
    });
  });

  test("uses the Codex window and invocation", async () => {
    const transcript = writeTranscript("codex.jsonl", [codexUsage(90_000)]);
    const result = await runHook(event(transcript, "codex"));

    expect(result.exitCode).toBe(0);
    expect(result.stderr).toBe("");
    expect(JSON.parse(result.stdout).reason).toContain("Use $handoff");
  });

  test("uses the configured threshold at its exact boundary", async () => {
    const transcript = writeTranscript("configured-threshold.jsonl", [
      claudeUsage(50_000),
    ]);
    const result = await runHook(event(transcript, "configured-threshold"), {
      HANDOFF_TOKEN_THRESHOLD: "50000",
    });

    expect(result.exitCode).toBe(0);
    expect(result.stderr).toBe("");
    expect(JSON.parse(result.stdout).reason).toContain(
      "past the 50k handoff threshold",
    );
  });

  test("ignores sidechains and uses the latest main-chain usage", async () => {
    const transcript = writeTranscript("sidechain.jsonl", [
      claudeUsage(40_000),
      claudeUsage(90_000, true),
    ]);

    expect(await runHook(event(transcript, "sidechain"))).toEqual({
      exitCode: 0,
      stderr: "",
      stdout: "",
    });
  });

  test("retains only the latest 500 transcript lines", async () => {
    const unrelated = Array.from({ length: 499 }, () =>
      JSON.stringify({ type: "other" }),
    );
    const transcript = writeTranscript("retention.jsonl", [
      claudeUsage(90_000),
      ...unrelated,
      claudeUsage(40_000),
    ]);

    expect(await runHook(event(transcript, "retention"))).toEqual({
      exitCode: 0,
      stderr: "",
      stdout: "",
    });
  });

  test("does not recurse when the stop hook is already active", async () => {
    expect(await runHook(event("/missing/transcript", "active", true))).toEqual(
      {
        exitCode: 0,
        stderr: "",
        stdout: "",
      },
    );
  });

  test("creates at most one handoff for repeated and concurrent events", async () => {
    const transcript = writeTranscript("repeated.jsonl", [claudeUsage(90_000)]);
    const input = event(transcript, "same-session");
    const results = await Promise.all([
      runHook(input),
      runHook(input),
      runHook(input),
    ]);

    expect(results.filter((result) => result.stdout !== "")).toHaveLength(1);
    expect(
      results.every((result) => result.exitCode === 0 && result.stderr === ""),
    ).toBe(true);
    expect(await runHook(input)).toEqual({
      exitCode: 0,
      stderr: "",
      stdout: "",
    });
  });

  test.each([
    ["malformed event", "not-json", "invalid hook event"],
    [
      "missing session",
      JSON.stringify({ transcript_path: "/tmp/x" }),
      "missing session_id",
    ],
    [
      "missing transcript",
      JSON.stringify({ session_id: "x" }),
      "missing transcript_path",
    ],
    [
      "unsafe session",
      JSON.stringify({ session_id: "../escape", transcript_path: "/tmp/x" }),
      "invalid session_id",
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
  ])("fails visibly for %s", async (_name, filename, setup, diagnostic) => {
    const records = "records" in setup ? setup.records : undefined;
    const transcript =
      records === undefined ? filename : writeTranscript(filename, records);
    const environment = "environment" in setup ? setup.environment : {};
    const result = await runHook(event(transcript, "failure"), environment);

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
    const result = await runHook(event(transcript, "write-failure"), {
      XDG_STATE_HOME: stateDirectory,
    });

    expect(result.exitCode).toBe(3);
    expect(result.stdout).toBe("");
    expect(result.stderr).toContain("cannot create handoff sentinel");
  });
});
