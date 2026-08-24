import {
  chmodSync,
  mkdirSync,
  readFileSync,
  rmSync,
  symlinkSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";

const root = dirname(import.meta.dir);
const entryPoint = join(import.meta.dir, "codegraph-repository-size");
const provider = join(
  import.meta.dir,
  "codegraph-repository-size-test-provider.ts",
);
const temporaryDirectories: string[] = [];

export function createFixture() {
  const directory = join(
    tmpdir(),
    `codegraph-repository-size-${crypto.randomUUID()}`,
  );
  const repository = join(directory, "repository");
  const binaries = join(directory, "bin");
  const argumentsLog = join(directory, "arguments.log");
  temporaryDirectories.push(directory);
  mkdirSync(repository, { recursive: true });
  mkdirSync(binaries, { recursive: true });
  chmodSync(provider, 0o755);
  symlinkSync(process.execPath, join(binaries, "bun"));
  symlinkSync(provider, join(binaries, "tokei"));
  symlinkSync(provider, join(binaries, "git"));
  return {
    directory,
    repository,
    argumentsLog,
    fakeGit: join(binaries, "git"),
    fakeTokei: join(binaries, "tokei"),
    environment: {
      ...process.env,
      CODEGRAPH_TOKEI_BIN: join(binaries, "tokei"),
      CODEGRAPH_GIT_BIN: Bun.which("git") ?? "git",
      CODEGRAPH_TEST_ARGUMENTS_LOG: argumentsLog,
      PATH: `${binaries}:${process.env.PATH ?? ""}`,
    },
  };
}

export function runEntryPoint(
  repository: string,
  environment: Record<string, string | undefined>,
) {
  const result = Bun.spawnSync([entryPoint, repository], {
    cwd: root,
    env: environment,
    stdout: "pipe",
    stderr: "pipe",
  });
  return {
    exitCode: result.exitCode,
    stdout: result.stdout.toString(),
    stderr: result.stderr.toString(),
  };
}

export function run(command: string[], cwd?: string) {
  const options = {
    stdout: "pipe",
    stderr: "pipe",
  } as const;
  const result = Bun.spawnSync(
    command,
    cwd === undefined ? options : { ...options, cwd },
  );
  return {
    exitCode: result.exitCode,
    stdout: result.stdout.toString(),
    stderr: result.stderr.toString(),
  };
}

export function readArguments(path: string): string[][] {
  try {
    return readFileSync(path, "utf8")
      .trimEnd()
      .split("\n")
      .map((line) => JSON.parse(line) as string[]);
  } catch {
    return [];
  }
}

export function cleanupFixtures(): void {
  for (const path of temporaryDirectories.splice(0)) {
    rmSync(path, { force: true, recursive: true });
  }
}
