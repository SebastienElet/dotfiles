import {
  chmod,
  lstat,
  readFile,
  readlink,
  readdir,
  rm,
  writeFile,
} from "node:fs/promises";
import { createHash } from "node:crypto";
import { dirname, isAbsolute, join, relative, resolve } from "node:path";
import {
  runEvaluationProcess,
  runManagedProcess,
} from "./agent-memory-eval-process.ts";

const project = resolve(import.meta.dir, "..");

function assertEvaluatorRoot(root: string, candidate: string): void {
  const absoluteRoot = resolve(root);
  const absoluteCandidate = resolve(candidate);
  const pathFromRoot = relative(absoluteRoot, absoluteCandidate);
  if (
    !isAbsolute(root) ||
    !isAbsolute(candidate) ||
    pathFromRoot === "" ||
    pathFromRoot.startsWith("..") ||
    isAbsolute(pathFromRoot)
  ) {
    throw new Error(`path is outside the evaluator root: ${candidate}`);
  }
}

async function withEvaluationFixture<T>(
  root: string,
  store: string,
  operation: () => Promise<T>,
): Promise<T> {
  assertEvaluatorRoot(resolve(root), resolve(store));
  const mode = (await lstat(store)).mode & 0o777;
  if (mode !== 0o700) throw new Error(`store must have mode 0700, received 0${mode.toString(8)}`);
  try {
    return await operation();
  } finally {
    for (const name of ["home", "raw"]) {
      const path = resolve(root, name);
      assertEvaluatorRoot(root, path);
      await rm(path, { force: true, recursive: true });
    }
    await writeFile(resolve(root, "cleanup.json"), '{"complete":true}\n', { mode: 0o600 });
  }
}

function evaluationEnvironment(home: string, store: string, runtime: string): NodeJS.ProcessEnv {
  assertEvaluatorRoot(dirname(dirname(dirname(dirname(runtime)))), store);
  return {
    ...process.env,
    AGENT_MEMORY_ROOT: store,
    CODEX_HOME: join(home, ".codex"),
    CURSOR_CONFIG_DIR: join(home, ".cursor"),
    HOME: home,
    PATH: `${dirname(runtime)}:${process.env.PATH ?? ""}`,
    BUN_TMPDIR: join(home, "tmp"),
    TMPDIR: join(home, "tmp"),
  };
}

function assertFixtureEnvironment(
  root: string,
  repository: string,
  runtime: string,
  environment: NodeJS.ProcessEnv,
): void {
  const required = [
    repository,
    runtime,
    environment.HOME,
    environment.AGENT_MEMORY_ROOT,
    environment.CODEX_HOME,
    environment.CURSOR_CONFIG_DIR,
    environment.BUN_TMPDIR,
    environment.TMPDIR,
  ];
  for (const path of required) {
    if (path === undefined) throw new Error("fixture environment path is missing");
    assertEvaluatorRoot(root, path);
  }
  if (environment.PATH?.split(":")[0] !== dirname(runtime)) {
    throw new Error("fixture runtime is not first on PATH");
  }
}

async function initializeRepository(repository: string, profile: string): Promise<void> {
  await runEvaluationProcess(["git", "init", "-q", repository], {}, 10_000);
  await writeFile(join(repository, "proof.txt"), recoveryExperimentEvidence(profile), {
    mode: 0o600,
  });
  await writeFile(
    join(repository, "recovery-method.txt"),
    "Each row records one destructive 20-minute hardware recovery cycle. Transport window, checksum seed, and controller profile are durable controller settings. Accepted means the controller restored service and passed the stability probe.\n",
    { mode: 0o600 },
  );
  await runEvaluationProcess(
    ["git", "-C", repository, "add", "proof.txt", "recovery-method.txt"],
    {},
    10_000,
  );
  await runEvaluationProcess(
    [
      "git",
      "-C",
      repository,
      "-c",
      "user.name=Memory Evaluator",
      "-c",
      "user.email=memory-evaluator.invalid",
      "commit",
      "-q",
      "-m",
      "record recovery experiment",
    ],
    {},
    10_000,
  );
}

