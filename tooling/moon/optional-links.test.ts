import type {
  CommandResult,
  DeploymentFixture,
} from "../deployment-test-support.ts";
import { afterEach, expect, test } from "bun:test";
import {
  cleanupDeploymentFixtures,
  createDeploymentFixture,
  project,
  requireCommand,
} from "../deployment-test-support.ts";
import { dirname, join } from "node:path";
import {
  lstatSync,
  mkdirSync,
  readFileSync,
  readlinkSync,
  symlinkSync,
  writeFileSync,
} from "node:fs";
import { runDeploymentMoon } from "../deployment-moon-runner.ts";

const psqlrcSource = join(project, "home", ".psqlrc");
const cursorDeploymentAndReplayTimeout = 30_000;

afterEach(cleanupDeploymentFixtures);

test("links psqlrc and replays silently", () => {
  const fixture = createDeploymentFixture("optional-postgresql");
  expect(runDeploymentMoon(fixture, ["home:postgresql"]).exitCode).toBe(0);
  const destination = join(fixture.home, ".psqlrc");
  expect(readlinkSync(destination)).toBe(psqlrcSource);
  const before = lstatSync(destination).ino;

  const replay = runDeploymentMoon(fixture, ["home:postgresql"]);

  expect(replay).toEqual({ exitCode: 0, stdout: "", stderr: "" });
  expect(lstatSync(destination).ino).toBe(before);
});

test("refuses a divergent psqlrc without replacing it", () => {
  const fixture = createDeploymentFixture("optional-postgresql-collision");
  const destination = join(fixture.home, ".psqlrc");
  writeFileSync(destination, "personal\n");

  const result = runDeploymentMoon(fixture, ["home:postgresql"]);

  expect(result.exitCode).not.toBe(0);
  expect(result.stderr).toContain(
    `${destination} exists and is not the expected symbolic link`,
  );
  expect(readFileSync(destination, "utf8")).toBe("personal\n");
});

function scraplingFixture(): Readonly<{
  destination: string;
  fixture: DeploymentFixture;
  trace: string;
}> {
  const fixture = createDeploymentFixture("optional-scrapling");
  const destination = join(fixture.home, ".local", "bin", "scrapling_mcp");
  const trace = join(fixture.root, "docker-trace");
  mkdirSync(dirname(destination), { recursive: true });
  symlinkSync(
    join(project, "tooling", "docker-install-test-provider.ts"),
    join(fixture.bin, "docker"),
  );
  writeFileSync(trace, "");
  return { destination, fixture, trace };
}

test("Moon links Scrapling without Docker", () => {
  const { destination, fixture, trace } = scraplingFixture();
  const source = join(project, "tooling", "scrapling-mcp");
  const environment = { PATH: `${dirname(process.execPath)}:/usr/bin:/bin` };

  expect(
    runDeploymentMoon(fixture, ["tooling:scrapling-mcp"], environment).exitCode,
  ).toBe(0);
  expect(readlinkSync(destination)).toBe(source);
  expect(readFileSync(trace, "utf8")).toBe("");
});

test("Moon preserves an existing Scrapling launcher silently without Docker", () => {
  const { destination, fixture, trace } = scraplingFixture();
  const source = join(project, "tooling", "scrapling-mcp");
  const environment = { PATH: `${dirname(process.execPath)}:/usr/bin:/bin` };
  symlinkSync(source, destination);
  const inode = lstatSync(destination).ino;

  expect(
    runDeploymentMoon(fixture, ["tooling:scrapling-mcp"], environment),
  ).toEqual({
    exitCode: 0,
    stdout: "",
    stderr: "",
  });
  expect(lstatSync(destination).ino).toBe(inode);
  expect(readFileSync(trace, "utf8")).toBe("");
});

test("Moon Scrapling refuses an occupied link before Docker API commands", () => {
  const { destination, fixture, trace } = scraplingFixture();
  writeFileSync(destination, "personal\n");

  const result = Bun.spawnSync(
    [
      process.env.DEPLOYMENT_MOON ?? requireCommand("moon"),
      "exec",
      "--quiet",
      "--ignore-ci-checks",
      "--no-actions",
      "--upstream",
      "none",
      "repository:scrapling",
    ],
    {
      cwd: project,
      env: {
        ...process.env,
        HOME: fixture.home,
        MOON_HOME:
          process.env.MOON_HOME ?? join(process.env.HOME ?? "", ".moon"),
        PROTO_HOME:
          process.env.PROTO_HOME ?? join(process.env.HOME ?? "", ".proto"),
        PROTO_OFFLINE: "true",
        DOCKER_INSTALL_TEST_SCENARIO: "artifact-present",
        DOCKER_INSTALL_TEST_STATE: trace,
        DOCKER_INSTALL_TEST_TARGET: "scrapling",
        PATH: `${fixture.bin}:${dirname(process.execPath)}:/usr/bin:/bin`,
      },
      stderr: "pipe",
      stdout: "pipe",
    },
  );

  expect(result.exitCode).not.toBe(0);
  expect(result.stderr.toString()).toContain(
    `${destination} exists and is not the expected symbolic link`,
  );
  expect(readFileSync(destination, "utf8")).toBe("personal\n");
  expect(readFileSync(trace, "utf8")).toBe("");
});

function cursorFixture(): Readonly<{
  fixture: DeploymentFixture;
  trace: string;
}> {
  const fixture = createDeploymentFixture("optional-cursor");
  const trace = join(fixture.root, "arnes-trace");
  const arnes = join(fixture.home, ".local", "bin", "arnes");
  mkdirSync(dirname(arnes), { recursive: true });
  symlinkSync(join(project, "tooling", "deployment-test-command.ts"), arnes);
  writeFileSync(trace, "");
  return { fixture, trace };
}

