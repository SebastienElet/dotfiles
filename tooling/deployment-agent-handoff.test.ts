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

test("installs an executable handoff runtime through Moon", () => {
  const fixture = createMoonDeploymentFixture("agent-handoff");
  const result = runMoon(fixture, "agent-handoff:install");
  const destination = join(fixture.home, ".local/bin/agent-handoff");

  expectSuccess(result);
  expect(linkTarget(destination)).toBe(
    join(
      realpathSync(fixture.repository),
      "tooling/agent-handoff/target/release/agent-handoff",
    ),
  );
  const event = {
    event: "Stop",
    session_id: "deployment-smoke",
    stop_hook_active: true,
    transcript_path: join(fixture.root, "unused-transcript"),
  };
  const invocation = Bun.spawnSync([destination], {
    stdin: Buffer.from(JSON.stringify(event)),
    stdout: "pipe",
    stderr: "pipe",
  });
  expect(invocation.exitCode).toBe(0);
  expect(invocation.stdout.toString()).toBe("");
  expect(pathExists(join(fixture.home, ".local/bin/agent-memory"))).toBeFalse();
});

test("propagates a handoff build failure", () => {
  const fixture = createMoonDeploymentFixture("agent-handoff");
  const result = runMoon(fixture, "agent-handoff:install", {
    cache: "off",
    environment: { RUSTC: join(fixture.root, "missing-rustc") },
    force: true,
  });

  expect(result.exitCode).not.toBe(0);
  expect(result.stderr).toContain("missing-rustc");
});

test("refuses a handoff build that produces no canonical binary", () => {
  const fixture = createMoonDeploymentFixture("agent-handoff");
  const alternateTarget = join(fixture.root, "alternate-target");
  const result = runMoon(fixture, "agent-handoff:install", {
    cache: "off",
    environment: { CARGO_TARGET_DIR: alternateTarget },
    force: true,
  });

  expect(result.exitCode).not.toBe(0);
  expect(result.stderr).toContain(
    "tooling/agent-handoff/target/release/agent-handoff",
  );
});

test("propagates a handoff deployment failure", () => {
  const fixture = createMoonDeploymentFixture("agent-handoff");
  writeFileSync(join(fixture.home, ".local"), "occupied\n");
  const result = runMoon(fixture, "agent-handoff:install");

  expect(result.exitCode).not.toBe(0);
  expect(result.stderr).toContain(".local");
  expect(result.stderr).toContain("Not a directory");
});

test.each(["file", "directory", "symlink"] as const)(
  "clean removes the owned handoff %s destination only",
  (destinationType) => {
    const fixture = createDeploymentFixture("handoff-clean");
    const destination = join(fixture.home, ".local/bin/agent-handoff");
    const neighbor = join(fixture.home, ".local/bin/keep");
    const external = join(fixture.root, "external");
    mkdirSync(join(fixture.home, ".local/bin"), { recursive: true });
    if (destinationType === "directory") {
      mkdirSync(destination);
    } else if (destinationType === "symlink") {
      mkdirSync(external);
      symlinkSync(external, destination);
    } else {
      writeFileSync(destination, "handoff\n");
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
