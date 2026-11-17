import { afterEach, expect, test } from "bun:test";
import {
  cleanupDeploymentFixtures,
  createDeploymentFixture,
  project,
} from "../deployment-test-support.ts";
import {
  lstatSync,
  mkdirSync,
  readFileSync,
  readlinkSync,
  writeFileSync,
} from "node:fs";
import { join } from "node:path";
import { runDeploymentMoon } from "../deployment-moon-runner.ts";

const psqlrcSource = join(project, "home", ".psqlrc");

afterEach(cleanupDeploymentFixtures);

test("links psqlrc and replays silently", () => {
  const fixture = createDeploymentFixture("optional-postgresql");
  expect(runDeploymentMoon(fixture, ["home:postgresql"]).exitCode).toBe(0);
  const destination = join(fixture.home, ".psqlrc");
  expect(readlinkSync(destination)).toBe(psqlrcSource);
  const before = lstatSync(destination).ino;

  const replay = runDeploymentMoon(fixture, ["home:postgresql"]);

  expect(replay).toEqual({ exitCode: 0, stdout: "", stderr: "" });
  expect(lstatSync(destination).ino).toBe(before);
});

test("refuses a divergent psqlrc without replacing it", () => {
  const fixture = createDeploymentFixture("optional-postgresql-collision");
  const destination = join(fixture.home, ".psqlrc");
  writeFileSync(destination, "personal\n");

  const result = runDeploymentMoon(fixture, ["home:postgresql"]);

  expect(result.exitCode).not.toBe(0);
  expect(result.stderr).toContain(
    `${destination} exists and is not the expected symbolic link`,
  );
  expect(readFileSync(destination, "utf8")).toBe("personal\n");
});

test("deploys the profile launcher and replays without replacing it", () => {
  const fixture = createDeploymentFixture("codex-profile");
  const destination = join(fixture.home, ".local/bin/codex-profile");
  expect(runDeploymentMoon(fixture, ["harness:codex-profile"]).exitCode).toBe(
    0,
  );
  const result = Bun.spawnSync([process.execPath, destination, "--help"]);
  expect(result.exitCode).toBe(0);
  expect(result.stdout.toString()).toContain("codex-profile MODEL EFFORT");
  const inode = lstatSync(destination).ino;
  expect(runDeploymentMoon(fixture, ["harness:codex-profile"])).toEqual({
    exitCode: 0,
    stdout: "",
    stderr: "",
  });
  expect(lstatSync(destination).ino).toBe(inode);
});

test("refuses a divergent profile launcher without replacing it", () => {
  const fixture = createDeploymentFixture("codex-profile-collision");
  const destination = join(fixture.home, ".local/bin/codex-profile");
  mkdirSync(join(destination, ".."), { recursive: true });
  writeFileSync(destination, "personal launcher\n");
  const result = runDeploymentMoon(fixture, ["harness:codex-profile"]);
  expect(result.exitCode).not.toBe(0);
  expect(readFileSync(destination, "utf8")).toBe("personal launcher\n");
});