function runCursor(fixture: DeploymentFixture, trace: string): CommandResult {
  const moon = process.env.DEPLOYMENT_MOON ?? requireCommand("moon");
  const decoder = new TextDecoder("utf-8", { fatal: true });
  const result = Bun.spawnSync(
    [
      moon,
      "exec",
      "--quiet",
      "--ignore-ci-checks",
      "--no-actions",
      "--upstream",
      "direct",
      "harness:cursor",
    ],
    {
      cwd: project,
      env: {
        ...process.env,
        HOME: fixture.home,
        FAKE_FAIL_ON: "doctor hooks",
        FAKE_TRACE: trace,
        MOON_HOME:
          process.env.MOON_HOME ?? join(process.env.HOME ?? "", ".moon"),
        PATH: `${fixture.bin}:${dirname(process.execPath)}:/usr/bin:/bin`,
        PROTO_HOME:
          process.env.PROTO_HOME ?? join(process.env.HOME ?? "", ".proto"),
        PROTO_OFFLINE: "true",
      },
      stderr: "pipe",
      stdout: "pipe",
    },
  );
  return {
    exitCode: result.exitCode,
    stderr: decoder.decode(result.stderr),
    stdout: decoder.decode(result.stdout),
  };
}

test(
  "Moon deploys the Cursor rules and skills without global dependencies",
  () => {
    const { fixture, trace } = cursorFixture();
    const rule = join(
      fixture.home,
      ".cursor",
      "rules",
      "memory-governance-cursor.mdc",
    );
    const skill = join(fixture.home, ".cursor", "skills", "skill-manager");
    const plugin = join(fixture.home, ".cursor/plugins/local/dotfiles-harness");
    const commonRule = join(plugin, "rules/common-instructions.mdc");

    expect(runCursor(fixture, trace).exitCode).toBe(0);
    expect(readlinkSync(rule)).toBe(
      join(project, "harness", "rules", "memory-governance-cursor.mdc"),
    );
    expect(readlinkSync(skill)).toBe(
      join(project, "harness", "skills", "skill-manager"),
    );
    expect(lstatSync(plugin).isDirectory()).toBeTrue();
    expect(readlinkSync(join(plugin, ".cursor-plugin/plugin.json"))).toBe(
      join(
        project,
        "home/.cursor/plugins/local/dotfiles-harness/.cursor-plugin/plugin.json",
      ),
    );
    expect(readFileSync(commonRule, "utf8")).toContain(
      "alwaysApply: true\n---\n# Global AI Instructions",
    );
    expect(readFileSync(trace, "utf8")).toContain(
      '"arguments":["setup","hooks","--agent","cursor"]',
    );
    const ruleInode = lstatSync(rule).ino;
    const commonRuleInode = lstatSync(commonRule).ino;

    expect(runCursor(fixture, trace).exitCode).toBe(0);
    expect(lstatSync(rule).ino).toBe(ruleInode);
    expect(lstatSync(commonRule).ino).toBe(commonRuleInode);
  },
  cursorDeploymentAndReplayTimeout,
);

test.each(["existing", "dangling"])(
  "Moon refuses a %s symbolic Cursor plugin directory before mutation",
  (kind) => {
    const fixture = createDeploymentFixture("cursor-symbolic-plugin");
    const plugin = join(fixture.home, ".cursor/plugins/local/dotfiles-harness");
    const foreign = join(fixture.root, "foreign-plugin");
    mkdirSync(dirname(plugin), { recursive: true });
    if (kind === "existing") {
      mkdirSync(foreign);
    }
    symlinkSync(foreign, plugin);
    const result = runDeploymentMoon(fixture, ["harness:cursor-instructions"]);
    expect(result.exitCode).not.toBe(0);
    expect(result.stderr).toContain(
      "Refusing symbolic Cursor plugin directory",
    );
    expect(readlinkSync(plugin)).toBe(foreign);
    expect(
      lstatSync(join(foreign, ".cursor-plugin"), { throwIfNoEntry: false }),
    ).toBeUndefined();
    expect(
      lstatSync(join(foreign, "rules"), { throwIfNoEntry: false }),
    ).toBeUndefined();
  },
);

test("Moon refuses an occupied Cursor plugin manifest before changing its rule", () => {
  const fixture = createDeploymentFixture("cursor-instructions-collision");
  const plugin = join(fixture.home, ".cursor/plugins/local/dotfiles-harness");
  const manifest = join(plugin, ".cursor-plugin/plugin.json");
  const rule = join(plugin, "rules/common-instructions.mdc");
  for (const path of [manifest, rule]) {
    mkdirSync(dirname(path), { recursive: true });
    writeFileSync(path, "personal\n");
  }
  const result = runDeploymentMoon(fixture, ["harness:cursor-instructions"]);
  expect(result.exitCode).not.toBe(0);
  expect(result.stderr).toContain(
    "exists and is not the expected symbolic link",
  );
  expect(readFileSync(manifest, "utf8")).toBe("personal\n");
  expect(readFileSync(rule, "utf8")).toBe("personal\n");
});

test("Moon preserves a divergent Cursor rule and returns failure", () => {
  const { fixture, trace } = cursorFixture();
  const rule = join(
    fixture.home,
    ".cursor",
    "rules",
    "memory-governance-cursor.mdc",
  );
  mkdirSync(dirname(rule), { recursive: true });
  writeFileSync(rule, "personal\n");

  const result = runCursor(fixture, trace);

  expect(result.exitCode).not.toBe(0);
  expect(readFileSync(rule, "utf8")).toBe("personal\n");
});
