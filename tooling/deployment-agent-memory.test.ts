import { afterEach, expect, setDefaultTimeout, test } from "bun:test";
import {
  cleanupDeploymentFixtures,
  createDeploymentFixture,
  expectSuccess,
  linkTarget,
  pathExists,
  project,
  runMake,
} from "./deployment-test-support.ts";
import {
  cleanupMoonDeploymentFixtures,
  createMoonDeploymentFixture,
  runMoon,
} from "./deployment-moon-test-support.ts";
import { mkdirSync, realpathSync, symlinkSync, writeFileSync } from "node:fs";
import { join } from "node:path";

afterEach(() => {
  cleanupDeploymentFixtures();
  cleanupMoonDeploymentFixtures();
});

const deploymentTimeoutMilliseconds = 120_000;
setDefaultTimeout(deploymentTimeoutMilliseconds);

test("installs an executable memory runtime through Moon", () => {
  const fixture = createMoonDeploymentFixture("agent-memory");
  const result = runMoon(fixture, "agent-memory:install");
  const destination = join(fixture.home, ".local/bin/agent-memory");

  expectSuccess(result);
  expect(linkTarget(destination)).toBe(
    join(
      realpathSync(fixture.repository),
      "tooling/agent-memory/target/release/agent-memory",
    ),
  );
  expect(Bun.spawnSync([destination, "--help"]).exitCode).toBe(0);
  expect(
    pathExists(join(fixture.home, ".local/bin/agent-handoff")),
  ).toBeFalse();
});

test("propagates a memory build failure", () => {
  const fixture = createMoonDeploymentFixture("agent-memory");
  const result = runMoon(fixture, "agent-memory:install", {
    cache: "off",
    environment: { RUSTC: join(fixture.root, "missing-rustc") },
    force: true,
  });

  expect(result.exitCode).not.toBe(0);
  expect(result.stderr).toContain("missing-rustc");
});

test("refuses a memory build that produces no canonical binary", () => {
  const fixture = createMoonDeploymentFixture("agent-memory");
  const alternateTarget = join(fixture.root, "alternate-target");
  const result = runMoon(fixture, "agent-memory:install", {
    cache: "off",
    environment: { CARGO_TARGET_DIR: alternateTarget },
    force: true,
  });

  expect(result.exitCode).not.toBe(0);
  expect(result.stderr).toContain(
    "tooling/agent-memory/target/release/agent-memory",
  );
});

test("propagates a memory deployment failure", () => {
  const fixture = createMoonDeploymentFixture("agent-memory");
  writeFileSync(join(fixture.home, ".local"), "occupied\n");
  const result = runMoon(fixture, "agent-memory:install");

  expect(result.exitCode).not.toBe(0);
  expect(result.stderr).toContain(".local");
  expect(result.stderr).toContain("Not a directory");
});

test("deploys the Cursor memory rule from its canonical source", () => {
  const fixture = createDeploymentFixture("cursor-memory-rule");
  const source = join(project, "harness/rules/memory-governance-cursor.mdc");
  const destination = join(
    fixture.home,
    ".cursor/rules/memory-governance-cursor.mdc",
  );

  expectSuccess(runMake(fixture, [destination], { repository: project }));
  expect(linkTarget(destination)).toBe(source);
});

test.each(["file", "directory", "symlink"] as const)(
  "clean removes the owned memory %s destination only",
  (destinationType) => {
    const fixture = createDeploymentFixture("memory-clean");
    const destination = join(fixture.home, ".local/bin/agent-memory");
    const neighbor = join(fixture.home, ".local/bin/keep");
    const external = join(fixture.root, "external");
    mkdirSync(join(fixture.home, ".local/bin"), { recursive: true });
    if (destinationType === "directory") {
      mkdirSync(destination);
    } else if (destinationType === "symlink") {
      mkdirSync(external);
      symlinkSync(external, destination);
    } else {
      writeFileSync(destination, "memory\n");
    }
    writeFileSync(neighbor, "keep\n");

    expectSuccess(runMake(fixture, ["clean"], { repository: project }));
    expect(pathExists(destination)).toBeFalse();
    expect(pathExists(neighbor)).toBeTrue();
    if (destinationType === "symlink") {
      expect(pathExists(external)).toBeTrue();
    }
  },
);
