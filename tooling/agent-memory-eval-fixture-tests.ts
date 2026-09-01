import { expect, test } from "bun:test";
import { mkdtemp, readFile, readdir, rm, stat } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";

import { prepareAgent } from "./agent-memory-eval-auth.ts";
import { memoryDraft, runtimeCommand } from "./agent-memory-eval-fixture.ts";
import { runManagedProcess } from "./agent-memory-eval-process.ts";
import { withFreshEvaluationFixture } from "./agent-memory-eval-root.ts";
import {
  loadEvaluationScenarios,
  scenarioById,
} from "./agent-memory-eval-scenario.ts";

test("creates committed primary evidence for a costly recovery experiment", async () => {
  await withFreshEvaluationFixture("claude", 1, async (fixture) => {
    const head = await runManagedProcess({
      command: ["git", "rev-parse", "--verify", "HEAD"],
      cwd: fixture.repository,
      environment: fixture.environment,
      timeoutMilliseconds: 10_000,
    });
    expect(head.stdout.trim()).not.toBe("");
    const evidence = await readFile(join(fixture.repository, "proof.txt"), "utf8");
    expect(evidence.split("\n").filter((line) => line.includes(",accepted,"))).toHaveLength(1);
    expect(evidence.split("\n").filter((line) => line.includes(",rejected,")).length).toBeGreaterThan(50);
    expect(evidence).not.toContain("must use window-mica-37");
  });
});

test("removes the active fixture after SIGTERM", async () => {
  const diagnosticRoot = await mkdtemp(join(tmpdir(), "agent-memory-signal-test-"));
  const marker = join(diagnosticRoot, "root.txt");
  const module = resolve(import.meta.dir, "agent-memory-eval-root.ts");
  const script = `import { withFreshEvaluationFixture } from ${JSON.stringify(module)}; await withFreshEvaluationFixture("claude", 1, async (fixture) => { await Bun.write(${JSON.stringify(marker)}, fixture.root); await Bun.sleep(60000); });`;
  const child = Bun.spawn([process.execPath, "-e", script], { stderr: "pipe", stdout: "pipe" });
  try {
    for (let attempt = 0; attempt < 100; attempt += 1) {
      if (await Bun.file(marker).exists()) break;
      await Bun.sleep(10);
    }
    const fixtureRoot = await readFile(marker, "utf8");
    child.kill("SIGTERM");
    expect(await child.exited).toBe(143);
    await expect(stat(fixtureRoot)).rejects.toThrow();
  } finally {
    child.kill("SIGKILL");
    await rm(diagnosticRoot, { force: true, recursive: true });
  }
});

test("removes a fixture when runtime setup fails", async () => {
  const before = new Set(
    (await readdir(tmpdir())).filter((name) => name.startsWith("agent-memory-eval-")),
  );
  await expect(
    withFreshEvaluationFixture(
      "claude",
      1,
      async () => undefined,
      join(tmpdir(), "missing-agent-memory-runtime"),
    ),
  ).rejects.toThrow("runtime unavailable");
  const created = (await readdir(tmpdir())).filter(
    (name) => name.startsWith("agent-memory-eval-") && !before.has(name),
  );
  expect(created).toEqual([]);
});

test("keeps the candidate store absent until explicit admission", async () => {
  const scenarios = await loadEvaluationScenarios(
    resolve(import.meta.dir, "agent-memory-eval-scenarios.json"),
  );
  await withFreshEvaluationFixture("cursor", 1, async (fixture) => {
    await expect(stat(fixture.store)).rejects.toThrow();
    await prepareAgent("cursor", fixture.home, fixture.runtime, fixture.environment);
    await expect(stat(fixture.store)).rejects.toThrow();
    const trace = join(fixture.raw, "proposal-runtime.jsonl");
    const proposal = scenarioById(scenarios, "propose-without-writing");
    const output = await runManagedProcess({
      acceptedExitCodes: [0, 2, 3, 4],
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
      timeoutMilliseconds: 120_000,
    });
    expect(output.exitCode).toBe(0);
    await expect(stat(fixture.store)).rejects.toThrow();
    const admission = await runtimeCommand(
      fixture.runtime,
      ["admit", "--format", "json"],
      memoryDraft(fixture.nonce),
      fixture.repository,
      fixture.environment,
    );
    expect(admission.exitCode).toBe(0);
    expect((await stat(fixture.store)).isDirectory()).toBe(true);
  });
});
