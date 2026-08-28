import { afterEach, describe, expect, test } from "bun:test";
import {
  cleanupFixtures,
  createFixture,
  readLog,
  run,
  snapshot,
  start,
} from "./codegraph-configure-test-support.ts";
import { existsSync, readFileSync, statSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { tmpdir } from "node:os";
import { z } from "zod";

const mcpConfigurationSchema = z
  .object({
    mcpServers: z.object({ codegraph: z.unknown() }).loose(),
    unrelated: z.string().optional(),
  })
  .loose();
const configurationErrorExitCode = 2;
const providerFailureExitCode = 9;
const fileModeMask = 0o777;
const privateFileMode = 0o600;
const maximumFilenameLength = 220;
const pauseAttemptLimit = 1000;
const pausePollMilliseconds = 5;

afterEach(cleanupFixtures);

describe("codegraph-configure entry point", () => {
  registerConfigurationTest();
  registerMinimalConfigurationTests();
  registerOutputTests();
  registerValidationTests();
  registerProviderValidationTests();
  registerRollbackTests();
  registerTransactionTest();
});

function registerConfigurationTest(): void {
  test("configures all providers, preserves unrelated data, and is idempotent", () => {
    const fixture = createFixture();
    writeFileSync(fixture.claudeConfig, '{"unrelated":"claude"}\n');
    writeFileSync(fixture.codexConfig, 'unrelated = "codex"\n');
    writeFileSync(
      fixture.cursorConfig,
      '{"unrelated":"cursor","mcpServers":{"existing":{"command":"existing"}}}\n',
    );

    expect(run(fixture).exitCode).toBe(0);
    const first = snapshot(fixture);
    expect(run(fixture).exitCode).toBe(0);
    expect(snapshot(fixture)).toEqual(first);
    expect(
      mcpConfigurationSchema.parse(JSON.parse(first.claude)).unrelated,
    ).toBe("claude");
    expect(first.codex).toContain('unrelated = "codex"');
    expect(JSON.parse(first.cursor)).toEqual({
      mcpServers: {
        codegraph: {
          args: ["serve", "--mcp", "--path", String.raw`\${workspaceFolder}`],
          command: fixture.codegraphBinary,
          env: {
            CODEGRAPH_NO_DOWNLOAD: "1",
            CODEGRAPH_NO_UPDATE_CHECK: "1",
            CODEGRAPH_TELEMETRY: "0",
          },
          type: "stdio",
        },
        existing: { command: "existing" },
      },
      unrelated: "cursor",
    });
    expect(statSync(fixture.cursorConfig).mode & fileModeMask).toBe(
      privateFileMode,
    );
    const calls = readLog(fixture);
    expect(calls).toContain(
      `claude mcp add --scope user codegraph -e CODEGRAPH_TELEMETRY=0 -e CODEGRAPH_NO_UPDATE_CHECK=1 -e CODEGRAPH_NO_DOWNLOAD=1 -- ${fixture.codegraphBinary} serve --mcp`,
    );
    expect(calls).toContain(
      `codex mcp add codegraph --env CODEGRAPH_TELEMETRY=0 --env CODEGRAPH_NO_UPDATE_CHECK=1 --env CODEGRAPH_NO_DOWNLOAD=1 -- ${fixture.codegraphBinary} serve --mcp`,
    );
  });
}

function registerMinimalConfigurationTests(): void {
  test("configures Claude and Codex without creating Cursor configuration", () => {
    const fixture = createFixture({ CODEGRAPH_INCLUDE_CURSOR: "0" });

    expect(run(fixture).exitCode).toBe(0);

    expect(snapshot(fixture).cursor).toBe("<absent>");
    expect(snapshot(fixture).claude).not.toBe("<absent>");
    expect(snapshot(fixture).codex).not.toBe("<absent>");
  });

  test("a converged configuration produces no mutation output", () => {
    const fixture = createFixture({ CODEGRAPH_TEST_EMIT: "1" });
    expect(run(fixture).exitCode).toBe(0);

    const converged = run(fixture);

    expect(converged).toEqual({ exitCode: 0, stderr: "", stdout: "" });
  });
}

function registerOutputTests(): void {
  test("preserves mutation output channels and order", () => {
    const fixture = createFixture({ CODEGRAPH_TEST_EMIT: "1" });
    const result = run(fixture);

    expect(result.exitCode).toBe(0);
    expect(result.stdout).toBe(
      "out codegraph:telemetry off\nout claude:mcp add\nout codex:mcp add\n",
    );
    expect(result.stderr).toBe(
      "err codegraph:telemetry off\nerr claude:mcp add\nerr codex:mcp add\n",
    );
  });

  test("preserves a failed mutation status and output channels", () => {
    const fixture = createFixture({
      CODEGRAPH_TEST_EMIT: "1",
      CODEGRAPH_TEST_FAIL: "claude:mcp add",
    });
    const result = run(fixture);

    expect(result.exitCode).toBe(providerFailureExitCode);
    expect(result.stdout).toBe(
      "out codegraph:telemetry off\nout claude:mcp add\n",
    );
    expect(result.stderr).toBe(
      "err codegraph:telemetry off\nerr claude:mcp add\nfailed claude:mcp add\n",
    );
  });
}

function registerValidationTests(): void {
  test("rejects invalid Cursor JSON before invoking a provider", () => {
    const fixture = createFixture();
    writeFileSync(fixture.cursorConfig, "{invalid\n");

    const result = run(fixture);

    expect(result.exitCode).toBe(configurationErrorExitCode);
    expect(result.stderr).toContain("invalid Cursor MCP JSON");
    expect(readLog(fixture)).toBe("");
  });

  test("accepts a null Cursor server collection", () => {
    const fixture = createFixture();
    writeFileSync(
      fixture.cursorConfig,
      '{"unrelated":"cursor","mcpServers":null}\n',
    );

    expect(run(fixture).exitCode).toBe(0);
    expect(
      mcpConfigurationSchema.parse(
        JSON.parse(readFileSync(fixture.cursorConfig, "utf8")),
      ).mcpServers.codegraph,
    ).toBeDefined();
  });

  test("rejects invalid Claude JSON before invoking a provider", () => {
    const fixture = createFixture();
    writeFileSync(fixture.claudeConfig, "{invalid\n");

    const result = run(fixture);

    expect(result.exitCode).toBe(configurationErrorExitCode);
    expect(result.stderr).toContain("invalid Claude configuration JSON");
    expect(readLog(fixture)).toBe("");
  });
}

function registerProviderValidationTests(): void {
  for (const provider of ["claude", "codex"]) {
    test(`rejects an unexpected ${provider} response before mutation`, () => {
      const fixture = createFixture({
        CODEGRAPH_TEST_UNEXPECTED_GET: provider,
      });
      const before = snapshot(fixture);
      const result = run(fixture);

      expect(result.exitCode).toBe(1);
      expect(result.stderr).toContain("unexpected provider response");
      expect(snapshot(fixture)).toEqual(before);
      expect(readLog(fixture)).not.toContain("mcp add");
    });
  }

  test("rejects a missing dependency before mutation", () => {
    const fixture = createFixture({
      CODEGRAPH_CLAUDE_BIN: join(tmpdir(), "missing-claude"),
    });
    const before = snapshot(fixture);
    const result = run(fixture);

    expect(result.exitCode).toBe(configurationErrorExitCode);
    expect(result.stderr).toContain("missing executable");
    expect(snapshot(fixture)).toEqual(before);
    expect(readLog(fixture)).toBe("");
  });
}

function registerRollbackTests(): void {
  for (const failure of [
    "codegraph:telemetry off",
    "claude:mcp remove",
    "claude:mcp add",
    "codex:mcp remove",
    "codex:mcp add",
  ]) {
    test(`restores every agent configuration after ${failure} fails`, () => {
      const fixture = createFixture({ CODEGRAPH_TEST_FAIL: failure }, true);
      const before = snapshot(fixture);
      const result = run(fixture);

      expect(result.exitCode).toBe(providerFailureExitCode);
      expect(snapshot(fixture)).toEqual(before);
    });
  }

  test("restores native configurations when the Cursor write cannot start", () => {
    const fixture = createFixture({}, true);
    const before = snapshot(fixture);
    fixture.cursorConfig = join(
      fixture.directory,
      "c".repeat(maximumFilenameLength),
    );
    fixture.environment.CODEGRAPH_CURSOR_CONFIG = fixture.cursorConfig;

    const result = run(fixture);

    expect(result.exitCode).not.toBe(0);
    expect(snapshot(fixture)).toEqual({ ...before, cursor: "<absent>" });
    expect(readLog(fixture)).toContain("codex mcp add");
  });
}

function registerTransactionTest(): void {
  test("refuses an overlapping transaction before it can roll back a committed update", async () => {
    const ready = join(tmpdir(), `codegraph-ready-${crypto.randomUUID()}`);
    const release = join(tmpdir(), `codegraph-release-${crypto.randomUUID()}`);
    const fixture = createFixture({
      CODEGRAPH_TEST_PAUSE: "claude:mcp add",
      CODEGRAPH_TEST_PAUSE_READY: ready,
      CODEGRAPH_TEST_PAUSE_RELEASE: release,
    });
    const first = start(fixture);
    for (
      let attempt = 0;
      attempt < pauseAttemptLimit && !existsSync(ready);
      attempt += 1
    ) {
      Bun.sleepSync(pausePollMilliseconds);
    }
    const competingFixture = {
      ...fixture,
      environment: {
        ...fixture.environment,
        CODEGRAPH_TEST_FAIL: "codex:mcp add",
        CODEGRAPH_TEST_PAUSE: "",
      },
    };

    expect(existsSync(ready)).toBe(true);
    const competing = runAndRelease(competingFixture, release);

    expect(competing.exitCode).toBe(configurationErrorExitCode);
    expect(competing.stderr).toContain(
      "configuration update already in progress",
    );
    expect(await first.exited).toBe(0);
    const configurations = snapshot(fixture);
    expect(
      mcpConfigurationSchema.parse(JSON.parse(configurations.claude)).mcpServers
        .codegraph,
    ).toBeDefined();
    expect(configurations.codex).toContain("codex:added");
    expect(
      mcpConfigurationSchema.parse(JSON.parse(configurations.cursor)).mcpServers
        .codegraph,
    ).toBeDefined();
  });
}

function runAndRelease(
  fixture: Parameters<typeof run>[0],
  release: string,
): ReturnType<typeof run> {
  try {
    return run(fixture);
  } finally {
    writeFileSync(release, "release\n");
  }
}
