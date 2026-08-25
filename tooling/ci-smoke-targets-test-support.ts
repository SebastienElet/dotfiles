import { dirname, join } from "node:path";
import { mkdirSync, mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { expect } from "bun:test";
import { tmpdir } from "node:os";

const testRoot = mkdtempSync(join(tmpdir(), "ci-smoke-targets-"));
const selector = join(import.meta.dir, "ci-smoke-targets");
const decoder = new TextDecoder();

type Result = Readonly<{
  exitCode: number;
  stderr: string;
  stdout: string;
}>;

function run(command: readonly string[], cwd: string): Result {
  const result = Bun.spawnSync({
    cmd: [...command],
    cwd,
    env: process.env,
    stderr: "pipe",
    stdout: "pipe",
  });

  return {
    exitCode: result.exitCode,
    stderr: decoder.decode(result.stderr),
    stdout: decoder.decode(result.stdout),
  };
}

function git(
  repository: string,
  ...commandArguments: readonly string[]
): string {
  const result = run(["git", ...commandArguments], repository);
  expect(result.stderr).toBe("");
  expect(result.exitCode).toBe(0);
  return result.stdout.trim();
}

function write(repository: string, path: string, contents: string): void {
  const destination = join(repository, path);
  mkdirSync(dirname(destination), { recursive: true });
  writeFileSync(destination, contents);
}

function commit(repository: string, message: string): string {
  git(repository, "add", "-A");
  git(repository, "commit", "-q", "-m", message);
  return git(repository, "rev-parse", "HEAD");
}

function newRepository(name: string): string {
  const repository = join(testRoot, name);
  mkdirSync(repository);
  git(repository, "init", "-q");
  git(repository, "config", "user.email", "ci@example.test");
  git(repository, "config", "user.name", "CI");
  write(
    repository,
    "Makefile",
    [
      "SHARED=value",
      ".PHONY: all",
      "all: alpha beta",
      ".PHONY: alpha",
      "alpha:",
      "\t@echo alpha",
      ".PHONY: beta",
      "beta:",
      "\t@echo beta",
      "",
    ].join("\n"),
  );
  write(repository, "install.sh", "#!/usr/bin/env bash\nmake all\n");
  commit(repository, "initial");
  return repository;
}

function select(
  repository: string,
  ...commandArguments: readonly string[]
): Result {
  return run([selector, ...commandArguments], repository);
}

function expectTargets(repository: string, expected: readonly string[]): void {
  const result = select(repository, "HEAD^", "HEAD");
  expect(result).toEqual({
    exitCode: 0,
    stderr: "",
    stdout: `${JSON.stringify(expected)}\n`,
  });
}

function makefileWithAmbiguousRule(rule: string, recipe: string): string {
  return [
    "SHARED=value",
    ".PHONY: all",
    "all: alpha beta",
    ".PHONY: alpha",
    "alpha:",
    "\t@echo alpha",
    ".PHONY: beta",
    "beta:",
    "\t@echo beta",
    rule,
    `\t@echo ${recipe}`,
    "",
  ].join("\n");
}

function cleanup(): void {
  rmSync(testRoot, { force: true, recursive: true });
}

export {
  cleanup,
  commit,
  expectTargets,
  git,
  makefileWithAmbiguousRule,
  newRepository,
  select,
  write,
};
