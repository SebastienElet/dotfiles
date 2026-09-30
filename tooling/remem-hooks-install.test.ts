import { afterEach, expect, test } from "bun:test";
import {
  cleanupRememFixtures,
  createRememFixture,
  readTrace,
  runUtility,
} from "./remem-test-support.ts";
import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";

afterEach(cleanupRememFixtures);

const claudeHookCount = 6;
const codexHookCount = 3;

function writeFile(path: string, content: string): void {
  mkdirSync(join(path, ".."), { recursive: true });
  writeFileSync(path, content);
}

function hooksDocument(commands: readonly string[]): string {
  return JSON.stringify({
    hooks: {
      Event: commands.map((command) => ({ hooks: [{ command }] })),
    },
  });
}

function rememCommands(home: string, count: number): readonly string[] {
  return Array.from({ length: count }, () => `'${home}/.local/bin/remem' hook`);
}

test("installs both agent hooks and restores the tracked configuration", () => {
  const fixture = createRememFixture();
  const configuration = join(fixture.home, ".remem", "config.toml");
  writeFile(configuration, "tracked = true\n");

  const result = runUtility(
    fixture,
    ["remem-hooks-install.ts", "install", fixture.home],
    { FAKE_APPEND_TO: configuration },
  );

  expect(result.exitCode).toBe(0);
  expect(readFileSync(configuration, "utf8")).toBe("tracked = true\n");
  expect(readTrace(fixture).map(({ arguments: args }) => args)).toEqual([
    ["install", "--target", "claude", "--hooks-only"],
    ["install", "--target", "codex", "--hooks-only"],
  ]);
});

test("restores the tracked configuration when an installation fails", () => {
  const fixture = createRememFixture();
  const configuration = join(fixture.home, ".remem", "config.toml");
  writeFile(configuration, "tracked = true\n");

  const result = runUtility(
    fixture,
    ["remem-hooks-install.ts", "install", fixture.home],
    { FAKE_APPEND_TO: configuration, FAKE_FAIL_ON: "target codex" },
  );

  expect(result.exitCode).toBe(1);
  expect(readFileSync(configuration, "utf8")).toBe("tracked = true\n");
});

test("refuses to install without the deployed configuration", () => {
  const fixture = createRememFixture();

  const result = runUtility(fixture, [
    "remem-hooks-install.ts",
    "install",
    fixture.home,
  ]);

  expect(result.exitCode).toBe(1);
  expect(readTrace(fixture)).toEqual([]);
});

test("verifies the registered hook counts of both agents", () => {
  const fixture = createRememFixture();
  const claude = join(fixture.home, ".claude", "settings.json");
  const codex = join(fixture.home, ".codex", "hooks.json");
  const verify = (): number =>
    runUtility(fixture, ["remem-hooks-install.ts", "verify", fixture.home])
      .exitCode;

  writeFile(
    claude,
    hooksDocument([
      ...rememCommands(fixture.home, claudeHookCount),
      "'other' hook",
    ]),
  );
  writeFile(codex, hooksDocument(rememCommands(fixture.home, codexHookCount)));
  expect(verify()).toBe(0);

  writeFile(
    codex,
    hooksDocument(rememCommands(fixture.home, codexHookCount - 1)),
  );
  expect(verify()).toBe(1);

  writeFile(codex, hooksDocument(rememCommands(fixture.home, codexHookCount)));
  writeFile(
    claude,
    hooksDocument(rememCommands(fixture.home, claudeHookCount - 1)),
  );
  expect(verify()).toBe(1);

  writeFile(claude, "{}");
  expect(verify()).toBe(1);
});
