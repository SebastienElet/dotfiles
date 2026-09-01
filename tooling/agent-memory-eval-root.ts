import { createHash, randomUUID } from "node:crypto";
import { chmod, copyFile, mkdir, mkdtemp, rm, stat } from "node:fs/promises";
import { tmpdir } from "node:os";
import { basename, dirname, join, resolve } from "node:path";

import {
  assertEvaluatorRoot,
  assertFixtureEnvironment,
  evaluationEnvironment,
  initializeRepository,
} from "./agent-memory-eval-fixture.ts";
import type { Agent } from "./agent-memory-eval-process.ts";
import { terminateEvaluationProcesses } from "./agent-memory-eval-supervision.ts";

type EvaluationFixture = Readonly<{
  environment: NodeJS.ProcessEnv;
  home: string;
  nonce: string;
  raw: string;
  root: string;
  repository: string;
  runtime: string;
  runtimeSource: string;
  store: string;
  validationStore: string;
}>;

async function withFreshEvaluationFixture<T>(
  agent: Agent,
  replicate: number,
  operation: (fixture: EvaluationFixture) => Promise<T>,
  runtimeSource = resolve(import.meta.dir, "../tooling/agent-memory/target/release/agent-memory"),
): Promise<T> {
  const root = await mkdtemp(join(tmpdir(), "agent-memory-eval-"));
  if (!basename(root).startsWith("agent-memory-eval-")) throw new Error("unsafe evaluator root");
  let interrupted = false;
  const interrupt = (exitCode: number) => {
    if (interrupted) return;
    interrupted = true;
    void terminateEvaluationProcesses()
      .then(() => rm(root, { force: true, recursive: true }))
      .then(() => assertRemoved(root))
      .finally(() => process.exit(exitCode));
  };
  const terminate = () => interrupt(143);
  const interruptFromTerminal = () => interrupt(130);
  process.once("SIGTERM", terminate);
  process.once("SIGINT", interruptFromTerminal);
  try {
    const home = join(root, "home");
    const repository = join(root, "repository");
    const store = join(root, "store");
    const raw = join(root, "raw");
    const validationStore = join(root, "validation-store");
    const runtime = join(home, ".local/bin/agent-memory");
    const nonce = `profile-${createHash("sha256").update(`${agent}:${replicate}:${randomUUID()}`).digest("hex").slice(0, 16)}`;
    for (const directory of [
      home,
      join(home, "tmp"),
      repository,
      validationStore,
      raw,
      dirname(runtime),
    ]) {
      assertEvaluatorRoot(root, directory);
      await mkdir(directory, { recursive: true, mode: 0o700 });
      await chmod(directory, 0o700);
    }
    try {
      await copyFile(runtimeSource, runtime);
    } catch {
      throw new Error("runtime unavailable: build agent-memory release first");
    }
    await chmod(runtime, 0o700);
    await initializeRepository(repository, nonce);
    const environment = evaluationEnvironment(home, store, runtime);
    assertFixtureEnvironment(root, repository, runtime, environment);
    return await operation({
      environment,
      home,
      nonce,
      raw,
      root,
      repository,
      runtime,
      runtimeSource,
      store,
      validationStore,
    });
  } finally {
    process.off("SIGTERM", terminate);
    process.off("SIGINT", interruptFromTerminal);
    if (!interrupted) {
      await rm(root, { force: true, recursive: true });
      await assertRemoved(root);
    }
  }
}

async function assertRemoved(root: string): Promise<void> {
  try {
    await stat(root);
    throw new Error(`cleanup failed for ${root}`);
  } catch (error) {
    if (!(error instanceof Error && "code" in error && error.code === "ENOENT")) throw error;
  }
}

export { withFreshEvaluationFixture };
export type { EvaluationFixture };
