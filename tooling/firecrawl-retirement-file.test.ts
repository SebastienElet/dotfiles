import { afterEach, expect, test } from "bun:test";
import {
  cleanupFixtures,
  createFixture,
  readJson,
  run,
} from "./firecrawl-retirement-test-support.ts";
import {
  inspectFileVersion,
  recoverInterruptedFileReplacement,
  replaceFileVersion,
} from "./firecrawl-retirement-file.ts";
import {
  linkSync,
  readFileSync,
  readdirSync,
  renameSync,
  writeFileSync,
} from "node:fs";
import { dirname } from "node:path";

afterEach(cleanupFixtures);

test("recovery is a no-op when the configuration directory is absent", () => {
  const fixture = createFixture("absent");

  expect(() => {
    recoverInterruptedFileReplacement(
      `${fixture.root}/missing/.cursor/mcp.json`,
    );
  }).not.toThrow();
});

test("exclusive publication preserves a configuration created during update", () => {
  const fixture = createFixture("absent");
  writeFileSync(fixture.claudeConfig, "original\n");
  const expected = inspectFileVersion(fixture.claudeConfig);

  expect(() =>
    replaceFileVersion({
      beforePublish: () => {
        writeFileSync(fixture.claudeConfig, "concurrent\n");
      },
      content: [...Buffer.from("updated\n")],
      expected,
      label: "Claude",
      mode: expected.mode,
      path: fixture.claudeConfig,
      phase: "during retirement",
    }),
  ).toThrow("Claude configuration changed during retirement");
  expect(readFileSync(fixture.claudeConfig, "utf8")).toBe("concurrent\n");
});

test("exclusive rollback preserves a configuration created after update", () => {
  const fixture = createFixture("absent");
  writeFileSync(fixture.claudeConfig, "updated\n");
  const expected = inspectFileVersion(fixture.claudeConfig);

  expect(() =>
    replaceFileVersion({
      beforePublish: () => {
        writeFileSync(fixture.claudeConfig, "concurrent\n");
      },
      content: [...Buffer.from("original\n")],
      expected,
      label: "Claude",
      mode: expected.mode,
      path: fixture.claudeConfig,
      phase: "after update",
    }),
  ).toThrow("Claude configuration changed after update");
  expect(readFileSync(fixture.claudeConfig, "utf8")).toBe("concurrent\n");
});

test("a publication error restores the displaced configuration", () => {
  const fixture = createFixture("absent");
  writeFileSync(fixture.claudeConfig, "original\n");
  const expected = inspectFileVersion(fixture.claudeConfig);
  const publicationError = Object.assign(new Error("publication failed"), {
    code: "EIO",
  });

  expect(() =>
    replaceFileVersion({
      content: [...Buffer.from("updated\n")],
      expected,
      label: "Claude",
      mode: expected.mode,
      path: fixture.claudeConfig,
      phase: "during retirement",
      publish: () => {
        throw publicationError;
      },
    }),
  ).toThrow("publication failed");
  expect(readFileSync(fixture.claudeConfig, "utf8")).toBe("original\n");
  expect(readdirSync(dirname(fixture.claudeConfig)).join("\n")).not.toContain(
    "firecrawl-retirement.old",
  );
});

test("the next run recovers an interruption after displacing a configuration", () => {
  const fixture = createFixture("absent");
  writeAgentConfiguration(fixture.claudeConfig);
  const suffix = "interrupted";
  renameSync(
    fixture.claudeConfig,
    `${fixture.claudeConfig}.firecrawl-retirement.old.${suffix}`,
  );
  writeFileSync(
    `${fixture.claudeConfig}.firecrawl-retirement.new.${suffix}`,
    "partial\n",
  );

  const result = run(fixture);

  expectRecoveredConfiguration(fixture.claudeConfig, result.exitCode);
});

test("the next run recovers an interruption after publishing a configuration", () => {
  const fixture = createFixture("absent");
  writeAgentConfiguration(fixture.claudeConfig);
  const suffix = "published";
  const displaced = `${fixture.claudeConfig}.firecrawl-retirement.old.${suffix}`;
  const publication = `${fixture.claudeConfig}.firecrawl-retirement.new.${suffix}`;
  renameSync(fixture.claudeConfig, displaced);
  writeAgentConfiguration(publication);
  linkSync(publication, fixture.claudeConfig);

  const result = run(fixture);

  expectRecoveredConfiguration(fixture.claudeConfig, result.exitCode);
});

function writeAgentConfiguration(path: string): void {
  writeFileSync(path, `${JSON.stringify(existingConfiguration())}\n`);
}

function expectRecoveredConfiguration(path: string, exitCode: number): void {
  expect(exitCode).toBe(0);
  expect(readJson(path)).toEqual({
    mcpServers: { unrelated: { command: "unrelated" } },
    unrelated: "claude",
  });
  expect(readdirSync(dirname(path)).join("\n")).not.toContain(
    "firecrawl-retirement",
  );
}

function existingConfiguration(): Readonly<Record<string, unknown>> {
  return {
    mcpServers: {
      firecrawl: { args: ["run", "firecrawl"], command: "docker" },
      unrelated: { command: "unrelated" },
    },
    unrelated: "claude",
  };
}
