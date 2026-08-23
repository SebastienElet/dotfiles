import { afterEach, describe, expect, test } from "bun:test";
import {
  lstatSync,
  readFileSync,
  statSync,
  symlinkSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import {
  cleanupFixtures,
  createFixture,
  readLog,
  run,
  snapshot,
} from "./codegraph-configure-test-support.ts";

afterEach(cleanupFixtures);

describe("codegraph-configure entry point", () => {
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
    expect(JSON.parse(first.claude).unrelated).toBe("claude");
    expect(first.codex).toContain('unrelated = "codex"');
    expect(JSON.parse(first.cursor)).toEqual({
      unrelated: "cursor",
      mcpServers: {
        existing: { command: "existing" },
        codegraph: {
          type: "stdio",
          command: fixture.codegraphBinary,
          args: ["serve", "--mcp", "--path", "${workspaceFolder}"],
          env: {
            CODEGRAPH_TELEMETRY: "0",
            CODEGRAPH_NO_UPDATE_CHECK: "1",
            CODEGRAPH_NO_DOWNLOAD: "1",
          },
        },
      },
    });
    expect(statSync(fixture.cursorConfig).mode & 0o777).toBe(0o600);
    const calls = readLog(fixture);
    expect(calls).toContain(
      `claude mcp add --scope user codegraph -e CODEGRAPH_TELEMETRY=0 -e CODEGRAPH_NO_UPDATE_CHECK=1 -e CODEGRAPH_NO_DOWNLOAD=1 -- ${fixture.codegraphBinary} serve --mcp`,
    );
    expect(calls).toContain(
      `codex mcp add codegraph --env CODEGRAPH_TELEMETRY=0 --env CODEGRAPH_NO_UPDATE_CHECK=1 --env CODEGRAPH_NO_DOWNLOAD=1 -- ${fixture.codegraphBinary} serve --mcp`,
    );
  });

  test("rejects invalid Cursor JSON before invoking a provider", () => {
    const fixture = createFixture();
    writeFileSync(fixture.cursorConfig, "{invalid\n");

    const result = run(fixture);

    expect(result.exitCode).toBe(2);
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
      JSON.parse(readFileSync(fixture.cursorConfig, "utf8")).mcpServers
        .codegraph,
    ).toBeDefined();
  });

  test("rejects invalid Claude JSON before invoking a provider", () => {
    const fixture = createFixture();
    writeFileSync(fixture.claudeConfig, "{invalid\n");

    const result = run(fixture);

    expect(result.exitCode).toBe(2);
    expect(result.stderr).toContain("invalid Claude configuration JSON");
    expect(readLog(fixture)).toBe("");
  });

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

    expect(result.exitCode).toBe(2);
    expect(result.stderr).toContain("missing executable");
    expect(snapshot(fixture)).toEqual(before);
    expect(readLog(fixture)).toBe("");
  });

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

      expect(result.exitCode).toBe(9);
      expect(snapshot(fixture)).toEqual(before);
    });
  }

  test("restores native configurations when the Cursor write cannot start", () => {
    const fixture = createFixture({}, true);
    const before = snapshot(fixture);
    const blockingParent = join(fixture.directory, "cursor-parent");
    writeFileSync(blockingParent, "not a directory\n");
    fixture.cursorConfig = join(blockingParent, "mcp.json");
    fixture.environment.CODEGRAPH_CURSOR_CONFIG = fixture.cursorConfig;

    const result = run(fixture);

    expect(result.exitCode).not.toBe(0);
    expect(snapshot(fixture)).toEqual({ ...before, cursor: "<absent>" });
  });

  test("rejects symlinked configuration paths before mutation", () => {
    const fixture = createFixture();
    const target = join(fixture.directory, "cursor-target.json");
    writeFileSync(target, "{}\n");
    symlinkSync(target, fixture.cursorConfig);

    const result = run(fixture);

    expect(result.exitCode).toBe(2);
    expect(result.stderr).toContain("must not be a symlink");
    expect(lstatSync(fixture.cursorConfig).isSymbolicLink()).toBe(true);
    expect(readLog(fixture)).toBe("");
  });
});
