import { afterEach, describe, expect, test } from "bun:test";
import { chmod, mkdir, mkdtemp, readFile, rm, stat, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";

import "./agent-memory-eval-evidence-tests.ts";
import "./agent-memory-eval-action-tests.ts";
import "./agent-memory-eval-contract-tests.ts";
import "./agent-memory-eval-report-tests.ts";
import "./agent-memory-eval-runner-tests.ts";
import "./agent-memory-eval-fixture-tests.ts";
import "./agent-memory-eval-process-tests.ts";

import {
  assertEvaluatorRoot,
  assertFixtureEnvironment,
  buildAgentCommand,
  makeSourceUnavailable,
  runEvaluationProcess,
  withEvaluationFixture,
} from "./agent-memory-eval.ts";
import { cacheContainsFixture } from "./agent-memory-eval-cache.ts";
import { runTraceObservedProcess } from "./agent-memory-eval-process.ts";

const roots: string[] = [];

afterEach(async () => {
  await Promise.all(roots.splice(0).map((root) => rm(root, { force: true, recursive: true })));
});

describe("agent memory evaluator runner", () => {
  test("confines agents without bypass modes", () => {
    const codex = buildAgentCommand(
      "codex",
      process.cwd(),
      process.cwd(),
      "proposal",
      "prompt",
    );
    const claude = buildAgentCommand(
      "claude",
      process.cwd(),
      process.cwd(),
      "proposal",
      "prompt",
    );
    const cursor = buildAgentCommand(
      "cursor",
      process.cwd(),
      process.cwd(),
      "proposal",
      "prompt",
    );
    expect(codex.slice(0, 2)).toEqual(["/usr/bin/sandbox-exec", "-p"]);
    expect(codex).toContain("danger-full-access");
    expect(claude.slice(0, 2)).toEqual(["/usr/bin/sandbox-exec", "-p"]);
    expect(claude).toContain("plan");
    expect(claude).not.toContain("bypassPermissions");
    expect(cursor).toContain("plan");
    expect(cursor).not.toContain("--trust");
  });

  test("sandbox denies reads and writes outside the fixture root", async () => {
    if (process.platform !== "darwin") return;
    const root = await fixtureRoot();
    const outside = join(tmpdir(), `agent-memory-outside-${process.pid}`);
    try {
      await writeFile(outside, "outside");
      const command = buildAgentCommand("codex", root, root, "proposal", "prompt");
      const profile = command[2];
      if (profile === undefined) throw new Error("sandbox profile missing");
      const denied = Bun.spawnSync([
        "/usr/bin/sandbox-exec",
        "-p",
        profile,
        "/bin/sh",
        "-c",
        `cat '${outside}' >/dev/null || exit 41; touch '${outside}.write'`,
      ]);
      expect(denied.exitCode).not.toBe(0);
    } finally {
      await rm(outside, { force: true });
      await rm(`${outside}.write`, { force: true });
    }
  });

  test("keeps an unavailable source present while denying reads", async () => {
    const root = await fixtureRoot();
    const source = join(root, "source.txt");
    await writeFile(source, "proof", { mode: 0o600 });
    await makeSourceUnavailable(source);
    expect((await stat(source)).mode & 0o777).toBe(0);
  });

  test("requires the fixture-specific cache entry rather than an empty cache", async () => {
    const root = await fixtureRoot();
    const cache = join(root, "oracle-cache.json");
    await writeFile(
      join(root, "index.json"),
      `${JSON.stringify({ entries: [{ id: "mem_fixture", retrieval_terms: ["durable fixture nonce"] }] }, null, 2)}\n`,
      { mode: 0o600 },
    );
    await writeFile(cache, `${JSON.stringify({ schema_version: 1, entries: [] }, null, 2)}\n`, {
      mode: 0o600,
    });
    expect(await cacheContainsFixture(cache, "durable fixture nonce")).toBe(false);
    await writeFile(
      cache,
      `${JSON.stringify({ schema_version: 1, entries: [{ entry_id: "mem_fixture", verdict: "valid" }] }, null, 2)}\n`,
      { mode: 0o600 },
    );
    expect(await cacheContainsFixture(cache, "durable fixture nonce")).toBe(true);
  });

  test("rejects a non-zero process, timeout, and missing invocation", async () => {
    const root = await fixtureRoot();
    const failure = await executable(root, "failure", "process.exit(7)");
    await expect(runEvaluationProcess([failure], {}, 1_000)).rejects.toThrow("exit 7");

    const timeout = await executable(root, "timeout", "await Bun.sleep(5000)");
    await expect(runEvaluationProcess([timeout], {}, 20)).rejects.toThrow("timed out");

    const stubborn = await executable(
      root,
      "stubborn",
      'process.on("SIGTERM", () => undefined); setInterval(() => undefined, 1000)',
    );
    const started = performance.now();
    await expect(runEvaluationProcess([stubborn], {}, 20)).rejects.toThrow("timed out");
    expect(performance.now() - started).toBeLessThan(1_000);

  });

  test("kills descendants that ignore SIGTERM within the timeout grace", async () => {
    const root = await fixtureRoot();
    const childPid = join(root, "child.pid");
    const parent = await executable(
      root,
      "descendants",
      `const child = Bun.spawn([process.execPath, "-e", 'process.on("SIGTERM", () => undefined); setInterval(() => undefined, 1000)'], { stderr: "ignore", stdout: "ignore" }); await Bun.write(${JSON.stringify(childPid)}, String(child.pid)); process.on("SIGTERM", () => undefined); setInterval(() => undefined, 1000)`,
    );
    await expect(runEvaluationProcess([parent], {}, 1_000)).rejects.toThrow("timed out");
    const pid = Number(await readFile(childPid, "utf8"));
    let alive = true;
    try {
      process.kill(pid, 0);
    } catch {
      alive = false;
    }
    if (alive) process.kill(pid, "SIGKILL");
    expect(alive).toBe(false);
  });

  test("redacts a credential printed by a failing process", async () => {
    const root = await fixtureRoot();
    const token = "cursor-private-token-value";
    const failure = await executable(root, "credential-failure", "console.error(process.env.CURSOR_AUTH_TOKEN); process.exit(1)");
    let diagnostic = "";
    try {
      await runEvaluationProcess([failure], { CURSOR_AUTH_TOKEN: token }, 1_000);
    } catch (error) {
      diagnostic = error instanceof Error ? error.message : String(error);
    }
    expect(diagnostic).not.toContain(token);
    expect(diagnostic).toContain("redacted_process_failure");
  });

  test("observes trace completion before the nonce model output", async () => {
    const root = await fixtureRoot();
    const trace = join(root, "trace.jsonl");
    const nonce = "trace-order-nonce";
    const events = [
      { agent: "codex", command: "hook", event: "started", exit_class: "started", pid: 7, timestamp_ms: 10 },
      { agent: "codex", command: "hook", event: "completed", exit_class: "success", pid: 7, timestamp_ms: 11 },
    ];
    const model = `${JSON.stringify({ type: "item.completed", item: { type: "agent_message", text: nonce } })}\n`;
    const command = await executable(
      root,
      "trace-before-model",
      `await Bun.write(process.env.AGENT_MEMORY_EVAL_TRACE, ${JSON.stringify(events.map((event) => JSON.stringify(event)).join("\n") + "\n")}); process.stdout.write(${JSON.stringify(model)});`,
    );
    const output = await runTraceObservedProcess(
      [command],
      {
        ...process.env,
        AGENT_MEMORY_EVAL_AGENT: "codex",
        AGENT_MEMORY_EVAL_TRACE: trace,
      },
      trace,
      "codex",
      "relevant",
      nonce,
      1_000,
    );
    expect(output.traceCompletedBeforeModel).toBe(true);
    expect(output.runtimeTrace).toContain('"event":"completed"');
  });

  test("does not credit a trace completed after the first model event", async () => {
    const root = await fixtureRoot();
    const trace = join(root, "late-trace.jsonl");
    const nonce = "late-trace-nonce";
    const model = `${JSON.stringify({ type: "item.completed", item: { type: "agent_message", text: nonce } })}\n`;
    const events = [
      { agent: "codex", command: "hook", event: "started", exit_class: "started", pid: 7, timestamp_ms: 10 },
      { agent: "codex", command: "hook", event: "completed", exit_class: "success", pid: 7, timestamp_ms: 11 },
    ];
    const command = await executable(
      root,
      "trace-after-model",
      `process.stdout.write(${JSON.stringify(model)}); await Bun.sleep(50); await Bun.write(process.env.AGENT_MEMORY_EVAL_TRACE, ${JSON.stringify(events.map((event) => JSON.stringify(event)).join("\n") + "\n")}); await Bun.sleep(20); process.stdout.write(${JSON.stringify('{"type":"turn.completed"}\n')});`,
    );
    const output = await runTraceObservedProcess(
      [command],
      { ...process.env, AGENT_MEMORY_EVAL_AGENT: "codex", AGENT_MEMORY_EVAL_TRACE: trace },
      trace,
      "codex",
      "relevant",
      nonce,
      1_000,
    );
    expect(output.traceCompletedBeforeModel).toBe(false);
  });

  test("refuses stores outside the evaluator root and invalid permissions", async () => {
    const root = await fixtureRoot();
    expect(() => assertEvaluatorRoot(root, join(tmpdir(), "personal-store"))).toThrow("outside");
    const store = join(root, "store");
    await mkdir(store, { mode: 0o755 });
    await expect(withEvaluationFixture(root, store, async () => undefined)).rejects.toThrow(
      "0700",
    );
  });

  test("rejects a personal store in the production fixture environment", async () => {
    const root = await fixtureRoot();
    const repository = join(root, "repository");
    const runtime = join(root, "home/.local/bin/agent-memory");
    expect(() =>
      assertFixtureEnvironment(root, repository, runtime, {
        AGENT_MEMORY_ROOT: join(tmpdir(), "personal-store"),
        HOME: join(root, "home"),
        PATH: `${join(root, "home/.local/bin")}:${process.env.PATH ?? ""}`,
      }),
    ).toThrow("outside");
  });

  test("removes credentials and raw output after success, failure, and interruption", async () => {
    for (const outcome of ["success", "failure", "interrupted"] as const) {
      const root = await fixtureRoot();
      const store = join(root, "store");
      await mkdir(store, { mode: 0o700 });
      const credential = join(root, "home", "auth.json");
      const raw = join(root, "raw", "events.jsonl");
      await mkdir(join(root, "home"), { recursive: true, mode: 0o700 });
      await mkdir(join(root, "raw"), { recursive: true, mode: 0o700 });
      await writeFile(credential, "private", { mode: 0o600 });
      await writeFile(raw, "private", { mode: 0o600 });
      const operation = withEvaluationFixture(root, store, async () => {
        if (outcome !== "success") throw new Error(outcome);
      });
      if (outcome === "success") await operation;
      else await expect(operation).rejects.toThrow(outcome);
      await expect(stat(credential)).rejects.toThrow();
      await expect(stat(raw)).rejects.toThrow();
      expect(await readFile(join(root, "cleanup.json"), "utf8")).toContain('"complete":true');
    }
  });
});

async function fixtureRoot(): Promise<string> {
  const root = await mkdtemp(join(tmpdir(), "agent-memory-eval-test-"));
  roots.push(root);
  return root;
}

async function executable(root: string, name: string, body: string): Promise<string> {
  const path = join(root, name);
  await writeFile(path, `#!/usr/bin/env bun\n${body}\n`);
  await chmod(path, 0o755);
  return path;
}
