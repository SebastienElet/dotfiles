import {
  MeasurementError,
  type RepositoryMeasurement,
  type SourceMeasurement,
  aggregateMeasurements,
  parseTokeiOutput,
} from "./codegraph-repository-measurement.ts";
import {
  accessSync,
  constants,
  lstatSync,
  mkdtempSync,
  realpathSync,
  rmSync,
  symlinkSync,
} from "node:fs";
import {
  gitOpenTofuFiles,
  nonGitOpenTofuFiles,
} from "./codegraph-repository-files.ts";
import { join } from "node:path";
import { tmpdir } from "node:os";

const types =
  "Ark TypeScript,Astro,C,C Header,C#,COBOL,ColdFusion,ColdFusion CFScript,C++,C++ Header,C++ Module,CUDA,Dart,Erlang,Go,HCL,Java,JavaScript,JSX,Kotlin,Liquid,Lua,Metal Shading Language,Nix,Objective-C,Objective-C++,Pascal,PHP,Python,R,Razor,Ruby,Rust,Scala,Solidity,Svelte,Swift,TSX,TypeScript,Visual Basic,Vue";
const exclusions = [
  ".git",
  ".codegraph",
  ".worktrees",
  "node_modules",
  "vendor",
  "dist",
  "build",
  "out",
  "target",
  "coverage",
  "generated",
  "docs",
  "fixtures",
  "*.lock",
  "*.min.js",
];
const commandArgumentOffset = 2;
const gitNotRepositoryExitCode = 128;
const tokeiBatchSize = 200;

interface CommandResult {
  readonly exitCode: number;
  readonly stderr: string;
  readonly stdout: string;
}

function main(
  arguments_: readonly string[] = process.argv.slice(commandArgumentOffset),
): number {
  try {
    const measurement = measureRepository(arguments_[0] ?? ".");
    process.stdout.write(`${JSON.stringify(measurement)}\n`);
    return 0;
  } catch (error) {
    process.stderr.write(`${message(error)}\n`);
    return error instanceof MeasurementError ? error.exitCode : 1;
  }
}

function measureRepository(input: string): RepositoryMeasurement {
  const tokei = requireExecutable(
    process.env.CODEGRAPH_TOKEI_BIN ?? "tokei",
    "Tokei",
  );
  const git = requireExecutable(process.env.CODEGRAPH_GIT_BIN ?? "git", "Git");
  const repository = resolveRepository(input);
  const gitRepository = isGitRepository(git, repository);
  const tofuFiles = gitRepository
    ? gitOpenTofuFiles(repository, listGitFiles(git, repository))
    : nonGitOpenTofuFiles(repository);
  const measurements = runTokei(tokei, [repository, ...tokeiArguments()]);
  return aggregateMeasurements([
    ...measurements,
    ...measureOpenTofu(tokei, tofuFiles),
  ]);
}

function resolveRepository(input: string): string {
  const repository = resolveAccessiblePath(input);
  if (!lstatSync(repository).isDirectory()) {
    throw new MeasurementError(`repository is not a directory: ${repository}`);
  }
  try {
    accessSync(repository, constants.R_OK | constants.X_OK);
  } catch {
    throw new MeasurementError(`repository is not readable: ${repository}`);
  }
  return repository;
}

function resolveAccessiblePath(input: string): string {
  try {
    return realpathSync.native(input);
  } catch (error) {
    throw new MeasurementError(
      `repository is not accessible: ${message(error)}`,
    );
  }
}

function isGitRepository(git: string, repository: string): boolean {
  const result = runCommand(
    git,
    ["-C", repository, "rev-parse", "--is-inside-work-tree"],
    {
      ...process.env,
      LC_ALL: "C",
    },
  );
  if (result.exitCode === 0) {
    if (result.stdout.trimEnd() !== "true") {
      throw new MeasurementError("git rev-parse returned an unexpected result");
    }
    return true;
  }
  if (
    result.exitCode === gitNotRepositoryExitCode &&
    result.stderr.startsWith("fatal: not a git repository")
  ) {
    return false;
  }
  throw commandError(result);
}

function listGitFiles(git: string, repository: string): string {
  const result = runCommand(git, [
    "-C",
    repository,
    "ls-files",
    "--cached",
    "--others",
    "--exclude-standard",
    "-z",
  ]);
  if (result.exitCode !== 0) {
    throw commandError(result);
  }
  return result.stdout;
}

function measureOpenTofu(
  tokei: string,
  files: readonly string[],
): SourceMeasurement[] {
  if (files.length === 0) {
    return [];
  }
  const directory = mkdtempSync(join(tmpdir(), "codegraph-tofu-"));
  try {
    const links = files.map((file, index) => {
      const link = join(directory, `${index + 1}.tf`);
      symlinkSync(file, link);
      return link;
    });
    return chunk(links, tokeiBatchSize).flatMap((batch: readonly string[]) =>
      runTokei(tokei, [...batch, "--streaming", "json", "--types", "HCL"]),
    );
  } finally {
    rmSync(directory, { force: true, recursive: true });
  }
}

function tokeiArguments(): string[] {
  return [
    "--hidden",
    "--streaming",
    "json",
    "--types",
    types,
    ...exclusions.flatMap((exclusion) => ["--exclude", exclusion]),
  ];
}

function runTokei(
  tokei: string,
  arguments_: readonly string[],
): SourceMeasurement[] {
  const result = runCommand(tokei, arguments_);
  if (result.exitCode !== 0) {
    throw commandError(result);
  }
  if (result.stderr !== "") {
    process.stderr.write(result.stderr);
  }
  return parseTokeiOutput(result.stdout);
}

function runCommand(
  binary: string,
  arguments_: readonly string[],
  environment: Readonly<Record<string, string | undefined>> = process.env,
): CommandResult {
  const result = Bun.spawnSync([binary, ...arguments_], {
    env: environment,
    stderr: "pipe",
    stdout: "pipe",
  });
  return {
    exitCode: result.exitCode,
    stderr: result.stderr.toString(),
    stdout:
      result.exitCode === 0 ? decodeStandardOutput([...result.stdout]) : "",
  };
}

function decodeStandardOutput(output: readonly number[]): string {
  try {
    return new TextDecoder("utf-8", { fatal: true }).decode(
      Uint8Array.from(output),
    );
  } catch {
    throw new MeasurementError("command stdout contains invalid UTF-8");
  }
}

function commandError(result: ReturnType<typeof runCommand>): MeasurementError {
  return new MeasurementError(
    result.stderr.trimEnd() || `command failed with exit ${result.exitCode}`,
    result.exitCode,
  );
}

function requireExecutable(binary: string, name: string): string {
  const found = binary.includes("/") ? binary : Bun.which(binary);
  try {
    accessSync(found ?? binary, constants.X_OK);
  } catch {
    throw new MeasurementError(`${name} is required`);
  }
  return found ?? binary;
}

function chunk<Value>(values: readonly Value[], size: number): Value[][] {
  return Array.from(
    { length: Math.ceil(values.length / size) },
    (_unusedValue, index) => values.slice(index * size, (index + 1) * size),
  );
}

function message(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}

export { main };
