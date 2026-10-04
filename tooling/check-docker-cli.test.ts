import { afterEach, expect, test } from "bun:test";
import {
  chmodSync,
  mkdirSync,
  mkdtempSync,
  rmSync,
  symlinkSync,
  writeFileSync,
} from "node:fs";
import { join } from "node:path";
import { tmpdir } from "node:os";

const checker = join(import.meta.dir, "check-docker-cli");
const fixtures: string[] = [];
const executableMode = 0o755;
const unavailableMessage = "Error: Docker CLI unavailable\n";

afterEach(() => {
  for (const directory of fixtures.splice(0)) {
    rmSync(directory, { recursive: true, force: true });
  }
});

function fixture(withInterpreter = true): string {
  const directory = mkdtempSync(join(tmpdir(), "docker-cli-"));
  fixtures.push(directory);
  if (withInterpreter) {
    symlinkSync("/bin/bash", join(directory, "bash"));
  }
  return directory;
}

function run(
  directory: string,
  shellSetup = "",
): Bun.SyncSubprocess<"pipe", "pipe"> {
  const command = shellSetup
    ? ["/bin/bash", "-c", `${shellSetup}; exec "$1"`, "fixture", checker]
    : [checker];
  return Bun.spawnSync(command, {
    env: { PATH: directory },
    stdout: "pipe",
    stderr: "pipe",
  });
}

test("refuses a missing Docker executable", () => {
  const result = run(fixture());
  expect(result.exitCode).toBe(1);
  expect(result.stderr.toString()).toBe(unavailableMessage);
});

test("refuses a non-executable Docker file", () => {
  const directory = fixture();
  writeFileSync(join(directory, "docker"), "unusable");
  const result = run(directory);
  expect(result.exitCode).toBe(1);
  expect(result.stderr.toString()).toBe(unavailableMessage);
});

test("refuses an exported shell function without an external Docker executable", () => {
  const result = run(fixture(), "docker() { :; }; export -f docker");
  expect(result.exitCode).toBe(1);
  expect(result.stderr.toString()).toBe(unavailableMessage);
});

test("accepts an external Docker executable without invoking it", () => {
  const directory = fixture();
  const docker = join(directory, "docker");
  writeFileSync(docker, "#!/bin/sh\nexit 1\n");
  chmodSync(docker, executableMode);
  const result = run(directory);
  expect(result.exitCode).toBe(0);
  expect(result.stdout.toString()).toBe("");
  expect(result.stderr.toString()).toBe("");
});

test("fails when the required Bash interpreter is absent", () => {
  const result = run(fixture(false));
  expect(result.exitCode).not.toBe(0);
  expect(result.stderr.toString()).toContain("bash");
});

test("refuses an exported test function that reports false success", () => {
  const result = run(fixture(), "test() { return 0; }; export -f test");
  expect(result.exitCode).toBe(1);
  expect(result.stderr.toString()).toBe(unavailableMessage);
});

test("refuses an exported builtin function that spoofs the lookup", () => {
  const result = run(
    fixture(),
    "builtin() { printf /bin/bash; }; export -f builtin",
  );
  expect(result.exitCode).toBe(1);
  expect(result.stderr.toString()).toBe(unavailableMessage);
});

test("refuses exported lookup and diagnostic functions", () => {
  const result = run(
    fixture(),
    "which() { printf /bin/bash; }; echo() { :; }; export -f which echo",
  );
  expect(result.exitCode).toBe(1);
  expect(result.stderr.toString()).toBe(unavailableMessage);
});

test("refuses a shell alias without an external Docker executable", () => {
  const result = run(fixture(), "alias docker=/bin/bash");
  expect(result.exitCode).toBe(1);
  expect(result.stderr.toString()).toBe(unavailableMessage);
});

test("refuses an executable directory named Docker", () => {
  const directory = fixture();
  mkdirSync(join(directory, "docker"));
  const result = run(directory);
  expect(result.exitCode).toBe(1);
  expect(result.stderr.toString()).toBe(unavailableMessage);
});

test("finds a later executable after unusable PATH entries", () => {
  const directory = fixture();
  mkdirSync(join(directory, "docker"));
  const nonExecutable = fixture();
  writeFileSync(join(nonExecutable, "docker"), "unusable");
  const valid = fixture();
  symlinkSync("/bin/bash", join(valid, "docker"));
  const result = run(`${directory}:${nonExecutable}:${valid}`);
  expect(result.exitCode).toBe(0);
  expect(result.stderr.toString()).toBe("");
});
