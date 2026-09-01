import { afterEach, expect, test } from "bun:test";
import {
  assertEvaluatorRoot,
  assertFixtureEnvironment,
  withEvaluationFixture,
} from "./agent-memory-eval.ts";
import {
  chmod,
  mkdir,
  mkdtemp,
  readFile,
  rm,
  stat,
  writeFile,
} from "node:fs/promises";
import { join } from "node:path";
import { runTraceObservedProcess } from "./agent-memory-eval-process.ts";
import { tmpdir } from "node:os";

const roots: string[] = [];
const executableFileMode = 0o755;
const privateDirectoryMode = 0o700;
const privateFileMode = 0o600;
const permissiveDirectoryMode = 0o755;
const shortTimeoutMilliseconds = 1000;
const delayedTraceMilliseconds = 50;
const followUpOutputDelayMilliseconds = 20;

afterEach(async () => {
  await Promise.all(
    roots.splice(0).map((root) => rm(root, { force: true, recursive: true })),
  );
});

test("observes trace completion before the nonce model output", async () => {
  const root = await fixtureRoot();
  const trace = join(root, "trace.jsonl");
  const nonce = "trace-order-nonce";
  const events = traceEvents();
  const model = modelEvent(nonce);
  const command = await executable(
    root,
    "trace-before-model",
    `await Bun.write(process.env.AGENT_MEMORY_EVAL_TRACE, ${JSON.stringify(jsonLines(events))}); process.stdout.write(${JSON.stringify(model)});`,
  );
  const output = await runTraceObservedProcess(
    [command],
    traceEnvironment(trace),
    trace,
    "codex",
    "relevant",
    nonce,
    shortTimeoutMilliseconds,
  );
  expect(output.traceCompletedBeforeModel).toBe(true);
  expect(output.runtimeTrace).toContain('"event":"completed"');
});

test("does not credit a trace completed after the first model event", async () => {
  const root = await fixtureRoot();
  const trace = join(root, "late-trace.jsonl");
  const nonce = "late-trace-nonce";
  const command = await executable(
    root,
    "trace-after-model",
    `process.stdout.write(${JSON.stringify(modelEvent(nonce))}); await Bun.sleep(${delayedTraceMilliseconds}); await Bun.write(process.env.AGENT_MEMORY_EVAL_TRACE, ${JSON.stringify(jsonLines(traceEvents()))}); await Bun.sleep(${followUpOutputDelayMilliseconds}); process.stdout.write(${JSON.stringify('{"type":"turn.completed"}\n')});`,
  );
  const output = await runTraceObservedProcess(
    [command],
    traceEnvironment(trace),
    trace,
    "codex",
    "relevant",
    nonce,
    shortTimeoutMilliseconds,
  );
  expect(output.traceCompletedBeforeModel).toBe(false);
});

test("refuses stores outside the evaluator root and invalid permissions", async () => {
  const root = await fixtureRoot();
  expect(() => {
    assertEvaluatorRoot(root, join(tmpdir(), "personal-store"));
  }).toThrow("outside");
  const store = join(root, "store");
  await mkdir(store, { mode: permissiveDirectoryMode });
  expect(withEvaluationFixture(root, store, completedFixture)).rejects.toThrow(
    "0700",
  );
});

test("rejects a personal store in the production fixture environment", async () => {
  const root = await fixtureRoot();
  const repository = join(root, "repository");
  const runtime = join(root, "home/.local/bin/agent-memory");
  expect(() => {
    assertFixtureEnvironment(root, repository, runtime, {
      AGENT_MEMORY_ROOT: join(tmpdir(), "personal-store"),
      HOME: join(root, "home"),
      PATH: `${join(root, "home/.local/bin")}:${process.env.PATH ?? ""}`,
    });
  }).toThrow("outside");
});

test("removes credentials and raw output after success, failure, and interruption", async () => {
  for (const outcome of ["success", "failure", "interrupted"] as const) {
    const root = await fixtureRoot();
    const store = join(root, "store");
    await mkdir(store, { mode: privateDirectoryMode });
    const credential = join(root, "home", "auth.json");
    const raw = join(root, "raw", "events.jsonl");
    await mkdir(join(root, "home"), {
      recursive: true,
      mode: privateDirectoryMode,
    });
    await mkdir(join(root, "raw"), {
      recursive: true,
      mode: privateDirectoryMode,
    });
    await writeFile(credential, "private", { mode: privateFileMode });
    await writeFile(raw, "private", { mode: privateFileMode });
    const operation = withEvaluationFixture(root, store, () =>
      failFor(outcome),
    );
    if (outcome === "success") {
      await operation;
    } else {
      expect(operation).rejects.toThrow(outcome);
    }
    expect(stat(credential)).rejects.toThrow();
    expect(stat(raw)).rejects.toThrow();
    expect(await readFile(join(root, "cleanup.json"), "utf8")).toContain(
      '"complete":true',
    );
  }
});

function completedFixture(): Promise<void> {
  return Promise.resolve();
}

function failFor(
  outcome: "success" | "failure" | "interrupted",
): Promise<void> {
  if (outcome !== "success") {
    return Promise.reject(new Error(outcome));
  }
  return Promise.resolve();
}

function traceEvents(): Record<string, string | number>[] {
  return [
    {
      agent: "codex",
      command: "hook",
      event: "started",
      exit_class: "started",
      pid: 7,
      timestamp_ms: 10,
    },
    {
      agent: "codex",
      command: "hook",
      event: "completed",
      exit_class: "success",
      pid: 7,
      timestamp_ms: 11,
    },
  ];
}

function jsonLines(
  events: readonly Readonly<Record<string, string | number>>[],
): string {
  return `${events.map((event) => JSON.stringify(event)).join("\n")}\n`;
}

function modelEvent(nonce: string): string {
  return `${JSON.stringify({ type: "item.completed", item: { type: "agent_message", text: nonce } })}\n`;
}

function traceEnvironment(trace: string): NodeJS.ProcessEnv {
  return {
    ...process.env,
    AGENT_MEMORY_EVAL_AGENT: "codex",
    AGENT_MEMORY_EVAL_TRACE: trace,
  };
}

async function fixtureRoot(): Promise<string> {
  const root = await mkdtemp(join(tmpdir(), "agent-memory-eval-test-"));
  roots.push(root);
  return root;
}

async function executable(
  root: string,
  name: string,
  body: string,
): Promise<string> {
  const path = join(root, name);
  await writeFile(path, `#!/usr/bin/env bun\n${body}\n`);
  await chmod(path, executableFileMode);
  return path;
}
