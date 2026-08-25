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
type CommandLinkFixture = Parameters<typeof runMake>[0];
type CommandLinkResult = ReturnType<typeof runMake>;

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
    runCommandLinkMake(fixture, [destination], {
      repository,
    }),
  );

  expect(linkTarget(destination)).toBe(currentTarget);
  expectSuccess(
    runCommandLinkMake(fixture, [destination], {
      repository,
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

  const result = runCommandLinkMake(fixture, [destination], {
    repository: project,
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
    runCommandLinkMake(fixture, ["claude-developer-link-cleanup"], {
      repository: project,
    }),
  );

  expect(pathExists(destination)).toBeFalse();
  expectSuccess(
    runCommandLinkMake(fixture, ["claude-developer-link-cleanup"], {
      repository: project,
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
    runCommandLinkMake(fixture, ["claude-developer-link-cleanup"], {
      repository: project,
      variables: { LOCAL_BIN: localBin },
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
    runCommandLinkMake(fixture, ["claude-developer-link-cleanup"], {
      repository: project,
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
    runCommandLinkMake(fixture, ["claude-developer-link-cleanup"], {
      repository: project,
    }),
  );

  expect(pathExists(destination)).toBeTrue();
});

test("preserves a directory during retired command cleanup", () => {
  const fixture = createDeploymentFixture("claude-developer-directory");
  const destination = join(fixture.home, ".local", "bin", "claude-developer");
  mkdirSync(destination, { recursive: true });

  expectSuccess(
    runCommandLinkMake(fixture, ["claude-developer-link-cleanup"], {
      repository: project,
    }),
  );

  expect(pathExists(destination)).toBeTrue();
});

test("runs retired command cleanup from the Codex aggregate", () => {
  const fixture = createDeploymentFixture("claude-developer-wiring");
  const result = runCommandLinkMake(fixture, ["codex"], {
    dryRun: true,
    repository: project,
  });

  expectSuccess(result);
  expect(result.stdout).toContain(
    join(fixture.home, ".local", "bin", "claude-developer"),
  );
});

function runCommandLinkMake(
  fixture: CommandLinkFixture,
  targets: readonly string[],
  options: Readonly<{
    dryRun?: boolean;
    repository?: string;
    variables?: Readonly<Record<string, string>>;
  }> = {},
): CommandLinkResult {
  const bun = join(fixture.bin, "bun");
  const brew = join(fixture.bin, "brew");
  if (!pathExists(bun)) {
    symlinkSync(requireCommand("bun"), bun);
  }
  if (!pathExists(brew)) {
    symlinkSync(requireCommand("true"), brew);
  }
  return runMake(fixture, targets, {
    ...options,
    variables: { BREW_BIN: fixture.bin, ...options.variables },
  });
}
