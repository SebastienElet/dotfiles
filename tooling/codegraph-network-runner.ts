import {
  type Command,
  type FreshnessFixture,
  captureOperation,
  cleanupFreshnessFixture,
  createFreshnessFixture,
  privacyEnvironment,
} from "./codegraph/integration-fixture.ts";
import {
  NetworkViolationError,
  auditProcessNetwork,
} from "./codegraph/network-audit.ts";
import { join } from "node:path";
import { z } from "zod";

const probeSchema = z.object({
  scenarios: z.object({ daemonStopped: z.literal(true) }),
});
const networkSampleIntervalMilliseconds = 50;

interface NetworkTestReport {
  daemonStopped: true;
  descendantsRecursive: true;
  networkSocketsObserved: 0;
  phases: { init: true; mcp: true; sync: true };
  sampleIntervalMs: typeof networkSampleIntervalMilliseconds;
}

async function runCodeGraphNetworkTest(): Promise<NetworkTestReport> {
  requireNetworkPlatform();
  const binary = process.env.CODEGRAPH_BIN ?? Bun.which("codegraph");
  if (binary === null || binary === undefined) {
    throw new Error("codegraph is required");
  }
  const codegraph: Command = [binary];
  await proveNetworkCanary();
  const fixture = createFreshnessFixture(
    join(import.meta.dir, "codegraph", "fixtures", "freshness"),
  );
  const outcome = await captureOperation(() =>
    collectNetworkTestReport(fixture, codegraph, binary),
  );
  const operationError = outcome.ok ? undefined : outcome.error;
  await cleanupNetworkFixture(fixture, codegraph, operationError);
  if (!outcome.ok) {
    throw new Error(message(outcome.error), { cause: outcome.error });
  }
  return outcome.value;
}

async function collectNetworkTestReport(
  fixture: FreshnessFixture,
  codegraph: Command,
  binary: string,
): Promise<NetworkTestReport> {
  const options = {
    environment: privacyEnvironment,
    repository: fixture.repository,
  };
  await auditProcessNetwork(
    [...codegraph, "init", fixture.repository],
    options,
  );
  await auditProcessNetwork(
    [...codegraph, "sync", fixture.repository],
    options,
  );
  const probe = await auditProcessNetwork(
    [
      process.execPath,
      join(import.meta.dir, "codegraph-mcp-probe.ts"),
      fixture.repository,
    ],
    {
      environment: {
        ...privacyEnvironment,
        CODEGRAPH_BIN: binary,
        CODEGRAPH_PROBE_PAUSE_MS: "500",
      },
      repository: fixture.repository,
    },
  );
  probeSchema.parse(JSON.parse(probe.stdout));
  return {
    daemonStopped: true,
    descendantsRecursive: true,
    networkSocketsObserved: 0,
    phases: { init: true, mcp: true, sync: true },
    sampleIntervalMs: networkSampleIntervalMilliseconds,
  };
}

async function cleanupNetworkFixture(
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
      `network test failed: ${message(operationError)}; cleanup failed: ${message(cleanupError)}`,
      { cause: cleanupError },
    );
  }
}

async function proveNetworkCanary(): Promise<void> {
  try {
    await auditProcessNetwork(
      [
        process.execPath,
        join(import.meta.dir, "codegraph", "network-canary.ts"),
        "root",
      ],
      {
        environment: process.env,
        repository: undefined,
        timeoutMilliseconds: 10_000,
      },
    );
  } catch (error) {
    if (error instanceof NetworkViolationError && error.sockets.trim() !== "") {
      return;
    }
    throw error;
  }
  throw new Error("recursive network canary was not detected");
}

function requireNetworkPlatform(): void {
  if (process.platform !== "darwin") {
    throw new Error("CodeGraph network test requires macOS");
  }
  for (const dependency of ["git", "lsof", "pgrep"]) {
    if (Bun.which(dependency) === null) {
      throw new Error(`${dependency} is required`);
    }
  }
  const selfAudit = Bun.spawnSync(["lsof", "-nP", "-p", String(process.pid)]);
  if (selfAudit.exitCode !== 0) {
    throw new Error("lsof cannot inspect the current process");
  }
}

function message(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}

if (import.meta.main) {
  try {
    process.stdout.write(
      `${JSON.stringify(await runCodeGraphNetworkTest())}\n`,
    );
  } catch (error) {
    process.stderr.write(
      `${error instanceof Error ? error.message : String(error)}\n`,
    );
    process.exit(1);
  }
}

export { runCodeGraphNetworkTest };
