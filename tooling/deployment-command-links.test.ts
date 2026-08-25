import { afterEach, expect, test } from "bun:test";
import {
  cleanupDeploymentFixtures,
  createDeploymentFixture,
  expectSuccess,
  linkTarget,
  pathExists,
  project,
  requireCommand,
  runMake,
} from "./deployment-test-support.ts";
import { dirname, join } from "node:path";
import { mkdirSync, symlinkSync, utimesSync, writeFileSync } from "node:fs";

afterEach(cleanupDeploymentFixtures);

const FUTURE_MTIME_OFFSET_MILLISECONDS = 60_000;
const bunDirectory = dirname(requireCommand("bun"));

test("migrates the former CodeGraph measurement command link", () => {
  const fixture = createDeploymentFixture("codegraph-command-migration");
  const { repository } = fixture;
  const localBin = join(fixture.home, ".local", "bin");
  const destination = join(localBin, "codegraph-repository-size");
  const retiredTarget = join(
    repository,
    "harness",
    "skills",
    "codegraph",
    "scripts",
    "measure_repository.sh",
  );
  const currentTarget = join(
    repository,
    "tooling",
    "codegraph-repository-size",
  );
  const cleanupCommand = join(repository, "tooling", "retire-command-link");
  mkdirSync(join(repository, "tooling"), { recursive: true });
  mkdirSync(dirname(retiredTarget), { recursive: true });
  mkdirSync(localBin, { recursive: true });
  writeFileSync(currentTarget, "current\n");
  writeFileSync(retiredTarget, "retired\n");
  symlinkSync(join(project, "tooling", "retire-command-link"), cleanupCommand);
  const future = new Date(Date.now() + FUTURE_MTIME_OFFSET_MILLISECONDS);
  utimesSync(retiredTarget, future, future);
  symlinkSync(retiredTarget, destination);

  expectSuccess(
    runMake(fixture, [destination], {
      repository,
      variables: { BREW_BIN: bunDirectory },
    }),
  );

  expect(linkTarget(destination)).toBe(currentTarget);
  expectSuccess(
    runMake(fixture, [destination], {
      repository,
      variables: { BREW_BIN: bunDirectory },
    }),
  );
});

test("refuses a newline-suffixed CodeGraph command link", () => {
  const fixture = createDeploymentFixture("codegraph-command-unknown");
  const localBin = join(fixture.home, ".local", "bin");
  const destination = join(localBin, "codegraph-repository-size");
  const unexpected = `${join(
    project,
    "harness",
    "skills",
    "codegraph",
    "scripts",
    "measure_repository.sh",
  )}\n`;
  mkdirSync(localBin, { recursive: true });
  symlinkSync(unexpected, destination);

  const result = runMake(fixture, [destination], {
    repository: project,
    variables: { BREW_BIN: bunDirectory },
  });

  expect(result.exitCode).not.toBe(0);
  expect(linkTarget(destination)).toBe(unexpected);
});

test("removes only the retired Claude developer command link", () => {
  const fixture = createDeploymentFixture("claude-developer-retired");
  const localBin = join(fixture.home, ".local", "bin");
  const destination = join(localBin, "claude-developer");
  mkdirSync(localBin, { recursive: true });
  symlinkSync(join(project, "tooling", "claude-developer"), destination);

  expectSuccess(
    runMake(fixture, ["claude-developer-link-cleanup"], {
      repository: project,
      variables: { BREW_BIN: bunDirectory },
    }),
  );

  expect(pathExists(destination)).toBeFalse();
  expectSuccess(
    runMake(fixture, ["claude-developer-link-cleanup"], {
      repository: project,
      variables: { BREW_BIN: bunDirectory },
    }),
  );
});

test("removes the retired link from a destination containing an apostrophe", () => {
  const fixture = createDeploymentFixture("claude-developer-apostrophe");
  const localBin = join(fixture.home, "local'bin");
  const destination = join(localBin, "claude-developer");
  mkdirSync(localBin, { recursive: true });
  symlinkSync(join(project, "tooling", "claude-developer"), destination);

  expectSuccess(
    runMake(fixture, ["claude-developer-link-cleanup"], {
      repository: project,
      variables: { BREW_BIN: bunDirectory, LOCAL_BIN: localBin },
    }),
  );

  expect(pathExists(destination)).toBeFalse();
});

test("preserves a newline-suffixed link during retired command cleanup", () => {
  const fixture = createDeploymentFixture("claude-developer-preserved");
  const localBin = join(fixture.home, ".local", "bin");
  const destination = join(localBin, "claude-developer");
  const unexpected = `${join(project, "tooling", "claude-developer")}\n`;
  mkdirSync(localBin, { recursive: true });
  symlinkSync(unexpected, destination);

  expectSuccess(
    runMake(fixture, ["claude-developer-link-cleanup"], {
      repository: project,
      variables: { BREW_BIN: bunDirectory },
    }),
  );
  expect(linkTarget(destination)).toBe(unexpected);
});

test("preserves a regular file during retired command cleanup", () => {
  const fixture = createDeploymentFixture("claude-developer-file");
  const localBin = join(fixture.home, ".local", "bin");
  const destination = join(localBin, "claude-developer");
  mkdirSync(localBin, { recursive: true });
  writeFileSync(destination, "keep\n");

  expectSuccess(
    runMake(fixture, ["claude-developer-link-cleanup"], {
      repository: project,
      variables: { BREW_BIN: bunDirectory },
    }),
  );

  expect(pathExists(destination)).toBeTrue();
});

test("preserves a directory during retired command cleanup", () => {
  const fixture = createDeploymentFixture("claude-developer-directory");
  const destination = join(fixture.home, ".local", "bin", "claude-developer");
  mkdirSync(destination, { recursive: true });

  expectSuccess(
    runMake(fixture, ["claude-developer-link-cleanup"], {
      repository: project,
      variables: { BREW_BIN: bunDirectory },
    }),
  );

  expect(pathExists(destination)).toBeTrue();
});

test("runs retired command cleanup from the Codex aggregate", () => {
  const fixture = createDeploymentFixture("claude-developer-wiring");
  const result = runMake(fixture, ["codex"], {
    dryRun: true,
    repository: project,
    variables: { BREW_BIN: bunDirectory },
  });

  expectSuccess(result);
  expect(result.stdout).toContain(
    join(fixture.home, ".local", "bin", "claude-developer"),
  );
});
