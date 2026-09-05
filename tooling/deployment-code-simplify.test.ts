import {
  type CommandResult,
  cleanupDeploymentFixtures,
  createDeploymentFixture,
  expectSuccess,
  linkTarget,
  project,
  requireCommand,
} from "./deployment-test-support.ts";
import { afterEach, expect, setDefaultTimeout, test } from "bun:test";
import {
  lstatSync,
  mkdirSync,
  readFileSync,
  readdirSync,
  symlinkSync,
  writeFileSync,
} from "node:fs";
import { join } from "node:path";

const deploymentTimeoutMilliseconds = 15_000;
setDefaultTimeout(deploymentTimeoutMilliseconds);
afterEach(cleanupDeploymentFixtures);

type Agent = "claude" | "codex";
type Fixture = Readonly<{
  destination: string;
  home: string;
  root: string;
}>;

function skillFixture(agent: Agent): Fixture {
  const fixture = createDeploymentFixture(`code-simplify-${agent}`);
  const directory = agent === "claude" ? ".claude" : ".agents";
  const parent = join(fixture.home, directory, "skills");
  mkdirSync(parent, { recursive: true });
  return { ...fixture, destination: join(parent, "code-simplify") };
}

function deploySkill(fixture: Fixture, agent: Agent): CommandResult {
  const result = Bun.spawnSync(
    [
      requireCommand("moon"),
      "exec",
      "--quiet",
      "--no-actions",
      "--ignore-ci-checks",
      `repository:code-simplify-${agent}`,
    ],
    {
      cwd: project,
      env: {
        ...process.env,
        HOME: fixture.home,
        MOON_HOME: join(fixture.root, "moon"),
      },
      stdout: "pipe",
      stderr: "pipe",
    },
  );
  return {
    exitCode: result.exitCode,
    stdout: result.stdout.toString(),
    stderr: result.stderr.toString(),
  };
}

for (const agent of ["claude", "codex"] as const) {
  test(`${agent}: creates the skill link and replays without mutation`, () => {
    const fixture = skillFixture(agent);
    expectSuccess(deploySkill(fixture, agent));
    expect(linkTarget(fixture.destination)).toBe(
      join(project, "harness/skills/code-simplify"),
    );
    const inode = lstatSync(fixture.destination).ino;
    const replay = deploySkill(fixture, agent);
    expectSuccess(replay);
    expect(replay.stdout).toBe("");
    expect(replay.stderr).toBe("");
    expect(lstatSync(fixture.destination).ino).toBe(inode);
  });

  test(`${agent}: preserves a conflicting file`, () => {
    const fixture = skillFixture(agent);
    writeFileSync(fixture.destination, "preserve me");
    expect(deploySkill(fixture, agent).exitCode).not.toBe(0);
    expect(readFileSync(fixture.destination, "utf8")).toBe("preserve me");
  });

  test(`${agent}: refuses to link inside a conflicting directory`, () => {
    const fixture = skillFixture(agent);
    mkdirSync(fixture.destination);
    expect(deploySkill(fixture, agent).exitCode).not.toBe(0);
    expect(readdirSync(fixture.destination)).toEqual([]);
  });

  test(`${agent}: preserves a conflicting dangling symlink`, () => {
    const fixture = skillFixture(agent);
    const missing = join(fixture.root, "missing");
    symlinkSync(missing, fixture.destination);
    expect(deploySkill(fixture, agent).exitCode).not.toBe(0);
    expect(linkTarget(fixture.destination)).toBe(missing);
  });
}
