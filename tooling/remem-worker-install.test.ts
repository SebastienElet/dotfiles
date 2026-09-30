import { afterEach, expect, test } from "bun:test";
import {
  cleanupRememFixtures,
  createRememFixture,
  readTrace,
  runUtility,
} from "./remem-test-support.ts";
import { existsSync, readFileSync } from "node:fs";
import { join } from "node:path";

afterEach(cleanupRememFixtures);

const label = "dev.remem.worker";

function plistPath(home: string): string {
  return join(home, "Library", "LaunchAgents", `${label}.plist`);
}

function domain(): string {
  return `gui/${process.getuid?.()}`;
}

test("writes the agent definition and reloads a loaded service", () => {
  const fixture = createRememFixture();

  const result = runUtility(fixture, ["remem-worker-install.ts", fixture.home]);

  expect(result.exitCode).toBe(0);
  const plist = readFileSync(plistPath(fixture.home), "utf8");
  expect(plist).toContain(`<key>Label</key>\n\t<string>${label}</string>`);
  expect(plist).toContain(`<string>${fixture.home}/.local/bin/remem</string>`);
  expect(plist).toContain("<string>worker</string>");
  expect(plist).toContain("<string>--once</string>");
  expect(plist).toContain("<key>RunAtLoad</key>\n\t<true/>");
  expect(plist).toContain("<key>StartInterval</key>\n\t<integer>300</integer>");
  expect(plist).toContain("<string>Background</string>");
  expect(readTrace(fixture)).toEqual([
    { command: "launchctl", arguments: ["print", `${domain()}/${label}`] },
    { command: "launchctl", arguments: ["bootout", `${domain()}/${label}`] },
    {
      command: "launchctl",
      arguments: ["bootstrap", domain(), plistPath(fixture.home)],
    },
  ]);
});

test("bootstraps without booting out an unloaded service", () => {
  const fixture = createRememFixture();

  const result = runUtility(
    fixture,
    ["remem-worker-install.ts", fixture.home],
    { FAKE_FAIL_ON: "print" },
  );

  expect(result.exitCode).toBe(0);
  expect(readTrace(fixture).map(({ arguments: args }) => args[0])).toEqual([
    "print",
    "bootstrap",
  ]);
});

test("escapes markup in the home directory", () => {
  const fixture = createRememFixture();
  const home = `${fixture.home}/a&b<c>`;

  const result = runUtility(fixture, ["remem-worker-install.ts", home]);

  expect(result.exitCode).toBe(0);
  expect(readFileSync(plistPath(home), "utf8")).toContain(
    `${fixture.home}/a&amp;b&lt;c&gt;/.local/bin/remem`,
  );
});

test("reports a bootstrap failure and keeps the written definition", () => {
  const fixture = createRememFixture();

  const result = runUtility(
    fixture,
    ["remem-worker-install.ts", fixture.home],
    { FAKE_FAIL_ON: "bootstrap" },
  );

  expect(result.exitCode).toBe(1);
  expect(existsSync(plistPath(fixture.home))).toBeTrue();
});
