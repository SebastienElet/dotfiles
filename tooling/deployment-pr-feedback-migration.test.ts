import { afterEach, expect, test } from "bun:test";
import {
  cleanupDeploymentFixtures,
  createDeploymentFixture,
  expectSuccess,
  linkTarget,
  pathExists,
  runMake,
} from "./deployment-test-support.ts";
import { mkdirSync, readFileSync, symlinkSync, writeFileSync } from "node:fs";
import { join } from "node:path";

afterEach(cleanupDeploymentFixtures);

test("removes the legacy PR feedback command link idempotently", () => {
  const fixture = createDeploymentFixture("pr-feedback-command-migration");
  const commands = join(fixture.home, ".claude", "commands");
  const legacyCommand = join(commands, "pr-feedback.md");
  mkdirSync(commands, { recursive: true });
  symlinkSync(
    join(fixture.repository, "harness", "commands", "pr-feedback.md"),
    legacyCommand,
  );

  expectSuccess(runMake(fixture, ["claude-pr-feedback-migration"]));
  expect(pathExists(legacyCommand)).toBeFalse();
  expectSuccess(runMake(fixture, ["claude-pr-feedback-migration"]));
  expect(pathExists(legacyCommand)).toBeFalse();
});

test("preserves an unexpected PR feedback command link", () => {
  const fixture = createDeploymentFixture("pr-feedback-command-conflict");
  const commands = join(fixture.home, ".claude", "commands");
  const legacyCommand = join(commands, "pr-feedback.md");
  const unexpected = join(fixture.root, "unexpected.md");
  mkdirSync(commands, { recursive: true });
  writeFileSync(unexpected, "personal command");
  symlinkSync(unexpected, legacyCommand);

  const result = runMake(fixture, ["claude-pr-feedback-migration"]);

  expect(result.exitCode).not.toBe(0);
  expect(linkTarget(legacyCommand)).toBe(unexpected);
  expect(readFileSync(unexpected, "utf8")).toBe("personal command");
});

test("preserves a regular PR feedback command file", () => {
  const fixture = createDeploymentFixture("pr-feedback-command-file");
  const commands = join(fixture.home, ".claude", "commands");
  const legacyCommand = join(commands, "pr-feedback.md");
  mkdirSync(commands, { recursive: true });
  writeFileSync(legacyCommand, "personal command");

  const result = runMake(fixture, ["claude-pr-feedback-migration"]);

  expect(result.exitCode).not.toBe(0);
  expect(readFileSync(legacyCommand, "utf8")).toBe("personal command");
});
