import { join } from "node:path";
import { z } from "zod";
import {
  cleanupFreshnessFixture,
  createFreshnessFixture,
  privacyEnvironment,
  type Command,
} from "./codegraph/integration-fixture.ts";
import {
  auditProcessNetwork,
  NetworkViolation,
} from "./codegraph/network-audit.ts";

const probeSchema = z.object({
  scenarios: z.object({ daemonStopped: z.literal(true) }),
});

export async function runCodeGraphNetworkTest() {
  requireNetworkPlatform();
  const binary = process.env.CODEGRAPH_BIN ?? Bun.which("codegraph");
  if (binary === null || binary === undefined)
    throw new Error("codegraph is required");
  const codegraph: Command = [binary];
  await proveNetworkCanary();
  const fixture = createFreshnessFixture(
    join(import.meta.dir, "codegraph", "fixtures", "freshness"),
  );
  let operationError: unknown;
  try {
    await auditProcessNetwork(
      [...codegraph, "init", fixture.repository],
      privacyEnvironment,
      fixture.repository,
    );
    await auditProcessNetwork(
      [...codegraph, "sync", fixture.repository],
      privacyEnvironment,
      fixture.repository,
    );
    const probe = await auditProcessNetwork(
      [
        process.execPath,
        join(import.meta.dir, "codegraph-mcp-probe.ts"),
        fixture.repository,
      ],
      {
        ...privacyEnvironment,
        CODEGRAPH_BIN: binary,
        CODEGRAPH_PROBE_PAUSE_MS: "500",
      },
      fixture.repository,
    );
    probeSchema.parse(JSON.parse(probe.stdout));
    return {
      sampleIntervalMs: 50,
      descendantsRecursive: true,
      phases: { init: true, sync: true, mcp: true },
      networkSocketsObserved: 0,
      daemonStopped: true,
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
        `network test failed: ${message(operationError)}; cleanup failed: ${message(cleanupError)}`,
      );
    }
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
      process.env,
      undefined,
      10_000,
    );
  } catch (error) {
    if (error instanceof NetworkViolation && error.sockets.trim() !== "")
      return;
    throw error;
  }
  throw new Error("recursive network canary was not detected");
}

function requireNetworkPlatform(): void {
  if (process.platform !== "darwin")
    throw new Error("CodeGraph network test requires macOS");
  for (const dependency of ["git", "lsof", "pgrep"]) {
    if (Bun.which(dependency) === null)
      throw new Error(`${dependency} is required`);
  }
  const selfAudit = Bun.spawnSync(["lsof", "-nP", "-p", String(process.pid)]);
  if (selfAudit.exitCode !== 0)
    throw new Error("lsof cannot inspect the current process");
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
