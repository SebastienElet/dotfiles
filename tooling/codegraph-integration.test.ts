import { describe, expect, test } from "bun:test";
import { existsSync } from "node:fs";
import { join } from "node:path";
import { z } from "zod";
import {
  cleanupFreshnessFixture,
  createFreshnessFixture,
} from "./codegraph/integration-fixture.ts";

const provider = join(import.meta.dir, "codegraph", "mcp-test-provider.ts");
const probeEntryPoint = join(import.meta.dir, "codegraph-mcp-probe.ts");
const repository = import.meta.dir;
const mcpReportSchema = z.object({
  environment: z.object({
    os: z.literal("macOS"),
    linuxExercised: z.literal(false),
  }),
  initialIndexSeconds: z.number().nonnegative(),
  initialIndexMaxRssBytes: z.number().positive(),
  initialIndexCpuUserSeconds: z.number().nonnegative(),
  initialIndexCpuSystemSeconds: z.number().nonnegative(),
  indexDiskKiB: z.number().positive(),
  mcp: z.object({
    tools: z.tuple([z.literal("codegraph_explore")]),
    scenarios: z.object({
      initial: z.literal(true),
      branchSwitch: z.literal(true),
      edit: z.literal(true),
      rename: z.literal(true),
      delete: z.literal(true),
      restart: z.literal(true),
      watcherInterruption: z.enum(["fresh", "alerted-stale"]),
      reconciliation: z.literal(true),
      daemonStopped: z.literal(true),
    }),
    queryLatencyMs: z.record(z.string(), z.number().nonnegative()),
    synchronizationMs: z.number().nonnegative(),
  }),
});

describe("CodeGraph MCP failure paths", () => {
  test.each([
    ["startup", "1000", /stopped before replying|status=42/],
    ["timeout", "100", /timed out/],
    ["malformed", "1000", /malformed MCP response/],
  ])("fails explicitly on %s", (scenario, requestTimeout, expected) => {
    const result = Bun.spawnSync([probeEntryPoint, repository], {
      env: {
        ...process.env,
        CODEGRAPH_BIN: provider,
        CODEGRAPH_MCP_TEST_SCENARIO: scenario,
        CODEGRAPH_MCP_REQUEST_TIMEOUT_MS: requestTimeout,
        CODEGRAPH_MCP_STOP_TIMEOUT_MS: "100",
      },
      stdout: "pipe",
      stderr: "pipe",
    });

    expect(result.exitCode).not.toBe(0);
    expect(result.stdout.toString()).toBe("");
    expect(result.stderr.toString()).toMatch(expected);
  });

  test("reports cleanup failure after removing the isolated fixture", async () => {
    const fixture = createFreshnessFixture(
      join(import.meta.dir, "codegraph", "fixtures", "freshness"),
    );

    await expect(
      cleanupFreshnessFixture(fixture, [process.execPath, provider]),
    ).rejects.toThrow(/cleanup failed/);
    expect(existsSync(fixture.root)).toBeFalse();
  });
});

test.skipIf(process.env.CODEGRAPH_INTEGRATION !== "1")(
  "the real entry point proves MCP freshness and metrics",
  () => {
    const result = Bun.spawnSync(
      [join(import.meta.dir, "codegraph-mcp-test")],
      {
        stdout: "pipe",
        stderr: "pipe",
      },
    );
    expect(result.exitCode, result.stderr.toString()).toBe(0);
    const report = mcpReportSchema.parse(JSON.parse(result.stdout.toString()));
    expect(report.environment).toEqual({ os: "macOS", linuxExercised: false });
    expect(report.mcp.tools).toEqual(["codegraph_explore"]);
    expect(Object.keys(report.mcp.queryLatencyMs)).toHaveLength(9);
    expect(report.initialIndexSeconds).toBeGreaterThanOrEqual(0);
    expect(report.initialIndexMaxRssBytes).toBeGreaterThan(0);
    expect(report.indexDiskKiB).toBeGreaterThan(0);
  },
  180_000,
);

test.skipIf(
  process.env.CODEGRAPH_INTEGRATION !== "1" || process.platform !== "darwin",
)(
  "the real network entry point detects its canary then observes no CodeGraph socket",
  () => {
    const result = Bun.spawnSync(
      [join(import.meta.dir, "codegraph-network-test")],
      {
        stdout: "pipe",
        stderr: "pipe",
      },
    );
    expect(result.exitCode, result.stderr.toString()).toBe(0);
    const report = JSON.parse(result.stdout.toString());
    expect(report).toEqual({
      sampleIntervalMs: 50,
      descendantsRecursive: true,
      phases: { init: true, sync: true, mcp: true },
      networkSocketsObserved: 0,
      daemonStopped: true,
    });
  },
  180_000,
);
