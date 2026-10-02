import {
  chmodSync,
  lstatSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  readlinkSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { join } from "node:path";
import { tmpdir } from "node:os";

type DeploymentFixture = Readonly<{
  root: string;
  home: string;
  repository: string;
  bin: string;
}>;

type CommandResult = Readonly<{
  exitCode: number;
  stdout: string;
  stderr: string;
}>;

const project = join(import.meta.dir, "..");
const provider = join(import.meta.dir, "deployment-test-provider.ts");
const fixtures: string[] = [];
const executableFileMode = 0o755;

function createDeploymentFixture(name: string): DeploymentFixture {
  const root = mkdtempSync(join(tmpdir(), `deployment-${name}-`));
  const fixture = {
    bin: join(root, "bin"),
    home: join(root, "home"),
    repository: join(root, "repository"),
    root,
  };
  fixtures.push(root);
  mkdirSync(fixture.home, { recursive: true });
  mkdirSync(fixture.repository, { recursive: true });
  mkdirSync(fixture.bin, { recursive: true });
  return fixture;
}

function cleanupDeploymentFixtures(): void {
  for (const root of fixtures.splice(0)) {
    rmSync(root, { force: true, recursive: true });
  }
}

function installProvider(fixture: DeploymentFixture, command: string): string {
  const executable = join(fixture.bin, command);
  writeFileSync(
    executable,
    `#!/usr/bin/env bun\nimport ${JSON.stringify(provider)};\n`,
  );
  chmodSync(executable, executableFileMode);
  return executable;
}

function runDeploymentHelper(
  fixture: DeploymentFixture,
  invocation: Readonly<{ helper: string; arguments: readonly string[] }>,
  environment: Readonly<NodeJS.ProcessEnv> = {},
): CommandResult {
  const result = Bun.spawnSync(
    [
      process.execPath,
      "--config=/dev/null",
      "--no-env-file",
      join(project, "tooling", invocation.helper),
      ...invocation.arguments,
    ],
    {
      cwd: fixture.root,
      env: { ...process.env, HOME: fixture.home, ...environment },
      stderr: "pipe",
      stdout: "pipe",
    },
  );
  return {
    exitCode: result.exitCode,
    stderr: decode(result.stderr),
    stdout: decode(result.stdout),
  };
}

function expectSuccess(result: CommandResult): void {
  if (result.exitCode !== 0) {
    throw new Error(
      `command failed (${result.exitCode})\nstdout:\n${result.stdout}\nstderr:\n${result.stderr}`,
    );
  }
}

function linkTarget(path: string): string {
  const metadata = lstatSync(path);
  if (!metadata.isSymbolicLink()) {
    throw new Error(`${path} is not a symlink`);
  }
  return readlinkSync(path);
}

function pathExists(path: string): boolean {
  try {
    lstatSync(path);
    return true;
  } catch (error) {
    if (error instanceof Error && "code" in error && error.code === "ENOENT") {
      return false;
    }
    throw error;
  }
}

function fileIdentity(path: string): Readonly<{
  inode: number;
  content: string;
}> {
  const metadata = lstatSync(path);
  return { content: readFileSync(path, "utf8"), inode: metadata.ino };
}

function requireCommand(command: string): string {
  const resolved = Bun.which(command);
  if (resolved === null) {
    throw new Error(`${command} is required`);
  }
  return resolved;
}

function decode(bytes: Readonly<ArrayLike<number>>): string {
  return new TextDecoder("utf-8", { fatal: true }).decode(
    Uint8Array.from(bytes),
  );
}

export {
  cleanupDeploymentFixtures,
  createDeploymentFixture,
  expectSuccess,
  fileIdentity,
  installProvider,
  linkTarget,
  pathExists,
  project,
  requireCommand,
  runDeploymentHelper,
};
export type { CommandResult, DeploymentFixture };
