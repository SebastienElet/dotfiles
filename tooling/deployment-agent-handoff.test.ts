import { afterEach, expect, setDefaultTimeout, test } from "bun:test";
import {
  cleanupDeploymentFixtures,
  createDeploymentFixture,
  expectSuccess,
  linkTarget,
  pathExists,
  project,
} from "./deployment-test-support.ts";
import {
  cleanupMoonDeploymentFixtures,
  createMoonDeploymentFixture,
  foreignGitHubEnvironment,
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

test("installs an executable handoff runtime through Moon with foreign CI references", () => {
  const fixture = createMoonDeploymentFixture("agent-handoff");
  const result = runMoon(fixture, "agent-handoff:install", {
    environment: {
      ...foreignGitHubEnvironment,
      MOON_BASE: "foreign-base",
      MOON_HEAD: "foreign-head",
    },
  });
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
  expect(result.stderr).toContain("ENOTDIR");
  expect(readFileSync(join(fixture.home, ".local"), "utf8")).toBe("occupied\n");
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
  const fixture = createDeploymentFixture("handoff-clean");
  const destination = join(fixture.home, ".local/bin/agent-handoff");
  const source = join(
    project,
    "tooling/agent-handoff/target/release/agent-handoff",
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
    writeFileSync(destination, "handoff\n");
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
  "clean removes the handoff destination only when its link is owned: $destinationType",
  ({ destinationType, removed }) => {
    const { fixture, destination, source, sourceBefore, neighbor, external } =
      cleanupFixture(destinationType);

    expectSuccess(runDeploymentMoon(fixture, ["repository:clean"]));
    expect(pathExists(destination)).toBe(!removed);
    expect(readFileSync(neighbor, "utf8")).toBe("keep\n");
    expect(readFileSync(external, "utf8")).toBe("external\n");
    if (sourceBefore !== undefined) {
      expect(readFileSync(source)).toEqual(sourceBefore);
    }
    if (destinationType === "file") {
      expect(readFileSync(destination, "utf8")).toBe("handoff\n");
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
