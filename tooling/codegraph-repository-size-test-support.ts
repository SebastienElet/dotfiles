import {
  chmodSync,
  mkdirSync,
  readFileSync,
  rmSync,
  symlinkSync,
} from "node:fs";
import { dirname, join } from "node:path";
import { tmpdir } from "node:os";
import { z } from "zod";

const argumentsSchema = z.array(z.string());

const root = dirname(import.meta.dir);
const entryPoint = join(import.meta.dir, "codegraph-repository-size");
const provider = join(
  import.meta.dir,
  "codegraph-repository-size-test-provider.ts",
);
const temporaryDirectories: string[] = [];
const executableFileMode = 0o755;

interface CommandResult {
  exitCode: number;
  stderr: string;
  stdout: string;
}
interface Fixture {
  argumentsLog: string;
  directory: string;
  environment: NodeJS.ProcessEnv;
  fakeGit: string;
  fakeTokei: string;
  repository: string;
}

function createFixture(): Fixture {
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
  chmodSync(provider, executableFileMode);
  symlinkSync(process.execPath, join(binaries, "bun"));
  symlinkSync(provider, join(binaries, "tokei"));
  symlinkSync(provider, join(binaries, "git"));
  return {
    argumentsLog,
    directory,
    environment: {
      ...process.env,
      CODEGRAPH_GIT_BIN: Bun.which("git") ?? "git",
      CODEGRAPH_TEST_ARGUMENTS_LOG: argumentsLog,
      CODEGRAPH_TOKEI_BIN: join(binaries, "tokei"),
      PATH: `${binaries}:${process.env.PATH ?? ""}`,
    },
    fakeGit: join(binaries, "git"),
    fakeTokei: join(binaries, "tokei"),
    repository,
  };
}

function runEntryPoint(
  repository: string,
  environment: Readonly<Record<string, string | undefined>>,
): CommandResult {
  const result = Bun.spawnSync([entryPoint, repository], {
    cwd: root,
    env: environment,
    stderr: "pipe",
    stdout: "pipe",
  });
  return {
    exitCode: result.exitCode,
    stderr: result.stderr.toString(),
    stdout: result.stdout.toString(),
  };
}

function run(command: readonly string[], cwd?: string): CommandResult {
  const options = {
    stderr: "pipe",
    stdout: "pipe",
  } as const;
  const result = Bun.spawnSync(
    [...command],
    cwd === undefined ? options : { ...options, cwd },
  );
  return {
    exitCode: result.exitCode,
    stderr: result.stderr.toString(),
    stdout: result.stdout.toString(),
  };
}

function readArguments(path: string): string[][] {
  try {
    return readFileSync(path, "utf8")
      .trimEnd()
      .split("\n")
      .map((line) => argumentsSchema.parse(JSON.parse(line)));
  } catch {
    return [];
  }
}

function cleanupFixtures(): void {
  for (const path of temporaryDirectories.splice(0)) {
    rmSync(path, { force: true, recursive: true });
  }
}

export { cleanupFixtures, createFixture, readArguments, run, runEntryPoint };
