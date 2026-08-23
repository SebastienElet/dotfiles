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
import { tmpdir } from "node:os";
import { join } from "node:path";

export type DeploymentFixture = Readonly<{
  root: string;
  home: string;
  repository: string;
  bin: string;
}>;

export type CommandResult = Readonly<{
  exitCode: number;
  stdout: string;
  stderr: string;
}>;

const project = join(import.meta.dir, "..");
const makefile = join(project, "Makefile");
const provider = join(import.meta.dir, "deployment-test-provider.ts");
const fixtures: string[] = [];

export function createDeploymentFixture(name: string): DeploymentFixture {
  const root = mkdtempSync(join(tmpdir(), `deployment-${name}-`));
  const fixture = {
    root,
    home: join(root, "home"),
    repository: join(root, "repository"),
    bin: join(root, "bin"),
  };
  fixtures.push(root);
  mkdirSync(fixture.home, { recursive: true });
  mkdirSync(fixture.repository, { recursive: true });
  mkdirSync(fixture.bin, { recursive: true });
  return fixture;
}

export function cleanupDeploymentFixtures(): void {
  for (const root of fixtures.splice(0)) {
    rmSync(root, { recursive: true, force: true });
  }
}

export function runMake(
  fixture: DeploymentFixture,
  targets: readonly string[],
  options: Readonly<{
    repository?: string;
    environment?: NodeJS.ProcessEnv;
    variables?: Readonly<Record<string, string>>;
    dryRun?: boolean;
    cwd?: string;
    make?: string;
  }> = {},
): CommandResult {
  const repository = options.repository ?? fixture.repository;
  const result = Bun.spawnSync(
    [
      options.make ?? requireCommand("make"),
      ...(options.dryRun === true ? ["-n"] : []),
      "-f",
      makefile,
      `DOTFILES_PATH=${repository}`,
      ...Object.entries(options.variables ?? {}).map(
        ([name, value]) => `${name}=${value}`,
      ),
      ...targets,
    ],
    {
      cwd: options.cwd ?? fixture.root,
      env: {
        ...process.env,
        HOME: fixture.home,
        ...options.environment,
      },
      stdout: "pipe",
      stderr: "pipe",
    },
  );
  return {
    exitCode: result.exitCode,
    stdout: decode(result.stdout),
    stderr: decode(result.stderr),
  };
}

export function installProvider(
  fixture: DeploymentFixture,
  command: string,
): string {
  const executable = join(fixture.bin, command);
  writeFileSync(
    executable,
    `#!/usr/bin/env bun\nimport ${JSON.stringify(provider)};\n`,
  );
  chmodSync(executable, 0o755);
  return executable;
}

export function expectSuccess(result: CommandResult): void {
  if (result.exitCode !== 0) {
    throw new Error(
      `command failed (${result.exitCode})\nstdout:\n${result.stdout}\nstderr:\n${result.stderr}`,
    );
  }
}

export function linkTarget(path: string): string {
  const metadata = lstatSync(path);
  if (!metadata.isSymbolicLink()) throw new Error(`${path} is not a symlink`);
  return readlinkSync(path);
}

export function pathExists(path: string): boolean {
  try {
    lstatSync(path);
    return true;
  } catch (error) {
    if ((error as NodeJS.ErrnoException).code === "ENOENT") return false;
    throw error;
  }
}

export function fileIdentity(path: string): Readonly<{
  inode: number;
  content: string;
}> {
  const metadata = lstatSync(path);
  return { inode: metadata.ino, content: readFileSync(path, "utf8") };
}

export function requireCommand(command: string): string {
  const resolved = Bun.which(command);
  if (resolved === null) throw new Error(`${command} is required`);
  return resolved;
}

function decode(bytes: Uint8Array): string {
  return new TextDecoder("utf-8", { fatal: true }).decode(bytes);
}

export { project };
