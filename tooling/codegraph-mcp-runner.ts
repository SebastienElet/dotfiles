import { readFileSync } from "node:fs";
import { join } from "node:path";
import { z } from "zod";
import {
  cleanupFreshnessFixture,
  createFreshnessFixture,
  privacyEnvironment,
  runCommand,
  type Command,
} from "./codegraph/integration-fixture.ts";
import { runFreshnessProbe } from "./codegraph/mcp-probe.ts";

const statusSchema = z.record(z.string(), z.unknown());

export async function runCodeGraphMcpTest() {
  requirePlatform();
  const codegraph = resolveCommand("CODEGRAPH_BIN", "codegraph");
  const fixture = createFreshnessFixture(
    join(import.meta.dir, "codegraph", "fixtures", "freshness"),
  );
  let operationError: unknown;
  try {
    const initial = measureInitialIndex(
      codegraph,
      fixture.repository,
      fixture.root,
    );
    statusSchema.parse(
      JSON.parse(
        runCommand(
          codegraph,
          ["status", "--json", fixture.repository],
          fixture.repository,
        ),
      ),
    );
    const mcp = await runFreshnessProbe(fixture.repository, codegraph);
    return {
      environment: { os: "macOS", linuxExercised: false },
      initialIndexSeconds: initial.seconds,
      initialIndexMaxRssBytes: initial.maxRssBytes,
      initialIndexCpuUserSeconds: initial.cpuUserSeconds,
      initialIndexCpuSystemSeconds: initial.cpuSystemSeconds,
      indexDiskKiB: diskKiB(fixture.repository),
      mcp,
    };
  } catch (error) {
    operationError = error;
    throw error;
  } finally {
    try {
      await cleanupFreshnessFixture(fixture, codegraph);
    } catch (cleanupError) {
      if (operationError === undefined) throw cleanupError;
      throw new Error(
        `CodeGraph test failed: ${message(operationError)}; cleanup failed: ${message(cleanupError)}`,
      );
    }
  }
}

function measureInitialIndex(
  codegraph: Command,
  repository: string,
  root: string,
) {
  const timeLog = join(root, "index-time");
  const started = performance.now();
  runCommand(
    ["/usr/bin/time"],
    [
      "-l",
      "-o",
      timeLog,
      "env",
      ...privacyAssignments(),
      ...codegraph,
      "init",
      repository,
    ],
    repository,
    process.env,
  );
  const report = readFileSync(timeLog, "utf8");
  const firstLine = report.split("\n")[0]?.trim().split(/\s+/) ?? [];
  const maximumRss = report.match(
    /^\s*(\d+)\s+maximum resident set size/m,
  )?.[1];
  const userSeconds = firstLine[2];
  const systemSeconds = firstLine[4];
  if (
    maximumRss === undefined ||
    userSeconds === undefined ||
    systemSeconds === undefined
  ) {
    throw new Error(`invalid /usr/bin/time report: ${report}`);
  }
  return {
    seconds: Math.floor((performance.now() - started) / 1_000),
    maxRssBytes: parseMetric(maximumRss, "maximum RSS"),
    cpuUserSeconds: parseMetric(userSeconds, "user CPU"),
    cpuSystemSeconds: parseMetric(systemSeconds, "system CPU"),
  };
}

function diskKiB(repository: string): number {
  const output = runCommand(
    ["du"],
    ["-sk", join(repository, ".codegraph")],
    repository,
  );
  return parseMetric(output.trim().split(/\s+/)[0] ?? "", "index disk KiB");
}

function parseMetric(value: string, name: string): number {
  const parsed = Number(value);
  if (!Number.isFinite(parsed) || parsed < 0)
    throw new Error(`invalid ${name}: ${value}`);
  return parsed;
}

function privacyAssignments(): string[] {
  return [
    "CODEGRAPH_TELEMETRY=0",
    "CODEGRAPH_NO_UPDATE_CHECK=1",
    "CODEGRAPH_NO_DOWNLOAD=1",
    `CODEGRAPH_DAEMON_IDLE_TIMEOUT_MS=${privacyEnvironment.CODEGRAPH_DAEMON_IDLE_TIMEOUT_MS}`,
  ];
}

function resolveCommand(variable: string, fallback: string): Command {
  const binary = process.env[variable] ?? Bun.which(fallback);
  if (binary === null || binary === undefined || binary === "") {
    throw new Error(`${fallback} is required`);
  }
  return [binary];
}

function requirePlatform(): void {
  if (process.platform !== "darwin")
    throw new Error("CodeGraph MCP test requires macOS");
  if (Bun.which("git") === null) throw new Error("git is required");
  if (Bun.which("du") === null) throw new Error("du is required");
  if (!Bun.file("/usr/bin/time").size)
    throw new Error("/usr/bin/time is required");
}

function message(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}

if (import.meta.main) {
  try {
    process.stdout.write(
      `${JSON.stringify(await runCodeGraphMcpTest(), null, 2)}\n`,
    );
  } catch (error) {
    process.stderr.write(
      `${error instanceof Error ? error.message : String(error)}\n`,
    );
    process.exit(1);
  }
}
