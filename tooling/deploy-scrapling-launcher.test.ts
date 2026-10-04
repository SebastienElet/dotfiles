import { afterEach, expect, test } from "bun:test";
import {
  cleanupDeploymentFixtures,
  createDeploymentFixture,
  pathExists,
  project,
  runDeploymentHelper,
} from "./deployment-test-support.ts";
import { dirname, join } from "node:path";
import {
  lstatSync,
  mkdirSync,
  readFileSync,
  readlinkSync,
  symlinkSync,
  writeFileSync,
} from "node:fs";

afterEach(cleanupDeploymentFixtures);

test("deploys the canonical launcher and preserves its inode silently on replay", () => {
  const fixture = createDeploymentFixture("scrapling-launcher-replay");
  const invocation = { helper: "deploy-scrapling-launcher.ts", arguments: [] };
  const destination = join(fixture.home, ".local/bin/scrapling_mcp");

  expect(runDeploymentHelper(fixture, invocation)).toEqual({
    exitCode: 0,
    stdout: "",
    stderr: "",
  });
  expect(readlinkSync(destination)).toBe(
    join(project, "tooling/scrapling-mcp"),
  );
  const inode = lstatSync(destination).ino;

  expect(runDeploymentHelper(fixture, invocation)).toEqual({
    exitCode: 0,
    stdout: "",
    stderr: "",
  });
  expect(lstatSync(destination).ino).toBe(inode);
});

test.each([undefined, ""])(
  "rejects invalid HOME %s before deployment",
  (home) => {
    const fixture = createDeploymentFixture("scrapling-launcher-home");
    const result = runDeploymentHelper(
      fixture,
      { helper: "deploy-scrapling-launcher.ts", arguments: [] },
      { HOME: home },
    );

    expect(result.exitCode).not.toBe(0);
    expect(result.stderr).toContain("HOME");
    expect(pathExists(join(fixture.home, ".local"))).toBe(false);
    expect(pathExists(join(fixture.root, ".local"))).toBe(false);
  },
);

test("rejects arguments before deployment", () => {
  const fixture = createDeploymentFixture("scrapling-launcher-arguments");
  const result = runDeploymentHelper(fixture, {
    helper: "deploy-scrapling-launcher.ts",
    arguments: ["unexpected"],
  });

  expect(result.exitCode).not.toBe(0);
  expect(result.stderr).toContain("Too big");
  expect(pathExists(join(fixture.home, ".local"))).toBe(false);
});

test.each(["file", "divergent-link", "directory", "parent-file"])(
  "refuses %s collisions without mutation",
  (collision) => {
    const fixture = createDeploymentFixture("scrapling-launcher-collision");
    const destination = join(fixture.home, ".local/bin/scrapling_mcp");
    mkdirSync(dirname(destination), { recursive: true });
    const occupied =
      collision === "parent-file"
        ? join(fixture.home, ".local/bin/occupied")
        : destination;
    if (collision === "divergent-link") {
      symlinkSync("missing", occupied);
    } else if (collision === "directory") {
      mkdirSync(occupied);
    } else {
      writeFileSync(occupied, "personal\n");
    }
    const inode = lstatSync(occupied).ino;
    const home = collision === "parent-file" ? occupied : fixture.home;
    const result = runDeploymentHelper(
      fixture,
      { helper: "deploy-scrapling-launcher.ts", arguments: [] },
      { HOME: home },
    );

    expect(result.exitCode).not.toBe(0);
    expect(result.stderr).toContain(occupied);
    expect(lstatSync(occupied).ino).toBe(inode);
    if (collision === "divergent-link") {
      expect(readlinkSync(occupied)).toBe("missing");
    } else if (collision !== "directory") {
      expect(readFileSync(occupied, "utf8")).toBe("personal\n");
    }
  },
);
