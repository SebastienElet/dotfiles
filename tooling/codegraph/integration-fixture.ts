import {
  cpSync,
  existsSync,
  mkdtempSync,
  readFileSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { join } from "node:path";
import { tmpdir } from "node:os";
import { z } from "zod";

type Command = readonly [string, ...string[]];

type FreshnessFixture = Readonly<{
  root: string;
  repository: string;
}>;

type RunCommandOptions = Readonly<{
  cwd: string;
  environment?: Readonly<NodeJS.ProcessEnv>;
}>;

type OperationOutcome<Result> =
  | Readonly<{ ok: true; value: Result }>
  | Readonly<{ error: unknown; ok: false }>;

const privacyEnvironment: NodeJS.ProcessEnv = {
  ...process.env,
  CODEGRAPH_DAEMON_IDLE_TIMEOUT_MS: "500",
  CODEGRAPH_NO_DOWNLOAD: "1",
  CODEGRAPH_NO_UPDATE_CHECK: "1",
  CODEGRAPH_TELEMETRY: "0",
};

const decoder = new TextDecoder("utf-8", { fatal: true });
const daemonSchema = z.object({ pid: z.number().int().positive() });
const daemonStopAttemptLimit = 30;
const daemonStopPollMilliseconds = 100;

function runCommand(
  command: Command,
  arguments_: readonly string[],
  { cwd, environment = privacyEnvironment }: RunCommandOptions,
): string {
  const result = Bun.spawnSync([...command, ...arguments_], {
    cwd,
    env: environment,
    stderr: "pipe",
    stdout: "pipe",
  });
  const { stderr, stdout } = decodeCommandOutput(
    [...result.stdout],
    [...result.stderr],
    command,
  );
  if (result.exitCode !== 0) {
    throw new Error(
      `${[...command, ...arguments_].join(" ")} failed (${result.exitCode}): ${stderr}`,
    );
  }
  return stdout;
}

function decodeCommandOutput(
  stdoutBytes: readonly number[],
  stderrBytes: readonly number[],
  command: Command,
): Readonly<{ stderr: string; stdout: string }> {
  try {
    return {
      stderr: decoder.decode(Uint8Array.from(stderrBytes)),
      stdout: decoder.decode(Uint8Array.from(stdoutBytes)),
    };
  } catch {
    throw new Error(`${command[0]} returned invalid UTF-8`);
  }
}

function createFreshnessFixture(fixtureSource: string): FreshnessFixture {
  const root = mkdtempSync(join(tmpdir(), "codegraph-integration-"));
  const repository = join(root, "repository");
  cpSync(fixtureSource, repository, { recursive: true });
  const git = (arguments_: readonly string[]): string =>
    runCommand(["git"], arguments_, {
      cwd: repository,
      environment: process.env,
    });
  git(["init", "-b", "main"]);
  git(["config", "user.email", "codegraph-fixture@example.invalid"]);
  git(["config", "user.name", "CodeGraph Fixture"]);
  git(["add", "."]);
  git(["commit", "-m", "baseline"]);
  git(["switch", "-c", "codegraph-alt"]);
  writeFileSync(
    join(repository, "src", "branch.ts"),
    'export const branchAltValue = 30\nexport const branchSentinel = "FIXTURE_BRANCH_ALT"\n',
  );
  git(["add", "src/branch.ts"]);
  git(["commit", "-m", "alternate"]);
  git(["switch", "main"]);
  return { repository, root };
}

async function cleanupFreshnessFixture(
  fixture: FreshnessFixture,
  codegraph: Command,
): Promise<void> {
  const outcome = await captureOperation(() =>
    uninitializeFreshnessFixture(fixture, codegraph),
  );
  rmSync(fixture.root, { force: true, recursive: true });
  const cleanupError = outcome.ok ? outcome.value : asError(outcome.error);
  if (cleanupError !== undefined) {
    throw new Error(`CodeGraph cleanup failed: ${cleanupError.message}`);
  }
}

async function uninitializeFreshnessFixture(
  fixture: FreshnessFixture,
  codegraph: Command,
): Promise<Error | undefined> {
  try {
    runCommand(codegraph, ["uninit", "--force", fixture.repository], {
      cwd: fixture.repository,
    });
    if (existsSync(join(fixture.repository, ".codegraph"))) {
      await stopRecordedDaemon(fixture.repository);
      return new Error("CodeGraph index remains after uninit");
    }
    return undefined;
  } catch (error) {
    await stopRecordedDaemon(fixture.repository);
    return error instanceof Error ? error : new Error(String(error));
  }
}

async function captureOperation<Result>(
  operation: () => Promise<Result>,
): Promise<OperationOutcome<Result>> {
  try {
    return { ok: true, value: await operation() };
  } catch (error) {
    return { error, ok: false };
  }
}

function requireDaemonStopped(repository: string): void {
  const pidFile = join(repository, ".codegraph", "daemon.pid");
  if (!existsSync(pidFile)) {
    return;
  }
  const { pid } = daemonSchema.parse(JSON.parse(readFileSync(pidFile, "utf8")));
  try {
    process.kill(pid, 0);
  } catch (error) {
    if (hasErrorCode(error, "ESRCH")) {
      return;
    }
    throw error;
  }
  throw new Error(`CodeGraph daemon still running: ${pid}`);
}

async function stopRecordedDaemon(repository: string): Promise<void> {
  const pidFile = join(repository, ".codegraph", "daemon.pid");
  const processId = recordedProcessId(pidFile);
  if (processId === undefined) {
    return;
  }
  if (!signalProcess(processId, "SIGTERM")) {
    return;
  }
  if (await waitForProcessExit(processId)) {
    return;
  }
  process.kill(processId, "SIGKILL");
}

function recordedProcessId(pidFile: string): number | undefined {
  if (!existsSync(pidFile)) {
    return undefined;
  }
  try {
    const parsed = daemonSchema.safeParse(
      JSON.parse(readFileSync(pidFile, "utf8")),
    );
    return parsed.success ? parsed.data.pid : undefined;
  } catch {
    return undefined;
  }
}

function signalProcess(
  processId: number,
  signal: NodeJS.Signals | number,
): boolean {
  try {
    process.kill(processId, signal);
    return true;
  } catch (error) {
    if (hasErrorCode(error, "ESRCH")) {
      return false;
    }
    throw error;
  }
}

function asError(error: unknown): Error {
  return error instanceof Error ? error : new Error(String(error));
}

async function waitForProcessExit(processId: number): Promise<boolean> {
  for (let attempt = 0; attempt < daemonStopAttemptLimit; attempt += 1) {
    await Bun.sleep(daemonStopPollMilliseconds);
    if (!signalProcess(processId, 0)) {
      return true;
    }
  }
  return false;
}

function hasErrorCode(error: unknown, code: string): boolean {
  return z.object({ code: z.literal(code) }).safeParse(error).success;
}

export {
  cleanupFreshnessFixture,
  captureOperation,
  type Command,
  createFreshnessFixture,
  type FreshnessFixture,
  privacyEnvironment,
  requireDaemonStopped,
  type RunCommandOptions,
  runCommand,
};
