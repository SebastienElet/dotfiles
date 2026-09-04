import {
  type CommandResult,
  expectSuccess,
  project,
  requireCommand,
} from "./deployment-test-support.ts";
import { cpSync, mkdirSync, mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { homedir, tmpdir } from "node:os";
import { join } from "node:path";

type MoonDeploymentFixture = Readonly<{
  home: string;
  repository: string;
  root: string;
}>;

type RunMoonOptions = Readonly<{
  cache?: "off" | "read-write";
  environment?: Readonly<NodeJS.ProcessEnv>;
  force?: boolean;
}>;

type MoonProjectFixturePaths = Readonly<{
  destination: string;
  home: string;
  projectId: "agent-memory" | "agent-handoff";
  repository: string;
  source: string;
}>;

const fixtures: string[] = [];

function createMoonDeploymentFixture(
  projectId: "agent-memory" | "agent-handoff",
): MoonDeploymentFixture {
  const root = mkdtempSync(join(tmpdir(), `moon-${projectId}-deployment-`));
  const repository = join(root, "repository");
  const home = join(root, "home");
  const source = join(project, "tooling", projectId);
  const destination = join(repository, "tooling", projectId);
  fixtures.push(root);
  copyMoonProjectFixture({
    destination,
    home,
    projectId,
    repository,
    source,
  });
  initializeGitRepository(repository);
  return { home, repository, root };
}

function copyMoonProjectFixture({
  destination,
  home,
  projectId,
  repository,
  source,
}: MoonProjectFixturePaths): void {
  mkdirSync(join(repository, ".moon"), { recursive: true });
  mkdirSync(join(repository, ".github", "workflows"), { recursive: true });
  mkdirSync(home);
  cpSync(source, destination, {
    filter: (path) => !path.includes(`${join(source, "target")}/`),
    recursive: true,
  });
  cpSync(
    join(project, ".github", "workflows", `test-${projectId}.yml`),
    join(repository, ".github", "workflows", `test-${projectId}.yml`),
  );
  writeFileSync(
    join(repository, ".moon", "workspace.yml"),
    `projects:
  repository: .
  ${projectId}: tooling/${projectId}

defaultProject: repository

vcs:
  defaultBranch: main
`,
  );
  writeFileSync(
    join(repository, "moon.yml"),
    "tasks:\n  rust:\n    command: noop\n    toolchains: system\n    options:\n      cache: false\n      runInCI: skip\n",
  );
}

function initializeGitRepository(repository: string): void {
  expectSuccess(
    spawn([requireCommand("git"), "init", "--initial-branch=main"], {
      cwd: repository,
      environment: process.env,
    }),
  );
  expectSuccess(
    spawn([requireCommand("git"), "add", "."], {
      cwd: repository,
      environment: process.env,
    }),
  );
  expectSuccess(
    spawn(
      [
        requireCommand("git"),
        "-c",
        "user.name=Moon deployment test",
        "-c",
        "user.email=moon-deployment@example.invalid",
        "commit",
        "-m",
        "fixture",
      ],
      { cwd: repository, environment: process.env },
    ),
  );
}

function cleanupMoonDeploymentFixtures(): void {
  for (const root of fixtures.splice(0)) {
    rmSync(root, { force: true, recursive: true });
  }
}

function runMoon(
  fixture: MoonDeploymentFixture,
  target: string,
  options: RunMoonOptions = {},
): CommandResult {
  return spawn(
    [
      requireCommand("moon"),
      "exec",
      "--quiet",
      "--ignore-ci-checks",
      "--no-actions",
      "--cache",
      options.cache ?? "read-write",
      ...(options.force === true ? ["--force"] : []),
      target,
    ],
    {
      cwd: fixture.repository,
      environment: {
        ...withoutMoonTaskContext(process.env),
        CARGO_HOME: process.env.CARGO_HOME ?? join(homedir(), ".cargo"),
        HOME: fixture.home,
        MOON_HOME: join(fixture.root, "moon-home"),
        RUSTUP_HOME: process.env.RUSTUP_HOME ?? join(homedir(), ".rustup"),
        ...options.environment,
      },
    },
  );
}

function withoutMoonTaskContext(
  environment: Readonly<NodeJS.ProcessEnv>,
): NodeJS.ProcessEnv {
  return Object.fromEntries(
    Object.entries(environment).filter(
      (entry: readonly [string, string | undefined]) =>
        !entry[0].startsWith("MOON_"),
    ),
  );
}

function spawn(
  command: readonly string[],
  options: Readonly<{
    cwd: string;
    environment: Readonly<NodeJS.ProcessEnv>;
  }>,
): CommandResult {
  const result = Bun.spawnSync([...command], {
    cwd: options.cwd,
    env: options.environment,
    stderr: "pipe",
    stdout: "pipe",
  });
  return {
    exitCode: result.exitCode,
    stderr: result.stderr.toString(),
    stdout: result.stdout.toString(),
  };
}

export { cleanupMoonDeploymentFixtures, createMoonDeploymentFixture, runMoon };
export type { MoonDeploymentFixture };
