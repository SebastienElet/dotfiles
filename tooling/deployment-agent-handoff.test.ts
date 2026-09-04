import { afterEach, expect, setDefaultTimeout, test } from "bun:test";
import {
  cleanupDeploymentFixtures,
  createDeploymentFixture,
  expectSuccess,
  linkTarget,
  pathExists,
  project,
  requireCommand,
  runMake,
} from "./deployment-test-support.ts";
import {
  cleanupMoonDeploymentFixtures,
  createMoonDeploymentFixture,
  runMoon,
} from "./deployment-moon-test-support.ts";
import {
  mkdirSync,
  realpathSync,
  statSync,
  symlinkSync,
  writeFileSync,
} from "node:fs";
import { join } from "node:path";

afterEach(() => {
  cleanupDeploymentFixtures();
  cleanupMoonDeploymentFixtures();
});

const deploymentTimeoutMilliseconds = 120_000;
const executableFileMode = 0o755;
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
  expect(statSync(destination).mode & executableFileMode).not.toBe(0);
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

test.each([
  ["codex", true],
  ["claude-code", true],
  ["cursor", false],
] as const)("%s preserves its handoff dependency", (target, deploysHandoff) => {
  const fixture = createDeploymentFixture(`handoff-wiring-${target}`);
  installBuildProviders(fixture);
  const result = runMake(fixture, [target], {
    dryRun: true,
    repository: project,
    variables: { BREW_BIN: fixture.bin },
  });

  expectSuccess(result);
  expect(
    result.stdout.includes("moon exec --quiet agent-handoff:install"),
  ).toBe(deploysHandoff);
});

test("keeps the handoff runtime independent from memory", () => {
  const fixture = createDeploymentFixture("handoff-runtime-independence");
  const result = runMake(fixture, ["agent-handoff"], {
    dryRun: true,
    repository: project,
  });

  expectSuccess(result);
  expect(result.stdout).toContain("moon exec --quiet agent-handoff:install");
  expect(result.stdout).not.toContain("agent-memory");
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

function installBuildProviders(
  fixture: ReturnType<typeof createDeploymentFixture>,
): void {
  const provider = requireCommand("true");
  for (const command of ["bun", "cargo", "volta"]) {
    symlinkSync(provider, join(fixture.bin, command));
  }
}
