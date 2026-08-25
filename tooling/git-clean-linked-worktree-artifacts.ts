import { realpath } from "node:fs/promises";
import { z } from "zod";

const argumentsSchema = z.tuple([z.string().min(1)]);
const recordSeparator = "\0\0";
const minimumRecordFieldCount = 2;
const worktreeFieldSchema = z
  .string()
  .startsWith("worktree ")
  .transform((field) => field.slice("worktree ".length))
  .pipe(z.string().min(1));

type CommandResult = Readonly<{
  status: number;
  stderr: string;
  stdout: Uint8Array;
}>;

function run(arguments_: readonly string[]): CommandResult {
  try {
    const result = Bun.spawnSync(["git", ...arguments_], {
      stderr: "pipe",
      stdout: "pipe",
    });
    return {
      status: result.exitCode,
      stderr: result.stderr.toString(),
      stdout: result.stdout,
    };
  } catch (error) {
    return {
      status: 127,
      stderr: `git is unavailable: ${String(error)}\n`,
      stdout: new Uint8Array(),
    };
  }
}

function fail(message: string, detail = ""): number {
  if (detail !== "") {
    process.stderr.write(detail);
  }
  process.stderr.write(`git-clean-linked-worktree-artifacts: ${message}\n`);
  return 1;
}

function parseWorktrees(
  output: string,
): readonly Readonly<{ locked: boolean; path: string }>[] {
  const worktreeOutput = z.string().endsWith(recordSeparator).parse(output);
  return worktreeOutput
    .slice(0, -recordSeparator.length)
    .split(recordSeparator)
    .map((record) => {
      const fields = z
        .array(z.string())
        .min(minimumRecordFieldCount)
        .parse(record.split("\0"));
      return {
        locked: fields.some(
          (field) => field === "locked" || field.startsWith("locked "),
        ),
        path: worktreeFieldSchema.parse(fields[0]),
      };
    });
}

async function linkedWorktreeState(
  worktree: string,
): Promise<"linked" | "locked" | "refused"> {
  const worktreeList = run([
    "-C",
    worktree,
    "worktree",
    "list",
    "--porcelain",
    "-z",
  ]);
  if (worktreeList.status !== 0) {
    throw new Error(worktreeList.stderr.trim() || "Git worktree lookup failed");
  }
  const worktrees = parseWorktrees(
    new TextDecoder("utf-8", { fatal: true }).decode(worktreeList.stdout),
  );
  const [resolvedWorktreePaths, resolvedWorktree] = await Promise.all([
    Promise.all(worktrees.map(({ path }) => realpath(path))),
    realpath(worktree),
  ]);
  const index = resolvedWorktreePaths.indexOf(resolvedWorktree);
  if (index < 1) {
    return "refused";
  }
  return worktrees[index]?.locked === true ? "locked" : "linked";
}

async function main(arguments_: readonly string[]): Promise<number> {
  const parsedArguments = argumentsSchema.safeParse(arguments_);
  if (!parsedArguments.success) {
    return fail("expected exactly one worktree path");
  }
  const [worktree] = parsedArguments.data;
  try {
    const state = await linkedWorktreeState(worktree);
    if (state === "refused") {
      return fail("refusing to clean the primary or an unregistered worktree");
    }
    if (state === "locked") {
      return fail("refusing to clean a locked worktree");
    }
    const cleanup = run(["-C", worktree, "clean", "-dfXq"]);
    if (cleanup.status !== 0) {
      return fail("ignored file cleanup failed", cleanup.stderr);
    }
    return 0;
  } catch (error) {
    return fail(error instanceof Error ? error.message : String(error));
  }
}

export { main };
