import { afterEach, expect, test } from "bun:test";
import {
  cleanupDeploymentFixtures,
  createDeploymentFixture,
  expectSuccess,
  pathExists,
  runDeploymentHelper,
} from "./deployment-test-support.ts";
import {
  lstatSync,
  mkdirSync,
  readlinkSync,
  symlinkSync,
  writeFileSync,
} from "node:fs";
import { join } from "node:path";

afterEach(cleanupDeploymentFixtures);

test("creates parent directories and preserves the same link silently on replay", () => {
  const fixture = createDeploymentFixture("link-replay");
  const source = join(fixture.repository, "source");
  writeFileSync(source, "source\n");
  const destination = join(fixture.home, "nested", "link");
  expectSuccess(
    runDeploymentHelper(fixture, {
      helper: "deploy-link.ts",
      arguments: [source, destination],
    }),
  );
  expect(readlinkSync(destination)).toBe(source);
  const before = lstatSync(destination).ino;
  const replay = runDeploymentHelper(fixture, {
    helper: "deploy-link.ts",
    arguments: [source, destination],
  });
  expectSuccess(replay);
  expect(replay.stdout).toBe("");
  expect(replay.stderr).toBe("");
  expect(lstatSync(destination).ino).toBe(before);
});

test("rejects a missing source before creating a destination", () => {
  const fixture = createDeploymentFixture("missing-link-source");
  const destination = join(fixture.home, "link");
  const result = runDeploymentHelper(fixture, {
    helper: "deploy-link.ts",
    arguments: [join(fixture.repository, "absent"), destination],
  });
  expect(result.exitCode).not.toBe(0);
  expect(pathExists(destination)).toBeFalse();
});

test("rejects a divergent broken link without replacing it", () => {
  const fixture = createDeploymentFixture("broken-link");
  const destination = join(fixture.home, "link");
  symlinkSync("missing", destination);
  const result = runDeploymentHelper(fixture, {
    helper: "deploy-link.ts",
    arguments: [fixture.repository, destination],
  });
  expect(result.exitCode).not.toBe(0);
  expect(result.stderr).toContain(
    "exists and is not the expected symbolic link",
  );
  expect(readlinkSync(destination)).toBe("missing");
});

test("rejects a regular destination and a parent collision without mutation", () => {
  const fixture = createDeploymentFixture("file-collision");
  const destination = join(fixture.home, "file");
  writeFileSync(destination, "keep\n");
  const before = lstatSync(destination).ino;
  expect(
    runDeploymentHelper(fixture, {
      helper: "deploy-link.ts",
      arguments: [fixture.repository, destination],
    }).exitCode,
  ).not.toBe(0);
  expect(
    runDeploymentHelper(fixture, {
      helper: "deploy-link.ts",
      arguments: [fixture.repository, join(destination, "link")],
    }).exitCode,
  ).not.toBe(0);
  expect(lstatSync(destination).ino).toBe(before);
});

test("rejects malformed arguments before creating destinations", () => {
  const fixture = createDeploymentFixture("link-arguments");
  const destination = join(fixture.home, "link");
  mkdirSync(fixture.repository, { recursive: true });
  expect(
    runDeploymentHelper(fixture, {
      helper: "deploy-link.ts",
      arguments: ["", destination],
    }).exitCode,
  ).not.toBe(0);
  expect(pathExists(destination)).toBeFalse();
});
