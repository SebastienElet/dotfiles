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

function settingsPath(home: string): string {
  return join(home, ".claude", "settings.json");
}

function writeSettings(home: string, content: string): void {
  mkdirSync(join(home, ".claude"), { recursive: true });
  writeFileSync(settingsPath(home), content);
}

function readSettings(home: string): unknown {
  return JSON.parse(readFileSync(settingsPath(home), "utf8"));
}

test("replaces the registered server and disables native memory while keeping other settings", () => {
  const fixture = createRememFixture();
  writeSettings(fixture.home, '{"theme":"dark","autoMemoryEnabled":true}');

  const result = runUtility(fixture, [
    "remem-claude-install.ts",
    "install",
    fixture.home,
  ]);

  expect(result.exitCode).toBe(0);
  expect(readSettings(fixture.home)).toEqual({
    theme: "dark",
    autoMemoryEnabled: false,
  });
  const [get, remove, add] = readTrace(fixture).map(({ arguments: args }) =>
    args.join(" "),
  );
  expect(get).toBe("mcp get remem");
  expect(remove).toBe("mcp remove --scope user remem");
  expect(add).toStartWith("mcp add --scope user remem -- /bin/sh -c ");
});

test("creates the settings file and skips removal when the server is absent", () => {
  const fixture = createRememFixture();

  const result = runUtility(
    fixture,
    ["remem-claude-install.ts", "install", fixture.home],
    { FAKE_FAIL_ON: "mcp get" },
  );

  expect(result.exitCode).toBe(0);
  expect(readSettings(fixture.home)).toEqual({ autoMemoryEnabled: false });
  expect(readTrace(fixture).map(({ arguments: args }) => args[1])).toEqual([
    "get",
    "add",
  ]);
});

test("rejects invalid settings before running any command", () => {
  const fixture = createRememFixture();
  writeSettings(fixture.home, "[not an object");

  const result = runUtility(fixture, [
    "remem-claude-install.ts",
    "install",
    fixture.home,
  ]);

  expect(result.exitCode).toBe(1);
  expect(result.stderr).toContain("Error:");
  expect(readTrace(fixture)).toEqual([]);
  expect(readFileSync(settingsPath(fixture.home), "utf8")).toBe(
    "[not an object",
  );
});

test("leaves settings untouched when registering the server fails", () => {
  const fixture = createRememFixture();
  writeSettings(fixture.home, '{"autoMemoryEnabled":true}');

  const result = runUtility(
    fixture,
    ["remem-claude-install.ts", "install", fixture.home],
    { FAKE_FAIL_ON: "mcp add" },
  );

  expect(result.exitCode).toBe(1);
  expect(readSettings(fixture.home)).toEqual({ autoMemoryEnabled: true });
});

test("verifies that native memory is disabled", () => {
  const fixture = createRememFixture();
  writeSettings(fixture.home, '{"autoMemoryEnabled":false}');
  expect(
    runUtility(fixture, ["remem-claude-install.ts", "verify", fixture.home])
      .exitCode,
  ).toBe(0);

  writeSettings(fixture.home, '{"autoMemoryEnabled":true}');
  expect(
    runUtility(fixture, ["remem-claude-install.ts", "verify", fixture.home])
      .exitCode,
  ).toBe(1);

  writeSettings(fixture.home, "{}");
  expect(
    runUtility(fixture, ["remem-claude-install.ts", "verify", fixture.home])
      .exitCode,
  ).toBe(1);
});
