import {
  type Command,
  type FreshnessFixture,
  captureOperation,
  cleanupFreshnessFixture,
  createFreshnessFixture,
  privacyEnvironment,
  runCommand,
} from "./codegraph/integration-fixture.ts";
import { type ProbeReport, runFreshnessProbe } from "./codegraph/mcp-probe.ts";
import { join } from "node:path";
import { readFileSync } from "node:fs";
import { z } from "zod";

const statusSchema = z.record(z.string(), z.unknown());
const millisecondsPerSecond = 1000;
const jsonIndentSpaces = 2;
const userMetricIndex = 2;
const systemMetricIndex = 4;

interface InitialIndexMeasurement {
  cpuSystemSeconds: number;
  cpuUserSeconds: number;
  maxRssBytes: number;
  seconds: number;
}

interface McpTestReport {
  environment: { linuxExercised: false; os: "macOS" };
  indexDiskKiB: number;
  initialIndexCpuSystemSeconds: number;
  initialIndexCpuUserSeconds: number;
  initialIndexMaxRssBytes: number;
  initialIndexSeconds: number;
  mcp: ProbeReport;
}

async function runCodeGraphMcpTest(): Promise<McpTestReport> {
  requirePlatform();
  const codegraph = resolveCommand("CODEGRAPH_BIN", "codegraph");
  const fixture = createFreshnessFixture(
    join(import.meta.dir, "codegraph", "fixtures", "freshness"),
  );
  const outcome = await captureOperation(() =>
    collectMcpTestReport(fixture, codegraph),
  );
  const operationError = outcome.ok ? undefined : outcome.error;
  await cleanupMcpFixture(fixture, codegraph, operationError);
  if (!outcome.ok) {
    throw new Error(message(outcome.error), { cause: outcome.error });
  }
  return outcome.value;
}

async function collectMcpTestReport(
  fixture: FreshnessFixture,
  codegraph: Command,
): Promise<McpTestReport> {
  const initial = measureInitialIndex(
    codegraph,
    fixture.repository,
    fixture.root,
  );
  statusSchema.parse(
    JSON.parse(
      runCommand(codegraph, ["status", "--json", fixture.repository], {
        cwd: fixture.repository,
      }),
    ),
  );
  const mcp = await runFreshnessProbe(fixture.repository, codegraph);
  return {
    environment: { linuxExercised: false, os: "macOS" },
    indexDiskKiB: diskKiB(fixture.repository),
    initialIndexCpuSystemSeconds: initial.cpuSystemSeconds,
    initialIndexCpuUserSeconds: initial.cpuUserSeconds,
    initialIndexMaxRssBytes: initial.maxRssBytes,
    initialIndexSeconds: initial.seconds,
    mcp,
  };
}

async function cleanupMcpFixture(
  fixture: FreshnessFixture,
  codegraph: Command,
  operationError: unknown,
): Promise<void> {
  try {
    await cleanupFreshnessFixture(fixture, codegraph);
  } catch (cleanupError) {
    if (operationError === undefined) {
      throw cleanupError;
    }
    throw new Error(
      `CodeGraph test failed: ${message(operationError)}; cleanup failed: ${message(cleanupError)}`,
      { cause: cleanupError },
    );
  }
}

function measureInitialIndex(
  codegraph: Command,
  repository: string,
  root: string,
): InitialIndexMeasurement {
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
    { cwd: repository, environment: process.env },
  );
  const report = readFileSync(timeLog, "utf8");
  const firstLine = report.split("\n")[0]?.trim().split(/\s+/u) ?? [];
  const maximumRss =
    /^\s*(?<maximumRss>\d+)\s+maximum resident set size/mu.exec(report)?.groups
      ?.maximumRss;
  const userSeconds = firstLine.at(userMetricIndex);
  const systemSeconds = firstLine.at(systemMetricIndex);
  if (
    maximumRss === undefined ||
    userSeconds === undefined ||
    systemSeconds === undefined
  ) {
    throw new Error(`invalid /usr/bin/time report: ${report}`);
  }
  return {
    cpuSystemSeconds: parseMetric(systemSeconds, "system CPU"),
    cpuUserSeconds: parseMetric(userSeconds, "user CPU"),
    maxRssBytes: parseMetric(maximumRss, "maximum RSS"),
    seconds: Math.floor((performance.now() - started) / millisecondsPerSecond),
  };
}

function diskKiB(repository: string): number {
  const output = runCommand(["du"], ["-sk", join(repository, ".codegraph")], {
    cwd: repository,
  });
  return parseMetric(output.trim().split(/\s+/u)[0] ?? "", "index disk KiB");
}

function parseMetric(value: string, name: string): number {
  const parsed = Number(value);
  if (!Number.isFinite(parsed) || parsed < 0) {
    throw new Error(`invalid ${name}: ${value}`);
  }
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
  if (process.platform !== "darwin") {
    throw new Error("CodeGraph MCP test requires macOS");
  }
  if (Bun.which("git") === null) {
    throw new Error("git is required");
  }
  if (Bun.which("du") === null) {
    throw new Error("du is required");
  }
  if (Bun.file("/usr/bin/time").size === 0) {
    throw new Error("/usr/bin/time is required");
  }
}

function message(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}

if (import.meta.main) {
  try {
    process.stdout.write(
      `${JSON.stringify(await runCodeGraphMcpTest(), undefined, jsonIndentSpaces)}\n`,
    );
  } catch (error) {
    process.stderr.write(
      `${error instanceof Error ? error.message : String(error)}\n`,
    );
    process.exit(1);
  }
}

export { runCodeGraphMcpTest };
