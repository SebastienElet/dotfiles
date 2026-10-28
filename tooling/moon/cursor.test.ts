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

const cursorDeploymentAndReplayTimeout = 30_000;

afterEach(cleanupDeploymentFixtures);

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
      "visual-presentation.mdc",
    );
    const skill = join(fixture.home, ".cursor", "skills", "skill-manager");
    const plugin = join(fixture.home, ".cursor/plugins/local/dotfiles-harness");
    const commonRule = join(plugin, "rules/common-instructions.mdc");

    expect(runCursor(fixture, trace).exitCode).toBe(0);
    expect(readlinkSync(rule)).toBe(
      join(project, "harness", "visual-presentation.md"),
    );
    expect(readlinkSync(skill)).toBe(
      join(
        project,
        "node_modules/@sebastienelet/agent-skills/skills/skill-manager",
      ),
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
      '"arguments":["doctor","hooks","--agent","cursor","--color","never"]',
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
    "visual-presentation.mdc",
  );
  mkdirSync(dirname(rule), { recursive: true });
  writeFileSync(rule, "personal\n");

  const result = runCursor(fixture, trace);

  expect(result.exitCode).not.toBe(0);
  expect(readFileSync(rule, "utf8")).toBe("personal\n");
});
