import type { Command } from "./integration-fixture.ts";
import { delay } from "./mcp-client.ts";
import { join } from "node:path";
import { readFileSync } from "node:fs";
import { spawn } from "node:child_process";
import { z } from "zod";

const daemonSchema = z.object({ pid: z.number().int().positive() });
const decoder = new TextDecoder("utf-8", { fatal: true });
const defaultTimeoutMilliseconds = 120_000;
const sampleIntervalMilliseconds = 50;
const samplePending = Symbol("sample pending");
const byteChunkSchema = z.instanceof(Uint8Array);

class NetworkViolationError extends Error {
  public override readonly name = "NetworkViolationError";

  public constructor(public readonly sockets: string) {
    super("network sockets observed");
  }
}

type NetworkAuditOptions = Readonly<{
  environment: Readonly<NodeJS.ProcessEnv>;
  repository: string | undefined;
  timeoutMilliseconds?: number;
}>;
type ExitOutcome = Readonly<{
  signal: NodeJS.Signals | null;
  status: number | null;
}>;
type ProcessObservationOptions = Readonly<{
  command: Command;
  exit: Readonly<Promise<ExitOutcome>>;
  processId: number | undefined;
  repository: string | undefined;
  timeoutMilliseconds: number;
}>;

async function auditProcessNetwork(
  command: Command,
  {
    environment,
    repository,
    timeoutMilliseconds = defaultTimeoutMilliseconds,
  }: NetworkAuditOptions,
): Promise<{ stdout: string; stderr: string }> {
  const process = startAuditedProcess(command, environment);
  const { outcome, sockets } = await observeProcess({
    command,
    exit: process.exit,
    processId: process.processId,
    repository,
    timeoutMilliseconds,
  });
  const decoded = decodeOutput(process.stdout, process.stderr);
  requireNoSockets(sockets);
  requireSuccessfulExit(outcome, decoded.stderr);
  return decoded;
}

function startAuditedProcess(
  command: Command,
  environment: Readonly<NodeJS.ProcessEnv>,
): Readonly<{
  exit: Promise<ExitOutcome>;
  processId: number | undefined;
  stderr: number[][];
  stdout: number[][];
}> {
  const [binary, ...arguments_] = command;
  const child = spawn(binary, arguments_, {
    detached: true,
    env: environment,
    stdio: ["ignore", "pipe", "pipe"],
  });
  const stdout: number[][] = [];
  const stderr: number[][] = [];
  child.stdout.on("data", (chunk: unknown) => {
    stdout.push([...byteChunkSchema.parse(chunk)]);
  });
  child.stderr.on("data", (chunk: unknown) => {
    stderr.push([...byteChunkSchema.parse(chunk)]);
  });
  const exit = new Promise<ExitOutcome>((resolve, reject) => {
    child.once("error", reject);
    child.once("exit", (status, signal) => {
      resolve({ signal, status });
    });
  });
  return { exit, processId: child.pid, stderr, stdout };
}

async function observeProcess({
  command,
  exit,
  processId,
  repository,
  timeoutMilliseconds,
}: ProcessObservationOptions): Promise<
  Readonly<{ outcome: ExitOutcome; sockets: string[] }>
> {
  const sockets: string[] = [];
  const started = performance.now();
  let outcome:
    | { status: number | null; signal: NodeJS.Signals | null }
    | typeof samplePending = samplePending;
  try {
    while (outcome === samplePending) {
      sockets.push(...sampleSockets(processId, repository));
      if (performance.now() - started >= timeoutMilliseconds) {
        throw new Error(`audited process timed out: ${command.join(" ")}`);
      }
      outcome = await Promise.race([
        exit,
        delay(sampleIntervalMilliseconds).then(
          (): typeof samplePending => samplePending,
        ),
      ]);
    }
    sockets.push(...sampleSockets(processId, repository));
  } catch (error) {
    stopProcessGroup(processId);
    await exit.catch(() => samplePending);
    throw error;
  }
  return { outcome, sockets };
}

