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
  readFileSync,
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
setDefaultTimeout(deploymentTimeoutMilliseconds);
const executableFileMode = 0o755;

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
  expect(statSync(destination).mode & executableFileMode).not.toBe(0);
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

test.each([
  ["codex", "codex", true],
  ["claude-code", "claude", true],
  ["cursor", "cursor", false],
] as const)(
  "%s entry point deploys its memory runtime",
  (target, agent, deploysHandoff) => {
    const fixture = createDeploymentFixture(`memory-wiring-${target}`);
    const handoffTarget = "moon exec --quiet agent-handoff:install";
    const expected = `"${fixture.home}/.local/bin/arnes" setup hooks --agent ${agent}`;

    installBuildProviders(fixture);
    const result = runMake(fixture, [target], {
      dryRun: true,
      repository: project,
      variables: { BREW_BIN: fixture.bin },
    });

    expectSuccess(result);
    expect(result.stdout).toContain("moon exec --quiet agent-memory:install");
    expect(result.stdout.includes(handoffTarget)).toBe(deploysHandoff);
    expect(result.stdout).toContain(expected);
  },
);

test("declares memory hooks for Codex and Claude only", () => {
  const manifest = readFileSync(join(project, "home", ".arnes.yaml"), "utf8");

  expect(manifest).toMatch(
    /- id: memory\n\s+installations:\n\s+- \{ agent: claude, scope: user \}\n\s+- \{ agent: codex, scope: user \}/u,
  );
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
  const rule = readFileSync(source, "utf8");
  expect(rule).toContain("alwaysApply: true");
  expect(rule).toContain("agent-memory retrieve --query-stdin --format json");
  expect(rule).toContain("wait for completion");
  expect(rule).toContain("apply no memory");
  expect(rule).not.toMatch(/schema_version|ranking|privacy policy/u);
});

test("keeps memory and handoff runtime targets independent", () => {
  const fixture = createDeploymentFixture("memory-runtime-binaries");
  const memory = join(fixture.home, ".local", "bin", "agent-memory");
  const handoffTarget = "moon exec --quiet agent-handoff:install";
  const result = (target: string): ReturnType<typeof runMake> =>
    runMake(fixture, [target], {
      dryRun: true,
      repository: project,
      variables: { BREW_BIN: fixture.bin },
    });

  installBuildProviders(fixture);
  const memoryResult = result("agent-memory");
  const handoffResult = result("agent-handoff");
  expectSuccess(memoryResult);
  expectSuccess(handoffResult);
  expect(memoryResult.stdout).toContain(
    "moon exec --quiet agent-memory:install",
  );
  expect(memoryResult.stdout).not.toContain(handoffTarget);
  expect(handoffResult.stdout).toContain(handoffTarget);
  expect(handoffResult.stdout).not.toContain(memory);
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

function installBuildProviders(
  fixture: ReturnType<typeof createDeploymentFixture>,
): void {
  const provider = requireCommand("true");
  for (const command of ["bun", "cargo", "volta"]) {
    symlinkSync(provider, join(fixture.bin, command));
  }
}
