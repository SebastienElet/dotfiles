import { afterEach, describe, expect, test } from "bun:test";
import {
  cleanupFixtures,
  createFixture,
  readLog,
  run,
  snapshot,
} from "./codegraph-configure-test-support.ts";
import { linkSync, lstatSync, symlinkSync, writeFileSync } from "node:fs";
import { join } from "node:path";

const configurationErrorExitCode = 2;

afterEach(cleanupFixtures);

describe("codegraph-configure file boundaries", () => {
  test("fails closed when an existing lock prevents serialization", () => {
    const fixture = createFixture();
    writeFileSync(
      `${fixture.claudeConfig}.codegraph-configure.lock`,
      `${process.pid}\n`,
    );
    const before = snapshot(fixture);
    const result = run(fixture);

    expect(result.exitCode).toBe(configurationErrorExitCode);
    expect(result.stderr).toContain("configuration update already in progress");
    expect(snapshot(fixture)).toEqual(before);
    expect(readLog(fixture)).toBe("");
  });

  test("rejects symlinked configuration paths before mutation", () => {
    const fixture = createFixture();
    const target = join(fixture.directory, "cursor-target.json");
    writeFileSync(target, "{}\n");
    symlinkSync(target, fixture.cursorConfig);

    const result = run(fixture);

    expect(result.exitCode).toBe(configurationErrorExitCode);
    expect(result.stderr).toContain("must not be a symlink");
    expect(lstatSync(fixture.cursorConfig).isSymbolicLink()).toBe(true);
    expect(readLog(fixture)).toBe("");
  });

  test("rejects hard-linked configuration paths before mutation", () => {
    const fixture = createFixture();
    const alias = join(fixture.directory, "cursor-alias.json");
    writeFileSync(alias, "{}\n");
    linkSync(alias, fixture.cursorConfig);

    const result = run(fixture);

    expect(result.exitCode).toBe(configurationErrorExitCode);
    expect(result.stderr).toContain("must not have multiple hard links");
    expect(readLog(fixture)).toBe("");
  });
});
