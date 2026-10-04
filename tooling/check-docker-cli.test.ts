import { afterEach, expect, test } from "bun:test";
import {
  chmodSync,
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
  exportedFunction = false,
): Bun.SyncSubprocess<"pipe", "pipe"> {
  const command = exportedFunction
    ? [
        "/bin/bash",
        "-c",
        'docker() { :; }; export -f docker; exec "$1"',
        "fixture",
        checker,
      ]
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
  const result = run(fixture(), true);
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
