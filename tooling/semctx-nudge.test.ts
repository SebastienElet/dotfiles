import { type SemctxNudgeIo, runSemctxNudge } from "./semctx-nudge.ts";
import { expect, test } from "bun:test";
import { z } from "zod";

const claude = ["--host", "claude"];
const codex = ["--host", "codex"];

function fakeIo(
  entries: readonly string[],
  directories: readonly string[],
): SemctxNudgeIo {
  return {
    exists: (path) => entries.includes(path) || directories.includes(path),
    isDirectory: (path) => directories.includes(path),
  };
}

function sessionStart(cwd: string): string {
  return JSON.stringify({
    cwd,
    hook_event_name: "SessionStart",
    session_id: "session-1",
    source: "startup",
  });
}

const hookOutput = z.object({
  hookSpecificOutput: z.object({
    additionalContext: z.string(),
    hookEventName: z.literal("SessionStart"),
  }),
});

function additionalContext(stdout: string | undefined): string {
  return hookOutput.parse(JSON.parse(stdout ?? "{}")).hookSpecificOutput
    .additionalContext;
}

test("reminds from a subdirectory of a semctx-enabled checkout", () => {
  const io = fakeIo(["/repo/.git"], ["/repo/.semctx"]);

  const outcome = runSemctxNudge(
    claude,
    sessionStart("/repo/packages/api"),
    io,
  );

  const context = additionalContext(outcome.stdout);
  expect(context).toContain("/repo");
  expect(context).toContain("mcp__plugin_semctx_semctx__semctx_verify_change");
  expect(outcome.stderr).toBeUndefined();
});

test("names Codex tools without the Claude ToolSearch query", () => {
  const io = fakeIo(["/repo/.git"], ["/repo/.semctx"]);

  const context = additionalContext(
    runSemctxNudge(codex, sessionStart("/repo"), io).stdout,
  );

  expect(context).toContain("mcp__semctx__");
  expect(context).toContain('repositoryRoot "/repo"');
  expect(context).not.toContain("ToolSearch");
  expect(context).not.toContain("mcp__plugin_semctx_semctx__");
});

test("refuses a missing or unknown host", () => {
  const io = fakeIo(["/repo/.git"], ["/repo/.semctx"]);

  for (const argv of [[], ["--host"], ["--host", "cursor"], ["claude"]]) {
    const outcome = runSemctxNudge(argv, sessionStart("/repo"), io);
    expect(outcome.stdout).toBeUndefined();
    expect(outcome.stderr).toContain("semctx-nudge disabled");
  }
});

test("stays silent in a checkout without semctx", () => {
  const io = fakeIo(["/repo/.git"], []);

  expect(runSemctxNudge(claude, sessionStart("/repo"), io)).toEqual({});
});

test("stays silent outside any Git checkout", () => {
  const io = fakeIo([], ["/tmp/.semctx"]);

  expect(runSemctxNudge(claude, sessionStart("/tmp/work"), io)).toEqual({});
});

test("ignores a semctx directory above the nearest Git root", () => {
  const io = fakeIo(["/parent/child/.git"], ["/parent/.semctx"]);

  expect(runSemctxNudge(claude, sessionStart("/parent/child"), io)).toEqual({});
});

test("ignores a .semctx entry that is not a directory", () => {
  const io = fakeIo(["/repo/.git", "/repo/.semctx"], []);

  expect(runSemctxNudge(claude, sessionStart("/repo"), io)).toEqual({});
});

test("degrades to a stderr line on an unusable payload", () => {
  const io = fakeIo(["/repo/.git"], ["/repo/.semctx"]);

  for (const input of ["not json", "{}", JSON.stringify({ cwd: "relative" })]) {
    const outcome = runSemctxNudge(claude, input, io);
    expect(outcome.stdout).toBeUndefined();
    expect(outcome.stderr).toContain("semctx-nudge disabled");
  }
});
