import { afterEach, expect, test } from "bun:test";
import {
  cleanupMoonDeploymentFixtures,
  createMoonDeploymentFixture,
  foreignGitHubEnvironment,
  runMoon,
} from "./deployment-moon-test-support.ts";
import { dirname, join } from "node:path";
import {
  expectSuccess,
  fileIdentity,
  linkTarget,
  pathExists,
} from "./deployment-test-support.ts";
import {
  lstatSync,
  mkdirSync,
  readFileSync,
  realpathSync,
  rmSync,
  symlinkSync,
  writeFileSync,
} from "node:fs";

afterEach(cleanupMoonDeploymentFixtures);

test.each([
  { source: "GitHub", environment: foreignGitHubEnvironment },
  {
    source: "caller",
    environment: {
      ...foreignGitHubEnvironment,
      MOON_BASE: "foreign-base",
      MOON_HEAD: "foreign-head",
    },
  },
])(
  "executes fixture tasks with foreign $source references",
  ({ environment }: Readonly<{ environment: Readonly<NodeJS.ProcessEnv> }>) => {
    const fixture = createMoonDeploymentFixture("agent-memory");

    expectSuccess(
      runMoon(fixture, "repository:rust", { cache: "off", environment }),
    );
  },
);

const harnessBinaries = ["agent-memory", "agent-handoff", "arnes"] as const;
const binaryCollisions = [
  "file",
  "foreign-link",
  "broken-link",
  "directory",
] as const;

function occupyBinaryDestination(
  destination: string,
  foreign: string,
  destinationType: (typeof binaryCollisions)[number],
): void {
  mkdirSync(dirname(destination), { recursive: true });
  if (destinationType === "file") {
    writeFileSync(destination, "foreign binary\n");
    return;
  }
  if (destinationType === "directory") {
    mkdirSync(destination);
    writeFileSync(join(destination, "keep"), "personal\n");
    return;
  }
  if (destinationType === "foreign-link") {
    mkdirSync(dirname(foreign), { recursive: true });
    writeFileSync(foreign, "foreign binary\n");
  }
  symlinkSync(foreign, destination);
}

function binaryFixture(projectId: (typeof harnessBinaries)[number]): Readonly<{
  fixture: ReturnType<typeof createMoonDeploymentFixture>;
  source: string;
  destination: string;
}> {
  const fixture = createMoonDeploymentFixture(projectId);
  const source = join(
    realpathSync(fixture.repository),
    "tooling",
    projectId,
    "target/release",
    projectId,
  );
  const destination = join(fixture.home, ".local/bin", projectId);
  mkdirSync(dirname(source), { recursive: true });
  writeFileSync(source, "canonical binary\n", { mode: 0o755 });
  return { fixture, source, destination };
}

for (const projectId of harnessBinaries) {
  test(`${projectId} binary deployment through Moon creates the absent destination and preserves the canonical link silently on replay`, () => {
    const { fixture, source, destination } = binaryFixture(projectId);
    const sourceBefore = fileIdentity(source);
    expectSuccess(
      runMoon(fixture, `${projectId}:binary`, { upstream: "none" }),
    );
    expect(linkTarget(destination)).toBe(source);
    const inode = lstatSync(destination).ino;

    const replay = runMoon(fixture, `${projectId}:binary`, {
      upstream: "none",
    });

    expect(replay).toEqual({ exitCode: 0, stdout: "", stderr: "" });
    expect(linkTarget(destination)).toBe(source);
    expect(lstatSync(destination).ino).toBe(inode);
    expect(fileIdentity(source)).toEqual(sourceBefore);
  });

  test.each([...binaryCollisions])(
    `${projectId} binary deployment through Moon refuses a divergent %s without mutating the destination or its target`,
    (destinationType) => {
      const { fixture, source, destination } = binaryFixture(projectId);
      const sourceBefore = fileIdentity(source);
      const foreign = join(
        fixture.root,
        "other-checkout/tooling",
        projectId,
        "target/release",
        projectId,
      );
      occupyBinaryDestination(destination, foreign, destinationType);
      const inode = lstatSync(destination).ino;
      const foreignBefore =
        destinationType === "foreign-link" ? fileIdentity(foreign) : undefined;

      const result = runMoon(fixture, `${projectId}:binary`, {
        upstream: "none",
      });

      expect(result.exitCode).not.toBe(0);
      expect(result.stderr).toContain(
        `${destination} exists and is not the expected symbolic link`,
      );
      expect(lstatSync(destination).ino).toBe(inode);
      expect(fileIdentity(source)).toEqual(sourceBefore);
      if (destinationType === "file") {
        expect(readFileSync(destination, "utf8")).toBe("foreign binary\n");
      } else if (destinationType === "directory") {
        expect(readFileSync(join(destination, "keep"), "utf8")).toBe(
          "personal\n",
        );
        expect(pathExists(join(destination, projectId))).toBeFalse();
      } else {
        expect(linkTarget(destination)).toBe(foreign);
        if (foreignBefore === undefined) {
          expect(pathExists(foreign)).toBeFalse();
        } else {
          expect(fileIdentity(foreign)).toEqual(foreignBefore);
        }
      }
    },
  );

  test(`${projectId} binary deployment through Moon refuses a missing canonical source before creating a destination`, () => {
    const { fixture, source, destination } = binaryFixture(projectId);
    rmSync(source);

    const result = runMoon(fixture, `${projectId}:binary`, {
      upstream: "none",
    });

    expect(result.exitCode).not.toBe(0);
    expect(result.stderr).toContain(source);
    expect(pathExists(destination)).toBeFalse();
  });
}
