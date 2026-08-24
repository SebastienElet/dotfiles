import { readFileSync } from "node:fs";
import { join } from "node:path";
import { spawn } from "node:child_process";
import { z } from "zod";
import type { Command } from "./integration-fixture.ts";
import { delay } from "./mcp-client.ts";

const daemonSchema = z.object({ pid: z.number().int().positive() });
const decoder = new TextDecoder("utf-8", { fatal: true });

export class NetworkViolation extends Error {
  constructor(readonly sockets: string) {
    super("network sockets observed");
  }
}

export async function auditProcessNetwork(
  command: Command,
  environment: NodeJS.ProcessEnv,
  repository: string | undefined,
  timeoutMilliseconds = 120_000,
): Promise<{ stdout: string; stderr: string }> {
  const [binary, ...arguments_] = command;
  const child = spawn(binary, arguments_, {
    env: environment,
    stdio: ["ignore", "pipe", "pipe"],
    detached: true,
  });
  const stdout: Buffer[] = [];
  const stderr: Buffer[] = [];
  child.stdout.on("data", (chunk: Buffer) => stdout.push(chunk));
  child.stderr.on("data", (chunk: Buffer) => stderr.push(chunk));
  const exit = new Promise<{
    status: number | null;
    signal: NodeJS.Signals | null;
  }>((resolve, reject) => {
    child.once("error", reject);
    child.once("exit", (status, signal) => resolve({ status, signal }));
  });
  const sockets: string[] = [];
  const started = performance.now();
  let outcome:
    { status: number | null; signal: NodeJS.Signals | null } | undefined;
  try {
    while (outcome === undefined) {
      sampleSockets(child.pid, repository, sockets);
      if (performance.now() - started >= timeoutMilliseconds) {
        throw new Error(`audited process timed out: ${command.join(" ")}`);
      }
      outcome = await Promise.race([exit, delay(50).then(() => undefined)]);
    }
    sampleSockets(child.pid, repository, sockets);
  } catch (error) {
    stopProcessGroup(child.pid);
    await exit.catch(() => undefined);
    throw error;
  }
  const decoded = decodeOutput(stdout, stderr);
  if (sockets.length > 0) throw new NetworkViolation(sockets.join(""));
  if (outcome.status !== 0) {
    throw new Error(
      `audited process failed: status=${outcome.status ?? "none"} signal=${outcome.signal ?? "none"}\n${decoded.stderr}`,
    );
  }
  return decoded;
}

function sampleSockets(
  rootProcessId: number | undefined,
  repository: string | undefined,
  sockets: string[],
): void {
  if (rootProcessId === undefined)
    throw new Error("audited process has no PID");
  for (const processId of collectProcessIds(rootProcessId, repository)) {
    const result = Bun.spawnSync(
      ["lsof", "-nP", "-a", "-p", String(processId), "-i"],
      {
        stdout: "pipe",
        stderr: "pipe",
      },
    );
    if (result.exitCode === 0) {
      sockets.push(decode(result.stdout, "lsof stdout"));
    } else if (result.exitCode !== 1) {
      throw new Error(
        `lsof failed (${result.exitCode}): ${decode(result.stderr, "lsof stderr")}`,
      );
    }
  }
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
      const result = Bun.spawnSync(["pgrep", "-P", String(parent)], {
        stdout: "pipe",
        stderr: "pipe",
      });
      if (result.exitCode !== 0 && result.exitCode !== 1) {
        throw new Error(
          `pgrep failed (${result.exitCode}): ${decode(result.stderr, "pgrep stderr")}`,
        );
      }
      if (result.exitCode === 0) {
        for (const line of decode(result.stdout, "pgrep stdout")
          .trim()
          .split("\n")) {
          const child = Number(line);
          if (
            Number.isSafeInteger(child) &&
            child > 0 &&
            !collected.has(child)
          ) {
            collected.add(child);
            next.push(child);
          }
        }
      }
    }
    frontier = next;
  }
  const daemon =
    repository === undefined ? undefined : recordedDaemon(repository);
  if (daemon !== undefined) collected.add(daemon);
  return [...collected].sort((left, right) => left - right);
}

function recordedDaemon(repository: string): number | undefined {
  try {
    return daemonSchema.parse(
      JSON.parse(
        readFileSync(join(repository, ".codegraph", "daemon.pid"), "utf8"),
      ),
    ).pid;
  } catch (error) {
    if ((error as NodeJS.ErrnoException).code === "ENOENT") return undefined;
    throw new Error(`invalid CodeGraph daemon record: ${String(error)}`);
  }
}

function decodeOutput(stdout: Buffer[], stderr: Buffer[]) {
  return {
    stdout: decode(Buffer.concat(stdout), "audited stdout"),
    stderr: decode(Buffer.concat(stderr), "audited stderr"),
  };
}

function decode(bytes: Uint8Array, source: string): string {
  try {
    return decoder.decode(bytes);
  } catch {
    throw new Error(`${source} returned invalid UTF-8`);
  }
}

function stopProcessGroup(processId: number | undefined): void {
  if (processId === undefined) return;
  try {
    process.kill(-processId, "SIGKILL");
  } catch (error) {
    if ((error as NodeJS.ErrnoException).code !== "ESRCH") throw error;
  }
}
