import { afterEach, expect, test } from "bun:test";
import {
  cleanupDeploymentFixtures,
  createDeploymentFixture,
  expectSuccess,
  fileIdentity,
  linkTarget,
  pathExists,
  project,
} from "./deployment-test-support.ts";
import { dirname, join } from "node:path";
import {
  lstatSync,
  mkdirSync,
  readFileSync,
  symlinkSync,
  writeFileSync,
} from "node:fs";
import { runDeploymentMoon } from "./deployment-moon-runner.ts";

afterEach(cleanupDeploymentFixtures);

const harnessBinaries = ["arnes"] as const;
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
  fixture: ReturnType<typeof createDeploymentFixture>;
  source: string;
  destination: string;
}> {
  const fixture = createDeploymentFixture(projectId);
  const source = join(project, "node_modules/.bin", projectId);
  const destination = join(fixture.home, ".local/bin", projectId);
  return { fixture, source, destination };
}

for (const projectId of harnessBinaries) {
  test(`${projectId} binary deployment through Moon creates the absent destination and preserves the canonical link silently on replay`, () => {
    const { fixture, source, destination } = binaryFixture(projectId);
    const sourceBefore = fileIdentity(source);
    expectSuccess(runDeploymentMoon(fixture, [`harness:${projectId}`]));
    expect(linkTarget(destination)).toBe(source);
    const inode = lstatSync(destination).ino;

    const replay = runDeploymentMoon(fixture, [`harness:${projectId}`]);

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
        "other-checkout/node_modules/.bin",
        projectId,
      );
      occupyBinaryDestination(destination, foreign, destinationType);
      const inode = lstatSync(destination).ino;
      const foreignBefore =
        destinationType === "foreign-link" ? fileIdentity(foreign) : undefined;

      const result = runDeploymentMoon(fixture, [`harness:${projectId}`]);

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
}
