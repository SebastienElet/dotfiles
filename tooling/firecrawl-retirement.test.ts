import { afterEach, expect, test } from "bun:test";
import {
  cleanupFixtures,
  createFixture,
  readJson,
  readLog,
  run,
} from "./firecrawl-retirement-test-support.ts";
import { mkdirSync, writeFileSync } from "node:fs";
import { dirname } from "node:path";

afterEach(cleanupFixtures);

test("a fresh installation creates no Firecrawl configuration or Docker artifact", () => {
  const fixture = createFixture("absent");

  const result = run(fixture);

  expect(result.exitCode).toBe(0);
  expect(result.stderr).toBe("");
  expect(readLog(fixture)).not.toContain("container rm");
  expect(readLog(fixture)).not.toContain("volume rm");
  expect(readLog(fixture)).not.toContain("mcp remove firecrawl");
});

test("an existing installation removes Firecrawl registrations, containers, and volumes", () => {
  const fixture = createFixture("existing");
  writeAgentConfigurations(fixture);

  const result = run(fixture);

  expect(result.exitCode).toBe(0);
  expect(result.stderr).toBe("");
  expect(readJson(fixture.claudeConfig)).toEqual({
    mcpServers: { unrelated: { command: "unrelated" } },
    unrelated: "claude",
  });
  expect(readJson(fixture.cursorConfig)).toEqual({
    mcpServers: { unrelated: { command: "unrelated" } },
    unrelated: "cursor",
  });
  expect(readLog(fixture)).toContain("codex mcp remove firecrawl\n");
  expect(readLog(fixture)).toContain(
    "docker container rm --force --volumes -- aaaaaaaaaaaa bbbbbbbbbbbb\n",
  );
  expect(readLog(fixture)).toContain(
    "docker volume rm -- named-firecrawl-volume\n",
  );
  expect(result.stdout).toContain(
    "retained-images cleanup: docker image rm -- ghcr.io/firecrawl/firecrawl:latest ghcr.io/firecrawl/nuq-postgres:latest ghcr.io/firecrawl/playwright-service:latest rabbitmq:3-management redis:alpine",
  );
});

test("orphaned Firecrawl images are reported without containers", () => {
  const fixture = createFixture("images-only");

  const result = run(fixture);

  expect(result.exitCode).toBe(0);
  expect(readLog(fixture)).not.toContain("container rm");
  expect(result.stdout).toContain(
    "retained-images cleanup: docker image rm -- ghcr.io/firecrawl/firecrawl:latest ghcr.io/firecrawl/nuq-postgres:latest ghcr.io/firecrawl/playwright-service:latest rabbitmq:3-management redis:alpine",
  );
});

test("malformed Docker evidence prevents every configuration mutation", () => {
  const fixture = createFixture("malformed-docker");
  writeAgentConfigurations(fixture);

  const result = run(fixture);

  expect(result.exitCode).not.toBe(0);
  expect(result.stderr).toContain("invalid Docker container evidence");
  expect(readJson(fixture.claudeConfig)).toEqual(
    existingConfiguration("claude"),
  );
  expect(readJson(fixture.cursorConfig)).toEqual(
    existingConfiguration("cursor"),
  );
  expect(readLog(fixture)).not.toContain("mcp remove firecrawl");
});

test("invalid agent JSON prevents Docker and Codex mutation", () => {
  const fixture = createFixture("existing");
  writeFileSync(fixture.claudeConfig, "{invalid\n");

  const result = run(fixture);

  expect(result.exitCode).not.toBe(0);
  expect(result.stderr).toContain("invalid Claude configuration JSON");
  expect(readLog(fixture)).not.toContain("container rm");
  expect(readLog(fixture)).not.toContain("mcp remove firecrawl");
});

test("a required unavailable Docker daemon leaves configurations unchanged", () => {
  const fixture = createFixture("daemon-unavailable");
  writeAgentConfigurations(fixture);

  const result = run(fixture);

  expect(result.exitCode).not.toBe(0);
  expect(result.stderr).toContain("Docker daemon unavailable");
  expect(readJson(fixture.claudeConfig)).toEqual(
    existingConfiguration("claude"),
  );
  expect(readJson(fixture.cursorConfig)).toEqual(
    existingConfiguration("cursor"),
  );
});

test("a concurrent configuration update is never overwritten", () => {
  const fixture = createFixture("concurrent-configuration");
  writeAgentConfigurations(fixture);

  const result = run(fixture);

  expect(result.exitCode).not.toBe(0);
  expect(result.stderr).toContain(
    "Claude configuration changed during retirement",
  );
  expect(readJson(fixture.claudeConfig)).toEqual({
    ...existingConfiguration("claude"),
    concurrent: true,
  });
  expect(readLog(fixture)).not.toContain("container rm");
});

test("rollback never overwrites a later concurrent configuration update", () => {
  const fixture = createFixture("rollback-concurrent-configuration");
  writeAgentConfigurations(fixture);

  const result = run(fixture);

  expect(result.exitCode).not.toBe(0);
  expect(result.stderr).toContain("Claude configuration changed after update");
  expect(readJson(fixture.claudeConfig)).toEqual({
    concurrent: true,
    mcpServers: { unrelated: { command: "unrelated" } },
    unrelated: "claude",
  });
  expect(readLog(fixture)).not.toContain("container rm");
});

test("persistent Docker artifacts prevent a successful removal claim", () => {
  const fixture = createFixture("persistent-docker");

  const result = run(fixture);

  expect(result.exitCode).not.toBe(0);
  expect(result.stderr).toContain("Firecrawl Docker artifacts remain");
  expect(result.stdout).not.toContain("result=removed");
});

function writeAgentConfigurations(
  fixture: ReturnType<typeof createFixture>,
): void {
  mkdirSync(dirname(fixture.claudeConfig), { recursive: true });
  mkdirSync(dirname(fixture.cursorConfig), { recursive: true });
  writeFileSync(
    fixture.claudeConfig,
    `${JSON.stringify(existingConfiguration("claude"))}\n`,
  );
  writeFileSync(
    fixture.cursorConfig,
    `${JSON.stringify(existingConfiguration("cursor"))}\n`,
  );
}

function existingConfiguration(
  agent: string,
): Readonly<Record<string, unknown>> {
  return {
    mcpServers: {
      firecrawl: { args: ["run", "firecrawl"], command: "docker" },
      unrelated: { command: "unrelated" },
    },
    unrelated: agent,
  };
}
