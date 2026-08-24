import {
  cleanupFreshnessFixture,
  createFreshnessFixture,
} from "./codegraph/integration-fixture.ts";
import { describe, expect, test } from "bun:test";
import { existsSync } from "node:fs";
import { join } from "node:path";
import { z } from "zod";

const provider = join(import.meta.dir, "codegraph", "mcp-test-provider.ts");
const probeEntryPoint = join(import.meta.dir, "codegraph-mcp-probe.ts");
const repository = import.meta.dir;
const expectedQueryCount = 9;
const integrationTimeoutMilliseconds = 180_000;
const mcpReportSchema = z.object({
  environment: z.object({
    linuxExercised: z.literal(false),
    os: z.literal("macOS"),
  }),
  indexDiskKiB: z.number().positive(),
  initialIndexCpuSystemSeconds: z.number().nonnegative(),
  initialIndexCpuUserSeconds: z.number().nonnegative(),
  initialIndexMaxRssBytes: z.number().positive(),
  initialIndexSeconds: z.number().nonnegative(),
  mcp: z.object({
    queryLatencyMs: z.record(z.string(), z.number().nonnegative()),
    scenarios: z.object({
      branchSwitch: z.literal(true),
      daemonStopped: z.literal(true),
      delete: z.literal(true),
      edit: z.literal(true),
      initial: z.literal(true),
      reconciliation: z.literal(true),
      rename: z.literal(true),
      restart: z.literal(true),
      watcherInterruption: z.enum(["fresh", "alerted-stale"]),
    }),
    synchronizationMs: z.number().nonnegative(),
    tools: z.tuple([z.literal("codegraph_explore")]),
  }),
});

describe("CodeGraph MCP failure paths", () => {
  test.each([
    ["startup", "1000", /stopped before replying|status=42/u],
    ["timeout", "100", /timed out/u],
    ["malformed", "1000", /malformed MCP response/u],
  ])(
    "fails explicitly on %s",
    (scenario, requestTimeout, expected: Readonly<RegExp>) => {
      const result = Bun.spawnSync([probeEntryPoint, repository], {
        env: {
          ...process.env,
          CODEGRAPH_BIN: provider,
          CODEGRAPH_MCP_REQUEST_TIMEOUT_MS: requestTimeout,
          CODEGRAPH_MCP_STOP_TIMEOUT_MS: "100",
          CODEGRAPH_MCP_TEST_SCENARIO: scenario,
        },
        stderr: "pipe",
        stdout: "pipe",
      });

      expect(result.exitCode).not.toBe(0);
      expect(result.stdout.toString()).toBe("");
      expect(result.stderr.toString()).toMatch(expected);
    },
  );

  test("reports cleanup failure after removing the isolated fixture", async () => {
    const fixture = createFreshnessFixture(
      join(import.meta.dir, "codegraph", "fixtures", "freshness"),
    );

    const cleanupError = await captureFailure(() =>
      cleanupFreshnessFixture(fixture, [process.execPath, provider]),
    );
    expect(String(cleanupError)).toMatch(/cleanup failed/u);
    expect(existsSync(fixture.root)).toBeFalse();
  });
});

test.skipIf(process.env.CODEGRAPH_INTEGRATION !== "1")(
  "the real entry point proves MCP freshness and metrics",
  () => {
    const result = Bun.spawnSync(
      [join(import.meta.dir, "codegraph-mcp-test")],
      {
        stderr: "pipe",
        stdout: "pipe",
      },
    );
    expect(result.exitCode, result.stderr.toString()).toBe(0);
    const report = mcpReportSchema.parse(JSON.parse(result.stdout.toString()));
    expect(report.environment).toEqual({ linuxExercised: false, os: "macOS" });
    expect(report.mcp.tools).toEqual(["codegraph_explore"]);
    expect(Object.keys(report.mcp.queryLatencyMs)).toHaveLength(
      expectedQueryCount,
    );
    expect(report.initialIndexSeconds).toBeGreaterThanOrEqual(0);
    expect(report.initialIndexMaxRssBytes).toBeGreaterThan(0);
    expect(report.indexDiskKiB).toBeGreaterThan(0);
  },
  integrationTimeoutMilliseconds,
);

async function captureFailure(
  operation: () => Promise<void>,
): Promise<unknown> {
  try {
    await operation();
    return undefined;
  } catch (error) {
    return error;
  }
}

test.skipIf(
  process.env.CODEGRAPH_INTEGRATION !== "1" || process.platform !== "darwin",
)(
  "the real network entry point detects its canary then observes no CodeGraph socket",
  () => {
    const result = Bun.spawnSync(
      [join(import.meta.dir, "codegraph-network-test")],
      {
        stderr: "pipe",
        stdout: "pipe",
      },
    );
    expect(result.exitCode, result.stderr.toString()).toBe(0);
    const report = z.unknown().parse(JSON.parse(result.stdout.toString()));
    expect(report).toEqual({
      daemonStopped: true,
      descendantsRecursive: true,
      networkSocketsObserved: 0,
      phases: { init: true, mcp: true, sync: true },
      sampleIntervalMs: 50,
    });
  },
  integrationTimeoutMilliseconds,
);
