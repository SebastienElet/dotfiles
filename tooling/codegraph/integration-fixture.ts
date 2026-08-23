import {
  cpSync,
  existsSync,
  mkdtempSync,
  readFileSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { z } from "zod";

export type Command = readonly [string, ...string[]];

export type FreshnessFixture = Readonly<{
  root: string;
  repository: string;
}>;

export const privacyEnvironment: NodeJS.ProcessEnv = {
  ...process.env,
  CODEGRAPH_TELEMETRY: "0",
  CODEGRAPH_NO_UPDATE_CHECK: "1",
  CODEGRAPH_NO_DOWNLOAD: "1",
  CODEGRAPH_DAEMON_IDLE_TIMEOUT_MS: "500",
};

const decoder = new TextDecoder("utf-8", { fatal: true });
const daemonSchema = z.object({ pid: z.number().int().positive() });

export function runCommand(
  command: Command,
  arguments_: readonly string[],
  cwd: string,
  environment: NodeJS.ProcessEnv = privacyEnvironment,
): string {
  const result = Bun.spawnSync([...command, ...arguments_], {
    cwd,
    env: environment,
    stdout: "pipe",
    stderr: "pipe",
  });
  let stdout: string;
  let stderr: string;
  try {
    stdout = decoder.decode(result.stdout);
    stderr = decoder.decode(result.stderr);
  } catch {
    throw new Error(`${command[0]} returned invalid UTF-8`);
  }
  if (result.exitCode !== 0) {
    throw new Error(
      `${[...command, ...arguments_].join(" ")} failed (${result.exitCode}): ${stderr}`,
    );
  }
  return stdout;
}

export function createFreshnessFixture(
  fixtureSource: string,
): FreshnessFixture {
  const root = mkdtempSync(join(tmpdir(), "codegraph-integration-"));
  const repository = join(root, "repository");
  cpSync(fixtureSource, repository, { recursive: true });
  const git = (arguments_: readonly string[]) =>
    runCommand(["git"], arguments_, repository, process.env);
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
  return { root, repository };
}

export async function cleanupFreshnessFixture(
  fixture: FreshnessFixture,
  codegraph: Command,
): Promise<void> {
  let cleanupError: Error | undefined;
  try {
    runCommand(
      codegraph,
      ["uninit", "--force", fixture.repository],
      fixture.repository,
    );
    if (existsSync(join(fixture.repository, ".codegraph"))) {
      cleanupError = new Error("CodeGraph index remains after uninit");
      await stopRecordedDaemon(fixture.repository);
    }
  } catch (error) {
    cleanupError = error instanceof Error ? error : new Error(String(error));
    await stopRecordedDaemon(fixture.repository);
  } finally {
    rmSync(fixture.root, { recursive: true, force: true });
  }
  if (cleanupError !== undefined) {
    throw new Error(`CodeGraph cleanup failed: ${cleanupError.message}`);
  }
}

export function requireDaemonStopped(repository: string): void {
  const pidFile = join(repository, ".codegraph", "daemon.pid");
  if (!existsSync(pidFile)) return;
  const { pid } = daemonSchema.parse(JSON.parse(readFileSync(pidFile, "utf8")));
  try {
    process.kill(pid, 0);
  } catch (error) {
    if ((error as NodeJS.ErrnoException).code === "ESRCH") return;
    throw error;
  }
  throw new Error(`CodeGraph daemon still running: ${pid}`);
}

async function stopRecordedDaemon(repository: string): Promise<void> {
  const pidFile = join(repository, ".codegraph", "daemon.pid");
  if (!existsSync(pidFile)) return;
  let processId: number;
  try {
    const parsed = JSON.parse(readFileSync(pidFile, "utf8")) as {
      pid?: unknown;
    };
    if (!Number.isSafeInteger(parsed.pid) || Number(parsed.pid) <= 0) return;
    processId = Number(parsed.pid);
  } catch {
    return;
  }
  try {
    process.kill(processId, "SIGTERM");
  } catch (error) {
    if ((error as NodeJS.ErrnoException).code === "ESRCH") return;
    throw error;
  }
  for (let attempt = 0; attempt < 30; attempt += 1) {
    await Bun.sleep(100);
    try {
      process.kill(processId, 0);
    } catch (error) {
      if ((error as NodeJS.ErrnoException).code === "ESRCH") return;
      throw error;
    }
  }
  process.kill(processId, "SIGKILL");
}
