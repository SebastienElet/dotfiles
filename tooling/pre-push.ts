import { join, resolve } from "node:path";
import { mkdtempSync, rmSync, symlinkSync } from "node:fs";
import { tmpdir } from "node:os";
import { z } from "zod";

const hookArgumentsSchema = z.tuple([z.string().min(1), z.string().min(1)]);
const lineFeed = "\n";
const objectIdSchema = z.string().regex(/^(?:[0-9a-f]{40}|[0-9a-f]{64})$/u);
const zeroObjectIdSchema = z.string().regex(/^(?:0{40}|0{64})$/u);
const refSchema = z.string().min(1).regex(/^\S+$/u);
const updateSchema = z
  .tuple([refSchema, objectIdSchema, refSchema, objectIdSchema])
  .readonly();

type CommandResult = Readonly<{
  status: number;
  stderr: string;
  stdout: string;
}>;
type CommandRunner = (
  command: string,
  arguments_: readonly string[],
  directory?: string,
) => CommandResult;

const run: CommandRunner = (command, arguments_, directory) => {
  try {
    const result = Bun.spawnSync([command, ...arguments_], {
      stderr: "pipe",
      stdout: "pipe",
      ...(directory === undefined ? {} : { cwd: directory }),
    });
    return {
      status: result.exitCode,
      stderr: result.stderr.toString(),
      stdout: result.stdout.toString(),
    };
  } catch (error) {
    return {
      status: 127,
      stderr: `${command} is unavailable: ${String(error)}\n`,
      stdout: "",
    };
  }
};

function fail(message: string, detail = ""): number {
  if (detail !== "") {
    process.stderr.write(detail);
  }
  process.stderr.write(`pre-push: ${message}\n`);
  return 1;
}

function parseUpdates(input: string): readonly z.output<typeof updateSchema>[] {
  if (input === "") {
    return [];
  }
  const parsedInput = z.string().min(1).endsWith(lineFeed).parse(input);
  return parsedInput
    .slice(0, -lineFeed.length)
    .split(lineFeed)
    .map((line) => updateSchema.parse(line.split(" ")));
}

function requireOutput(
  options: Readonly<{
    runner: CommandRunner;
    command: string;
    arguments: readonly string[];
    failure: string;
  }>,
): string {
  const { runner, command, arguments: arguments_, failure } = options;
  const result = runner(command, arguments_);
  if (result.status !== 0) {
    throw new Error(result.stderr.trim() || failure);
  }
  return result.stdout;
}

function runStaticChecks(runner: CommandRunner, directory: string): number {
  for (const command of [
    [
      "bun",
      ["--config=/dev/null", "--no-env-file", "tooling/lint-typescript.ts"],
    ],
    ["bun", ["--config=/dev/null", "--no-env-file", "run", "typecheck"]],
  ] as const) {
    const [commandName, commandArguments] = command;
    const result = runner(commandName, commandArguments, directory);
    if (result.status !== 0) {
      return fail("static validation failed", result.stderr);
    }
  }
  return 0;
}

function runStaticChecksAtCommit(head: string, runner: CommandRunner): number {
  const validationWorktree = mkdtempSync(join(tmpdir(), "dotfiles-pre-push-"));
  const addition = runner("git", [
    "worktree",
    "add",
    "--detach",
    "--quiet",
    validationWorktree,
    head,
  ]);
  if (addition.status !== 0) {
    rmSync(validationWorktree, { force: true, recursive: true });
    return fail("validation worktree creation failed", addition.stderr);
  }
  let status = 1;
  try {
    symlinkSync(
      resolve("node_modules"),
      join(validationWorktree, "node_modules"),
      "dir",
    );
    status = runStaticChecks(runner, validationWorktree);
  } catch (error) {
    status = fail(error instanceof Error ? error.message : String(error));
  }
  const removal = runner("git", [
    "worktree",
    "remove",
    "--force",
    "--force",
    validationWorktree,
  ]);
  if (removal.status !== 0) {
    return fail("validation worktree cleanup failed", removal.stderr);
  }
  rmSync(validationWorktree, { force: true, recursive: true });
  return status;
}

function main(
  arguments_: readonly string[],
  input: string,
  runner: CommandRunner = run,
): number {
  try {
    hookArgumentsSchema.parse(arguments_);
    const branchUpdates = parseUpdates(input).filter(
      ([, localObjectId, remoteRef]) =>
        remoteRef.startsWith("refs/heads/") &&
        !zeroObjectIdSchema.safeParse(localObjectId).success,
    );
    if (branchUpdates.length === 0) {
      return 0;
    }
    const head = objectIdSchema.parse(
      requireOutput({
        runner,
        command: "git",
        arguments: ["rev-parse", "--verify", "HEAD"],
        failure: "HEAD lookup failed",
      }).trim(),
    );
    if (branchUpdates.some(([, localObjectId]) => localObjectId !== head)) {
      return fail("checkout every branch being pushed before validation");
    }
    return runStaticChecksAtCommit(head, runner);
  } catch (error) {
    return fail(error instanceof Error ? error.message : String(error));
  }
}

export { main };
export type { CommandRunner };
