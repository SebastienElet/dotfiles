import { afterEach, expect, test } from "bun:test";
import { mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import type { SeriesOptions } from "./runner.ts";
import { at } from "./test-support.ts";
import { capture } from "./process.ts";
import { codexExecutor } from "./live.ts";
import { join } from "node:path";
import { loadCases } from "./sources.ts";
import { prepareFixture } from "./fixture.ts";
import { tmpdir } from "node:os";

const seriesOptions: SeriesOptions = {
  agent: "codex",
  agentVersion: "synthetic",
  model: "explicit-model",
  only: ["code-search-structural"],
  runs: 1,
  controls: {
    sandbox: "workspace-write",
    network: false,
    tools: "shell-with-synthetic-cat-rg-fd-colgrep-v1",
    timeoutSeconds: 10,
    reasoningEffort: "low",
    tokenBudget: null,
  },
};

const roots: string[] = [];
afterEach(() => {
  for (const path of roots.splice(0)) {
    rmSync(path, { recursive: true, force: true });
  }
});
function script(content: string): string {
  const root = mkdtempSync(join(tmpdir(), "harness-process-test-"));
  roots.push(root);
  const path = join(root, "synthetic-provider");
  writeFileSync(path, `#!${process.execPath}\n${content}`, { mode: 0o755 });
  return path;
}

test("process adapter forwards exact UTF-8 stdin and contains timeout, exit, and output overflow", async () => {
  const echo = script("process.stdout.write(await Bun.stdin.text());");
  const options = { cwd: tmpdir(), env: {}, stdin: "", timeoutSeconds: 2 };
  expect(await capture(echo, [], { ...options, stdin: " é\n\n" })).toEqual({
    output: " é\n\n",
    error: null,
  });
  const fail = script("process.exit(9);");
  const failed = await capture(fail, [], options);
  expect(failed.error).toBe("agent-failed");
  const hang = script("setInterval(() => {}, 1000);");
  const timedOut = await capture(hang, [], { ...options, timeoutSeconds: 0.1 });
  expect(timedOut.error).toBe("timeout");
  const flood = script('process.stdout.write("x".repeat(5 * 1024 * 1024));');
  const flooded = await capture(flood, [], options);
  expect(flooded.error).toBe("output-limit");
  const missing = await capture("/nonexistent-synthetic-provider", [], options);
  expect(missing.error).toBe("agent-failed");
});

test("live adapter exercises the CLI protocol in a fresh HOME without real agent or inherited secrets", async () => {
  const executable = script(`
    const input = await Bun.stdin.text();
    await Bun.write("request.json", JSON.stringify({ input, argv: process.argv.slice(2), home: process.env.HOME, sentinel: process.env.PRIVATE_SENTINEL ?? null }));
    process.stdout.write(JSON.stringify({type:"turn.completed",usage:{input_tokens:4,cached_input_tokens:0,output_tokens:2}})+"\\n");
  `);
  const fixture = prepareFixture(
    process.cwd(),
    at(loadCases(process.cwd()), 0),
  );
  roots.push(fixture.root);
  const result = await codexExecutor(
    executable,
    seriesOptions,
    {},
  )(fixture, " é\n\n");
  expect(result.error).toBeNull();
  expect(result.tokens).toEqual({ input: 4, cachedInput: 0, output: 2 });
  const request: unknown = JSON.parse(
    readFileSync(join(fixture.workspace, "request.json"), "utf8"),
  );
  expect(request).toMatchObject({
    input: " é\n\n",
    home: fixture.home,
    sentinel: null,
  });
  expect(request).toHaveProperty(
    "argv",
    expect.arrayContaining([
      "--ignore-user-config",
      "--ephemeral",
      "explicit-model",
      "sandbox_workspace_write.network_access=false",
    ]),
  );
});

test("live adapter maps malformed output to INVALID input rather than success", async () => {
  const executable = script('process.stdout.write("not-json");');
  const fixture = prepareFixture(
    process.cwd(),
    at(loadCases(process.cwd()), 0),
  );
  roots.push(fixture.root);
  const result = await codexExecutor(
    executable,
    seriesOptions,
    {},
  )(fixture, "synthetic");
  expect(result.error).toBe("protocol-invalid");
});
