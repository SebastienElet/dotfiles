import { dirname, isAbsolute, join, relative, resolve } from "node:path";
import { lstat, readFile, rm, writeFile } from "node:fs/promises";
import {
  runEvaluationProcess,
  runManagedProcess,
} from "./agent-memory-eval-process.ts";
import { recoveryExperimentEvidence } from "./agent-memory-eval-fixture-evidence.ts";

const executableDirectoryMode = 0o700;
const fileModeMask = 0o777;
const gitTimeoutMilliseconds = 10_000;
const runtimeTimeoutMilliseconds = 120_000;
const expectedEvidenceColumns = 4;
const cleanupMarkerMode = 0o600;
const octalRadix = 8;
const outcomeColumnIndex = 2;
const profileColumnIndex = 3;

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

async function withEvaluationFixture<Result>(
  root: string,
  store: string,
  operation: () => Promise<Result>,
): Promise<Result> {
  assertEvaluatorRoot(resolve(root), resolve(store));
  const metadata = await lstat(store);
  const mode = metadata.mode & fileModeMask;
  if (mode !== executableDirectoryMode) {
    throw new Error(
      `store must have mode 0700, received 0${mode.toString(octalRadix)}`,
    );
  }
  try {
    return await operation();
  } finally {
    for (const name of ["home", "raw"]) {
      const path = resolve(root, name);
      assertEvaluatorRoot(root, path);
      await rm(path, { force: true, recursive: true });
    }
    await writeFile(resolve(root, "cleanup.json"), '{"complete":true}\n', {
      mode: cleanupMarkerMode,
    });
  }
}

function evaluationEnvironment(
  home: string,
  store: string,
  runtime: string,
): NodeJS.ProcessEnv {
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
  ...[root, repository, runtime, environment]: readonly [
    string,
    string,
    string,
    Readonly<NodeJS.ProcessEnv>,
  ]
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
    if (path === undefined) {
      throw new Error("fixture environment path is missing");
    }
    assertEvaluatorRoot(root, path);
  }
  if (environment.PATH?.split(":")[0] !== dirname(runtime)) {
    throw new Error("fixture runtime is not first on PATH");
  }
}

async function initializeRepository(
  repository: string,
  profile: string,
): Promise<void> {
  await runEvaluationProcess(
    ["git", "init", "-q", repository],
    {},
    gitTimeoutMilliseconds,
  );
  await writeFile(
    join(repository, "proof.txt"),
    recoveryExperimentEvidence(profile),
    {
      mode: 0o600,
    },
  );
  await writeFile(
    join(repository, "recovery-method.txt"),
    "Each row records one destructive 20-minute hardware recovery cycle. Transport window, checksum seed, and controller profile are durable controller settings. Accepted means the controller restored service and passed the stability probe.\n",
    { mode: 0o600 },
  );
  await runEvaluationProcess(
    ["git", "-C", repository, "add", "proof.txt", "recovery-method.txt"],
    {},
    gitTimeoutMilliseconds,
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
    gitTimeoutMilliseconds,
  );
}

async function acceptedRecoveryRelation(repository: string): Promise<
  Readonly<{
    profile: string;
    seed: string;
    window: string;
  }>
> {
  const evidence = await readFile(join(repository, "proof.txt"), "utf8");
  const rows: readonly (readonly string[])[] = evidence
    .split("\n")
    .slice(1)
    .filter(Boolean)
    .map((line) => line.split(","));
  const accepted = rows.filter((row) => {
    const outcome = row.at(outcomeColumnIndex);
    return outcome === "accepted";
  });
  if (
    accepted.length !== 1 ||
    accepted[0]?.length !== expectedEvidenceColumns
  ) {
    throw new Error("recovery evidence must contain one accepted relation");
  }
  const [relation] = accepted;
  const window = relation?.at(0);
  const seed = relation?.at(1);
  const profile = relation?.at(profileColumnIndex);
  if (window === undefined || seed === undefined || profile === undefined) {
    throw new Error("accepted recovery relation is incomplete");
  }
  return { profile, seed, window };
}

function runtimeCommand(
  ...[
    runtime,
    arguments_,
    stdin,
    cwd,
    environment,
    acceptedExitCodes,
  ]: readonly [
    string,
    readonly string[],
    string,
    string,
    Readonly<NodeJS.ProcessEnv>,
    (readonly number[])?,
  ]
): Promise<Readonly<{ exitCode: number; stdout: string; stderr: string }>> {
  return runManagedProcess({
    acceptedExitCodes: acceptedExitCodes ?? [0],
    command: [runtime, ...arguments_],
    cwd,
    environment,
    stdin,
    timeoutMilliseconds: runtimeTimeoutMilliseconds,
  });
}

function memoryDraft(nonce: string): string {
  return memoryDraftWithSource(
    nonce,
    "durable fixture nonce",
    "git-file",
    "proof.txt",
  );
}

function memoryDraftWithSource(
  ...[statement, term, kind, locator]: readonly [string, string, string, string]
): string {
  return `schema_version: 1\nkind: invariant\nstatement: ${JSON.stringify(statement)}\nretrieval_terms:\n  - ${JSON.stringify(term)}\nproof:\n  summary: The fixture source establishes this memory.\n  sources:\n    - kind: ${kind}\n      locator: ${JSON.stringify(locator)}\noracle:\n  automated:\n    kind: source-fingerprint\n    expected: all-proof-sources-unchanged\n  human_fallback:\n    question: Does the fixture source remain authoritative?\n    valid_when: The fixture source remains unchanged.\n  outcomes:\n    valid: The fixture memory remains valid.\n    invalidated: The fixture source changed.\n`;
}

export {
  assertEvaluatorRoot,
  assertFixtureEnvironment,
  acceptedRecoveryRelation,
  evaluationEnvironment,
  initializeRepository,
  memoryDraft,
  memoryDraftWithSource,
  runtimeCommand,
  withEvaluationFixture,
};
export {
  makeSourceUnavailable,
  runtimeSha,
  treeDigest,
} from "./agent-memory-eval-fixture-files.ts";
