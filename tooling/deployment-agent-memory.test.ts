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
import {
  mkdirSync,
  readFileSync,
  realpathSync,
  symlinkSync,
  writeFileSync,
} from "node:fs";
import { join } from "node:path";
import { runDeploymentMoon } from "./deployment-moon-runner.ts";

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

  expectSuccess(runDeploymentMoon(fixture, ["harness:cursor-rules"]));
  expect(linkTarget(destination)).toBe(source);
});

type CleanupDestination = "owned-link" | "file" | "directory" | "foreign-link";
type CleanupFixture = Readonly<{
  fixture: ReturnType<typeof createDeploymentFixture>;
  destination: string;
  source: string;
  sourceBefore: Buffer<ArrayBuffer> | undefined;
  neighbor: string;
  external: string;
}>;

function cleanupFixture(destinationType: CleanupDestination): CleanupFixture {
  const fixture = createDeploymentFixture("memory-clean");
  const destination = join(fixture.home, ".local/bin/agent-memory");
  const source = join(
    project,
    "tooling/agent-memory/target/release/agent-memory",
  );
  const neighbor = join(fixture.home, ".local/bin/keep");
  const external = join(fixture.root, "external");
  mkdirSync(join(fixture.home, ".local/bin"), { recursive: true });
  writeFileSync(external, "external\n");
  if (destinationType === "directory") {
    mkdirSync(destination);
    writeFileSync(join(destination, "keep"), "directory\n");
  } else if (destinationType === "owned-link") {
    symlinkSync(source, destination);
  } else if (destinationType === "foreign-link") {
    symlinkSync(external, destination);
  } else {
    writeFileSync(destination, "memory\n");
  }
  writeFileSync(neighbor, "keep\n");
  const sourceBefore = pathExists(source) ? readFileSync(source) : undefined;
  return { fixture, destination, source, sourceBefore, neighbor, external };
}

test.each([
  { destinationType: "owned-link", removed: true },
  { destinationType: "file", removed: false },
  { destinationType: "directory", removed: false },
  { destinationType: "foreign-link", removed: false },
] as const)(
  "clean removes the memory destination only when its link is owned: $destinationType",
  ({ destinationType, removed }) => {
    const { fixture, destination, source, sourceBefore, neighbor, external } =
      cleanupFixture(destinationType);

    expectSuccess(
      runMake(fixture, ["clean"], {
        repository: project,
        environment: {
          MOON_HOME:
            process.env.MOON_HOME ?? join(process.env.HOME ?? "", ".moon"),
          PROTO_HOME:
            process.env.PROTO_HOME ?? join(process.env.HOME ?? "", ".proto"),
          PROTO_OFFLINE: "true",
        },
        variables: {
          MOON_EXEC:
            "moon exec --quiet --ignore-ci-checks --no-actions --upstream none",
        },
      }),
    );
    expect(pathExists(destination)).toBe(!removed);
    expect(readFileSync(neighbor, "utf8")).toBe("keep\n");
    expect(readFileSync(external, "utf8")).toBe("external\n");
    if (sourceBefore !== undefined) {
      expect(readFileSync(source)).toEqual(sourceBefore);
    }
    if (destinationType === "file") {
      expect(readFileSync(destination, "utf8")).toBe("memory\n");
    }
    if (destinationType === "directory") {
      expect(readFileSync(join(destination, "keep"), "utf8")).toBe(
        "directory\n",
      );
    }
    if (destinationType === "foreign-link") {
      expect(linkTarget(destination)).toBe(external);
    }
  },
);
