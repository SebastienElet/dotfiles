import { afterEach, expect, test } from "bun:test";
import {
  cleanupRememFixtures,
  createRememFixture,
  readTrace,
} from "./remem-test-support.ts";
import { dirname, join } from "node:path";
import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import type { RememFixture } from "./remem-test-support.ts";
import { runDeploymentMoon } from "./deployment-moon-runner.ts";

const macosTest = test.skipIf(process.platform !== "darwin");
const expectedHookInstallCount = 4;

afterEach(cleanupRememFixtures);

function writeFixtureFile(path: string, content: string): void {
  mkdirSync(dirname(path), { recursive: true });
  writeFileSync(path, content);
}

function runTask(
  fixture: RememFixture,
  task: "remem-claude" | "remem-codex" | "remem-hooks" | "remem-worker",
  environment: Readonly<NodeJS.ProcessEnv> = {},
): ReturnType<typeof runDeploymentMoon> {
  return runDeploymentMoon(fixture, [`harness:${task}`], {
    FAKE_TRACE: fixture.trace,
    PATH: `${fixture.binaries}:${dirname(process.execPath)}:/usr/bin:/bin`,
    ...environment,
  });
}

macosTest(
  "Moon remem-hooks restores the deployed config after CLI failure",
  () => {
    const fixture = createRememFixture();
    const config = join(fixture.home, ".remem", "config.toml");
    writeFixtureFile(config, "tracked = true\n");

    const result = runTask(fixture, "remem-hooks", {
      FAKE_APPEND_TO: config,
      FAKE_FAIL_ON: "target codex",
    });

    expect(result.exitCode).not.toBe(0);
    expect(readFileSync(config, "utf8")).toBe("tracked = true\n");
    expect(readTrace(fixture).map(({ arguments: args }) => args)).toEqual([
      ["install", "--target", "claude", "--hooks-only"],
      ["install", "--target", "codex", "--hooks-only"],
    ]);
  },
);

macosTest(
  "Moon remem-hooks replays without changing the deployed config",
  () => {
    const fixture = createRememFixture();
    const config = join(fixture.home, ".remem", "config.toml");
    writeFixtureFile(config, "tracked = true\n");

    expect(
      runTask(fixture, "remem-hooks", { FAKE_APPEND_TO: config }).exitCode,
    ).toBe(0);
    expect(
      runTask(fixture, "remem-hooks", { FAKE_APPEND_TO: config }).exitCode,
    ).toBe(0);
    expect(readFileSync(config, "utf8")).toBe("tracked = true\n");
    expect(readTrace(fixture)).toHaveLength(expectedHookInstallCount);
  },
);

macosTest("Moon remem-codex preserves its config on replay", () => {
  const fixture = createRememFixture();
  const config = join(fixture.home, ".codex", "config.toml");
  writeFixtureFile(
    config,
    'model = "gpt"\n\n[mcp_servers.remem]\ncommand = "remem"\n',
  );

  expect(runTask(fixture, "remem-codex").exitCode).toBe(0);
  const installed = readFileSync(config, "utf8");
  expect(runTask(fixture, "remem-codex").exitCode).toBe(0);
  expect(readFileSync(config, "utf8")).toBe(installed);
  expect(installed).toContain('model = "gpt"');
});

macosTest("Moon remem-codex refuses invalid config before CLI mutation", () => {
  const fixture = createRememFixture();
  const config = join(fixture.home, ".codex", "config.toml");
  writeFixtureFile(config, "[mcp_servers.remem\n");

  const result = runTask(fixture, "remem-codex");

  expect(result.exitCode).not.toBe(0);
  expect(readFileSync(config, "utf8")).toBe("[mcp_servers.remem\n");
  expect(readTrace(fixture)).toEqual([]);
});

macosTest("Moon remem-claude preserves unrelated settings on replay", () => {
  const fixture = createRememFixture();
  const settings = join(fixture.home, ".claude", "settings.json");
  writeFixtureFile(settings, '{"theme":"dark","autoMemoryEnabled":true}');

  expect(runTask(fixture, "remem-claude").exitCode).toBe(0);
  const installed = readFileSync(settings, "utf8");
  expect(runTask(fixture, "remem-claude").exitCode).toBe(0);
  expect(readFileSync(settings, "utf8")).toBe(installed);
  expect(JSON.parse(installed)).toEqual({
    theme: "dark",
    autoMemoryEnabled: false,
  });
});

macosTest(
  "Moon remem-claude refuses invalid settings before CLI mutation",
  () => {
    const fixture = createRememFixture();
    const settings = join(fixture.home, ".claude", "settings.json");
    writeFixtureFile(settings, "{invalid");

    const result = runTask(fixture, "remem-claude");

    expect(result.exitCode).not.toBe(0);
    expect(readFileSync(settings, "utf8")).toBe("{invalid");
    expect(readTrace(fixture)).toEqual([]);
  },
);

macosTest("Moon remem-worker preserves its plist on replay", () => {
  const fixture = createRememFixture();
  const plist = join(
    fixture.home,
    "Library",
    "LaunchAgents",
    "dev.remem.worker.plist",
  );

  expect(runTask(fixture, "remem-worker").exitCode).toBe(0);
  const installed = readFileSync(plist, "utf8");
  expect(runTask(fixture, "remem-worker").exitCode).toBe(0);
  expect(readFileSync(plist, "utf8")).toBe(installed);
});

macosTest(
  "Moon remem-worker reports bootstrap failure and retains its plist",
  () => {
    const fixture = createRememFixture();
    const plist = join(
      fixture.home,
      "Library",
      "LaunchAgents",
      "dev.remem.worker.plist",
    );

    const result = runTask(fixture, "remem-worker", {
      FAKE_FAIL_ON: "bootstrap",
    });

    expect(result.exitCode).not.toBe(0);
    expect(existsSync(plist)).toBe(true);
    expect(
      readTrace(fixture).some(({ arguments: args }) => args[0] === "bootstrap"),
    ).toBe(true);
  },
);
