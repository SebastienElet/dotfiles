import { expect, test } from "bun:test";
import { mkdtemp, realpath, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";

import {
  classifyAgentFailure,
  conditionedAgentFailure,
} from "./agent-memory-eval-diagnostics.ts";
import { runTraceObservedProcess } from "./agent-memory-eval-process.ts";

test("classifies only allowlisted fields from the last agent error event", () => {
  const secret = "private prompt contents";
  const stream = [
    JSON.stringify({ type: "system", subtype: "init", message: "ignored auth text" }),
    JSON.stringify({
      type: "result",
      subtype: "error_during_execution",
      is_error: true,
      message: "Model provider unavailable",
      prompt: secret,
    }),
  ].join("\n");
  const classification = classifyAgentFailure("claude", stream, "");
  expect(classification).toBe("model_unavailable");
  expect(classification).not.toContain(secret);
  expect(
    classifyAgentFailure(
      "claude",
      JSON.stringify({ type: "assistant", message: secret }),
      "",
    ),
  ).toBe("empty_stderr");
});

test("binds an agent failure class to its condition", () => {
  const error = conditionedAgentFailure(
    "claude",
    "proposal",
    JSON.stringify({ type: "result", message: "Model provider unavailable" }),
    "",
  );
  expect(error.message).toBe("claude:proposal:model_unavailable");
});

test("runs an observed agent process in the fixture repository", async () => {
  const repository = await mkdtemp(join(tmpdir(), "agent-memory-process-test-"));
  try {
    const output = await runTraceObservedProcess(
      ["pwd"],
      {},
      join(repository, "trace.jsonl"),
      "claude",
      "control",
      undefined,
      10_000,
      undefined,
      repository,
    );
    expect(output.stdout.trim()).toBe(await realpath(repository));
  } finally {
    await rm(repository, { force: true, recursive: true });
  }
});

test("rejects an agent error event even when the process exits zero", async () => {
  const event = JSON.stringify({
    is_error: true,
    message: "Model provider unavailable",
    subtype: "error_during_execution",
    type: "result",
  });
  await expect(
    runTraceObservedProcess(
      ["printf", `${event}\n`],
      {},
      "/tmp/agent-memory-unused-trace",
      "claude",
      "relevant",
      undefined,
      10_000,
    ),
  ).rejects.toThrow("claude:relevant:model_unavailable");
});
