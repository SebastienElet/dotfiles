import {
  assertEvaluatorRoot,
  assertFixtureEnvironment,
  evaluationEnvironment,
  initializeRepository,
} from "./agent-memory-eval-fixture.ts";
import { basename, dirname, join, resolve } from "node:path";
import { chmod, copyFile, mkdir, mkdtemp, rm, stat } from "node:fs/promises";
import { createHash, randomUUID } from "node:crypto";
import type { Agent } from "./agent-memory-eval-process.ts";
import { rmSync } from "node:fs";
import { tmpdir } from "node:os";

type EvaluationFixture = Readonly<{
  environment: Readonly<NodeJS.ProcessEnv>;
  home: string;
  nonce: string;
  raw: string;
  repository: string;
  root: string;
  runtime: string;
  runtimeSource: string;
  store: string;
  validationStore: string;
}>;
type FixturePaths = Readonly<{
  home: string;
  raw: string;
  repository: string;
  root: string;
  runtime: string;
  store: string;
  validationStore: string;
}>;

const directoryMode = 0o700;
const nonceLength = 16;
const sigintExitCode = 130;
const sigtermExitCode = 143;
const runtimeSourceDefault = resolve(
  import.meta.dir,
  "../tooling/agent-memory/target/release/agent-memory",
);

async function withFreshEvaluationFixture<Result>(
  ...[
    agent,
    replicate,
    operation,
    runtimeSource = runtimeSourceDefault,
  ]: readonly [
    Agent,
    number,
    (fixture: Readonly<EvaluationFixture>) => Promise<Result>,
    string?,
  ]
): Promise<Result> {
  const root = await mkdtemp(join(tmpdir(), "agent-memory-eval-"));
  assertFixtureRoot(root);
  const terminate = (): void => {
    removeInterruptedFixture(root, sigtermExitCode);
  };
  const interrupt = (): void => {
    removeInterruptedFixture(root, sigintExitCode);
  };
  process.once("SIGTERM", terminate);
  process.once("SIGINT", interrupt);
  try {
    const paths = fixturePaths(root);
    await createFixtureDirectories(paths);
    await prepareRuntime(runtimeSource, paths.runtime);
    return await operation(
      await evaluationFixture({ agent, paths, replicate, runtimeSource }),
    );
  } finally {
    process.off("SIGTERM", terminate);
    process.off("SIGINT", interrupt);
    await removeFixture(root);
  }
}

function removeInterruptedFixture(root: string, exitCode: number): void {
  rmSync(root, { force: true, recursive: true });
  process.exit(exitCode);
}

async function removeFixture(root: string): Promise<void> {
  await rm(root, { force: true, recursive: true });
  await assertRemoved(root);
}

function fixturePaths(root: string): FixturePaths {
  const home = join(root, "home");
  return {
    home,
    raw: join(root, "raw"),
    repository: join(root, "repository"),
    root,
    runtime: join(home, ".local/bin/agent-memory"),
    store: join(root, "store"),
    validationStore: join(root, "validation-store"),
  };
}

async function createFixtureDirectories(paths: FixturePaths): Promise<void> {
  const directories = [
    paths.home,
    join(paths.home, "tmp"),
    paths.repository,
    paths.validationStore,
    paths.raw,
    dirname(paths.runtime),
  ];
  for (const directory of directories) {
    assertEvaluatorRoot(paths.root, directory);
    await mkdir(directory, { mode: directoryMode, recursive: true });
    await chmod(directory, directoryMode);
  }
}

async function prepareRuntime(source: string, runtime: string): Promise<void> {
  try {
    await copyFile(source, runtime);
  } catch {
    throw new Error("runtime unavailable: build agent-memory release first");
  }
  await chmod(runtime, directoryMode);
}

async function evaluationFixture(
  request: Readonly<{
    agent: Agent;
    paths: FixturePaths;
    replicate: number;
    runtimeSource: string;
  }>,
): Promise<EvaluationFixture> {
  const nonce = fixtureNonce(request.agent, request.replicate);
  await initializeRepository(request.paths.repository, nonce);
  const environment = evaluationEnvironment(
    request.paths.home,
    request.paths.store,
    request.paths.runtime,
  );
  assertFixtureEnvironment(
    request.paths.root,
    request.paths.repository,
    request.paths.runtime,
    environment,
  );
  return {
    ...request.paths,
    environment,
    nonce,
    runtimeSource: request.runtimeSource,
  };
}

function fixtureNonce(agent: Agent, replicate: number): string {
  const value = `${agent}:${replicate}:${randomUUID()}`;
  const digest = createHash("sha256").update(value).digest("hex");
  return `profile-${digest.slice(0, nonceLength)}`;
}

function assertFixtureRoot(root: string): void {
  if (!basename(root).startsWith("agent-memory-eval-")) {
    throw new Error("unsafe evaluator root");
  }
}

async function assertRemoved(root: string): Promise<void> {
  try {
    await stat(root);
    throw new Error(`cleanup failed for ${root}`);
  } catch (error) {
    if (
      !(error instanceof Error && "code" in error && error.code === "ENOENT")
    ) {
      throw error;
    }
  }
}

export { withFreshEvaluationFixture };
export type { EvaluationFixture };
