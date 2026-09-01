import { afterEach, expect, test } from "bun:test";
import {
  buildAgentCommand,
  makeSourceUnavailable,
  runEvaluationProcess,
} from "./agent-memory-eval.ts";
import {
  chmod,
  mkdtemp,
  readFile,
  rm,
  stat,
  writeFile,
} from "node:fs/promises";
import { cacheContainsFixture } from "./agent-memory-eval-cache.ts";
import { join } from "node:path";
import { tmpdir } from "node:os";

const roots: string[] = [];
const commandPrefixLength = 2;
const privateFileMode = 0o600;
const executableFileMode = 0o755;
const standardPermissionMask = 0o777;
const JSON_INDENTATION = 2;
const shortTimeoutMilliseconds = 1000;
const timeoutGraceMilliseconds = 20;

afterEach(async () => {
  await Promise.all(
    roots.splice(0).map((root) => rm(root, { force: true, recursive: true })),
  );
});

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
  expect(codex.slice(0, commandPrefixLength)).toEqual([
    "/usr/bin/sandbox-exec",
    "-p",
  ]);
  expect(codex).toContain("danger-full-access");
  expect(claude.slice(0, commandPrefixLength)).toEqual([
    "/usr/bin/sandbox-exec",
    "-p",
  ]);
  expect(claude).toContain("plan");
  expect(claude).not.toContain("bypassPermissions");
  expect(cursor).toContain("plan");
  expect(cursor).not.toContain("--trust");
});

test("sandbox denies reads and writes outside the fixture root", async () => {
  if (process.platform !== "darwin") {
    return;
  }
  const root = await fixtureRoot();
  const outside = join(tmpdir(), `agent-memory-outside-${process.pid}`);
  try {
    await writeFile(outside, "outside");
    const command = buildAgentCommand(
      "codex",
      root,
      root,
      "proposal",
      "prompt",
    );
    const [profile] = command.slice(commandPrefixLength);
    if (profile === undefined) {
      throw new Error("sandbox profile missing");
    }
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
  await writeFile(source, "proof", { mode: privateFileMode });
  await makeSourceUnavailable(source);
  const sourceStatus = await stat(source);
  expect(sourceStatus.mode & standardPermissionMask).toBe(0);
});

test("requires the fixture-specific cache entry rather than an empty cache", async () => {
  const root = await fixtureRoot();
  const cache = join(root, "oracle-cache.json");
  await writeFile(
    join(root, "index.json"),
    `${JSON.stringify({ entries: [{ id: "mem_fixture", retrieval_terms: ["durable fixture nonce"] }] }, null, JSON_INDENTATION)}\n`,
    { mode: privateFileMode },
  );
  await writeFile(
    cache,
    `${JSON.stringify({ schema_version: 1, entries: [] }, null, JSON_INDENTATION)}\n`,
    {
      mode: privateFileMode,
    },
  );
  expect(await cacheContainsFixture(cache, "durable fixture nonce")).toBe(
    false,
  );
  await writeFile(
    cache,
    `${JSON.stringify({ schema_version: 1, entries: [{ entry_id: "mem_fixture", verdict: "valid" }] }, null, JSON_INDENTATION)}\n`,
    { mode: privateFileMode },
  );
  expect(await cacheContainsFixture(cache, "durable fixture nonce")).toBe(true);
});

test("rejects a non-zero process, timeout, and missing invocation", async () => {
  const root = await fixtureRoot();
  const failure = await executable(root, "failure", "process.exit(7)");
  expect(
    runEvaluationProcess([failure], {}, shortTimeoutMilliseconds),
  ).rejects.toThrow("exit 7");

  const timeout = await executable(root, "timeout", "await Bun.sleep(5000)");
  expect(
    runEvaluationProcess([timeout], {}, timeoutGraceMilliseconds),
  ).rejects.toThrow("timed out");

  const stubborn = await executable(
    root,
    "stubborn",
    'process.on("SIGTERM", () => undefined); setInterval(() => undefined, 1000)',
  );
  const started = performance.now();
  expect(
    runEvaluationProcess([stubborn], {}, timeoutGraceMilliseconds),
  ).rejects.toThrow("timed out");
  expect(performance.now() - started).toBeLessThan(shortTimeoutMilliseconds);
});

test("kills descendants that ignore SIGTERM within the timeout grace", async () => {
  const root = await fixtureRoot();
  const childPid = join(root, "child.pid");
  const parent = await executable(
    root,
    "descendants",
    `const child = Bun.spawn([process.execPath, "-e", 'process.on("SIGTERM", () => undefined); setInterval(() => undefined, 1000)'], { stderr: "ignore", stdout: "ignore" }); await Bun.write(${JSON.stringify(childPid)}, String(child.pid)); process.on("SIGTERM", () => undefined); setInterval(() => undefined, 1000)`,
  );
  expect(
    runEvaluationProcess([parent], {}, shortTimeoutMilliseconds),
  ).rejects.toThrow("timed out");
  const pid = Number(await readFile(childPid, "utf8"));
  let alive = true;
  try {
    process.kill(pid, 0);
  } catch {
    alive = false;
  }
  if (alive) {
    process.kill(pid, "SIGKILL");
  }
  expect(alive).toBe(false);
});

test("redacts a credential printed by a failing process", async () => {
  const root = await fixtureRoot();
  const token = "cursor-private-token-value";
  const failure = await executable(
    root,
    "credential-failure",
    "console.error(process.env.CURSOR_AUTH_TOKEN); process.exit(1)",
  );
  let diagnostic = "";
  try {
    await runEvaluationProcess(
      [failure],
      { CURSOR_AUTH_TOKEN: token },
      shortTimeoutMilliseconds,
    );
  } catch (error) {
    diagnostic = error instanceof Error ? error.message : String(error);
  }
  expect(diagnostic).not.toContain(token);
  expect(diagnostic).toContain("redacted_process_failure");
});

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