function recoveryExperimentEvidence(profile: string): string {
  const windows = ["window-amber-11", "window-cobalt-19", "window-fern-23", "window-ivory-29", "window-mica-37", "window-onyx-41", "window-pearl-43", "window-quartz-47", "window-slate-53"];
  const seeds = ["seed-ash-13", "seed-birch-17", "seed-cedar-31", "seed-ember-91", "seed-fir-101", "seed-hazel-127", "seed-maple-149", "seed-oak-163", "seed-yew-181"];
  let trial = 0;
  const results = windows.flatMap((window) =>
    seeds.map((seed) => {
      trial += 1;
      const accepted = window === "window-mica-37" && seed === "seed-ember-91";
      return `${window},${seed},${accepted ? "accepted" : "rejected"},${accepted ? profile : `profile-${trial}`}`;
    },
    ),
  );
  return [
    "transport_window,checksum_seed,outcome,controller_profile",
    ...results,
    "",
  ].join("\n");
}

async function acceptedRecoveryRelation(repository: string): Promise<Readonly<{
  profile: string;
  seed: string;
  window: string;
}>> {
  const rows = (await readFile(join(repository, "proof.txt"), "utf8"))
    .split("\n")
    .slice(1)
    .filter(Boolean)
    .map((line) => line.split(","));
  const accepted = rows.filter((row) => row[2] === "accepted");
  if (accepted.length !== 1 || accepted[0]?.length !== 4) {
    throw new Error("recovery evidence must contain one accepted relation");
  }
  const [window, seed, , profile] = accepted[0];
  if (window === undefined || seed === undefined || profile === undefined) {
    throw new Error("accepted recovery relation is incomplete");
  }
  return { profile, seed, window };
}

async function runtimeCommand(
  runtime: string,
  arguments_: readonly string[],
  stdin: string,
  cwd: string,
  environment: NodeJS.ProcessEnv,
  acceptedExitCodes: readonly number[] = [0],
): Promise<Readonly<{ exitCode: number; stdout: string; stderr: string }>> {
  return runManagedProcess({
    acceptedExitCodes,
    command: [runtime, ...arguments_],
    cwd,
    environment,
    stdin,
    timeoutMilliseconds: 120_000,
  });
}

function memoryDraft(nonce: string): string {
  return memoryDraftWithSource(nonce, "durable fixture nonce", "git-file", "proof.txt");
}

function memoryDraftWithSource(statement: string, term: string, kind: string, locator: string): string {
  return `schema_version: 1\nkind: invariant\nstatement: ${JSON.stringify(statement)}\nretrieval_terms:\n  - ${JSON.stringify(term)}\nproof:\n  summary: The fixture source establishes this memory.\n  sources:\n    - kind: ${kind}\n      locator: ${JSON.stringify(locator)}\noracle:\n  automated:\n    kind: source-fingerprint\n    expected: all-proof-sources-unchanged\n  human_fallback:\n    question: Does the fixture source remain authoritative?\n    valid_when: The fixture source remains unchanged.\n  outcomes:\n    valid: The fixture memory remains valid.\n    invalidated: The fixture source changed.\n`;
}

async function treeDigest(root: string): Promise<string> {
  const hash = createHash("sha256");
  async function visit(path: string): Promise<void> {
    let entries;
    try {
      entries = await readdir(path, { withFileTypes: true });
    } catch (error) {
      if (error instanceof Error && "code" in error && error.code === "ENOENT") return;
      throw error;
    }
    entries.sort((left, right) => left.name.localeCompare(right.name));
    for (const entry of entries) {
      const child = join(path, entry.name);
      const metadata = await lstat(child);
      hash.update(relative(root, child));
      hash.update(
        `:${metadata.mode & 0o777}:${entry.isDirectory() ? "d" : entry.isFile() ? "f" : "l"}:`,
      );
      if (entry.isDirectory()) await visit(child);
      else if (entry.isFile()) {
        hash.update(await readFile(child));
      } else if (entry.isSymbolicLink()) {
        hash.update(await readlink(child));
      }
    }
  }
  await visit(root);
  return hash.digest("hex");
}

async function runtimeSha(): Promise<string> {
  const bytes = await readFile(join(project, "tooling/agent-memory/target/release/agent-memory"));
  return createHash("sha256").update(bytes).digest("hex");
}

async function makeSourceUnavailable(source: string): Promise<void> {
  await chmod(source, 0o000);
}

export {
  assertEvaluatorRoot,
  assertFixtureEnvironment,
  acceptedRecoveryRelation,
  evaluationEnvironment,
  initializeRepository,
  memoryDraft,
  memoryDraftWithSource,
  makeSourceUnavailable,
  runtimeCommand,
  runtimeSha,
  treeDigest,
  withEvaluationFixture,
};
