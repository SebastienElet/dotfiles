import {
  type EvaluationFixture,
  withFreshEvaluationFixture,
} from "./agent-memory-eval-root.ts";
import { expect, test } from "bun:test";
import { join, resolve } from "node:path";
import {
  loadEvaluationScenarios,
  scenarioById,
} from "./agent-memory-eval-scenario.ts";
import { memoryDraft, runtimeCommand } from "./agent-memory-eval-fixture.ts";
import { mkdtemp, readFile, readdir, rm, stat } from "node:fs/promises";
import { prepareAgent } from "./agent-memory-eval-auth.ts";
import { runManagedProcess } from "./agent-memory-eval-process.ts";
import { tmpdir } from "node:os";

const acceptedEvidenceCount = 1;
const successfulExitCode = 0;
const unavailableExitCode = 2;
const rejectedExitCode = 3;
const invalidExitCode = 4;
const admissionExitCodes = [
  successfulExitCode,
  unavailableExitCode,
  rejectedExitCode,
  invalidExitCode,
];
const fixturePollingAttempts = 100;
const fixturePollingMilliseconds = 10;
const minimumRejectedEvidenceCount = 50;
const signalTerminationExitCode = 143;
const standardTimeoutMilliseconds = 10_000;
const longTimeoutMilliseconds = 120_000;

test("creates committed primary evidence for a costly recovery experiment", async () => {
  await withFreshEvaluationFixture(
    "claude",
    1,
    async (fixture: EvaluationFixture) => {
      const head = await runManagedProcess({
        command: ["git", "rev-parse", "--verify", "HEAD"],
        cwd: fixture.repository,
        environment: fixture.environment,
        timeoutMilliseconds: standardTimeoutMilliseconds,
      });
      expect(head.stdout.trim()).not.toBe("");
      const evidence = await readFile(
        join(fixture.repository, "proof.txt"),
        "utf8",
      );
      expect(
        evidence.split("\n").filter((line) => line.includes(",accepted,")),
      ).toHaveLength(acceptedEvidenceCount);
      expect(
        evidence.split("\n").filter((line) => line.includes(",rejected,"))
          .length,
      ).toBeGreaterThan(minimumRejectedEvidenceCount);
      expect(evidence).not.toContain("must use window-mica-37");
    },
  );
});

test("removes the active fixture after SIGTERM", async () => {
  const diagnosticRoot = await mkdtemp(
    join(tmpdir(), "agent-memory-signal-test-"),
  );
  const marker = join(diagnosticRoot, "root.txt");
  const module = resolve(import.meta.dir, "agent-memory-eval-root.ts");
  const script = `import { withFreshEvaluationFixture } from ${JSON.stringify(module)}; await withFreshEvaluationFixture("claude", 1, async (fixture) => { await Bun.write(${JSON.stringify(marker)}, fixture.root); await Bun.sleep(60000); });`;
  const child = Bun.spawn([process.execPath, "-e", script], {
    stderr: "pipe",
    stdout: "pipe",
  });
  try {
    for (let attempt = 0; attempt < fixturePollingAttempts; attempt += 1) {
      if (await Bun.file(marker).exists()) {
        break;
      }
      await Bun.sleep(fixturePollingMilliseconds);
    }
    const fixtureRoot = await readFile(marker, "utf8");
    child.kill("SIGTERM");
    const childExitCode = await child.exited;
    expect(childExitCode).toBe(signalTerminationExitCode);
    expect(stat(fixtureRoot)).rejects.toThrow();
  } finally {
    child.kill("SIGKILL");
    await rm(diagnosticRoot, { force: true, recursive: true });
  }
});

test("removes a fixture when runtime setup fails", async () => {
  const temporaryEntriesBefore = await readdir(tmpdir());
  const before = new Set(
    temporaryEntriesBefore.filter((name) =>
      name.startsWith("agent-memory-eval-"),
    ),
  );
  expect(
    withFreshEvaluationFixture(
      "claude",
      1,
      async () => {
        await Promise.resolve();
      },
      join(tmpdir(), "missing-agent-memory-runtime"),
    ),
  ).rejects.toThrow("runtime unavailable");
  const temporaryEntriesAfter = await readdir(tmpdir());
  const created = temporaryEntriesAfter.filter(
    (name) => name.startsWith("agent-memory-eval-") && !before.has(name),
  );
  expect(created).toEqual([]);
});

test("keeps the candidate store absent until explicit admission", async () => {
  const scenarios = await loadEvaluationScenarios(
    resolve(import.meta.dir, "agent-memory-eval-scenarios.json"),
  );
  await withFreshEvaluationFixture(
    "cursor",
    1,
    async (fixture: EvaluationFixture) => {
      expect(stat(fixture.store)).rejects.toThrow();
      await prepareAgent(
        "cursor",
        fixture.home,
        fixture.runtime,
        fixture.environment,
      );
      expect(stat(fixture.store)).rejects.toThrow();
      const trace = join(fixture.raw, "proposal-runtime.jsonl");
      const proposal = scenarioById(scenarios, "propose-without-writing");
      const output = await runManagedProcess({
        acceptedExitCodes: admissionExitCodes,
        command: [fixture.runtime, "hook", "--agent", "codex"],
        cwd: fixture.repository,
        environment: {
          ...fixture.environment,
          AGENT_MEMORY_EVAL_AGENT: "codex",
          AGENT_MEMORY_EVAL_ROOT: fixture.root,
          AGENT_MEMORY_EVAL_TRACE: trace,
        },
        stdin: JSON.stringify({
          cwd: fixture.repository,
          hook_event_name: "UserPromptSubmit",
          prompt: proposal.prompt,
        }),
        timeoutMilliseconds: longTimeoutMilliseconds,
      });
      expect(output.exitCode).toBe(0);
      expect(stat(fixture.store)).rejects.toThrow();
      const admission = await runtimeCommand(
        fixture.runtime,
        ["admit", "--format", "json"],
        memoryDraft(fixture.nonce),
        fixture.repository,
        fixture.environment,
      );
      expect(admission.exitCode).toBe(0);
      const storeStatus = await stat(fixture.store);
      expect(storeStatus.isDirectory()).toBe(true);
    },
  );
});