function requireNoSockets(sockets: readonly string[]): void {
  if (sockets.length > 0) {
    throw new NetworkViolationError(sockets.join(""));
  }
}

function requireSuccessfulExit(outcome: ExitOutcome, stderr: string): void {
  if (outcome.status !== 0) {
    throw new Error(
      `audited process failed: status=${outcome.status ?? "none"} signal=${outcome.signal ?? "none"}\n${stderr}`,
    );
  }
}

function sampleSockets(
  rootProcessId: number | undefined,
  repository: string | undefined,
): string[] {
  if (rootProcessId === undefined) {
    throw new Error("audited process has no PID");
  }
  return collectProcessIds(rootProcessId, repository).flatMap((processId) => {
    const result = Bun.spawnSync(
      ["lsof", "-nP", "-a", "-p", String(processId), "-i"],
      {
        stderr: "pipe",
        stdout: "pipe",
      },
    );
    if (result.exitCode === 0) {
      return [decode([...result.stdout], "lsof stdout")];
    }
    if (result.exitCode !== 1) {
      throw new Error(
        `lsof failed (${result.exitCode}): ${decode([...result.stderr], "lsof stderr")}`,
      );
    }
    return [];
  });
}

function collectProcessIds(
  rootProcessId: number,
  repository: string | undefined,
): number[] {
  const collected = new Set([rootProcessId]);
  let frontier = [rootProcessId];
  while (frontier.length > 0) {
    const next: number[] = [];
    for (const parent of frontier) {
      for (const child of childProcessIds(parent)) {
        if (!collected.has(child)) {
          collected.add(child);
          next.push(child);
        }
      }
    }
    frontier = next;
  }
  const daemon =
    repository === undefined ? undefined : recordedDaemon(repository);
  if (daemon !== undefined) {
    collected.add(daemon);
  }
  return [...collected].toSorted((left, right) => left - right);
}

function childProcessIds(parent: number): number[] {
  const result = Bun.spawnSync(["pgrep", "-P", String(parent)], {
    stderr: "pipe",
    stdout: "pipe",
  });
  if (result.exitCode !== 0 && result.exitCode !== 1) {
    throw new Error(
      `pgrep failed (${result.exitCode}): ${decode([...result.stderr], "pgrep stderr")}`,
    );
  }
  if (result.exitCode === 1) {
    return [];
  }
  return decode([...result.stdout], "pgrep stdout")
    .trim()
    .split("\n")
    .map(Number)
    .filter((processId) => Number.isSafeInteger(processId) && processId > 0);
}

function recordedDaemon(repository: string): number | undefined {
  try {
    return daemonSchema.parse(
      JSON.parse(
        readFileSync(join(repository, ".codegraph", "daemon.pid"), "utf8"),
      ),
    ).pid;
  } catch (error) {
    if (hasErrorCode(error, "ENOENT")) {
      return undefined;
    }
    throw new Error(`invalid CodeGraph daemon record: ${String(error)}`, {
      cause: error,
    });
  }
}

function decodeOutput(
  stdout: readonly (readonly number[])[],
  stderr: readonly (readonly number[])[],
): { stderr: string; stdout: string } {
  return {
    stderr: decode(stderr.flat(), "audited stderr"),
    stdout: decode(stdout.flat(), "audited stdout"),
  };
}

function decode(bytes: readonly number[], source: string): string {
  try {
    return decoder.decode(Uint8Array.from(bytes));
  } catch {
    throw new Error(`${source} returned invalid UTF-8`);
  }
}

function stopProcessGroup(processId: number | undefined): void {
  if (processId === undefined) {
    return;
  }
  try {
    process.kill(-processId, "SIGKILL");
  } catch (error) {
    if (!hasErrorCode(error, "ESRCH")) {
      throw error;
    }
  }
}

function hasErrorCode(error: unknown, code: string): boolean {
  return z.object({ code: z.literal(code) }).safeParse(error).success;
}

export { NetworkViolationError, auditProcessNetwork };
