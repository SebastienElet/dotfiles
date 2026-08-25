import { dirname, isAbsolute, join, relative, resolve, sep } from "node:path";
import {
  lstat,
  mkdir,
  mkdtemp,
  readFile,
  realpath,
  rm,
  writeFile,
} from "node:fs/promises";
import { format } from "oxfmt";
import formatConfig from "../oxfmt.config.ts";
import { tmpdir } from "node:os";
import { z } from "zod";

const invocationSchema = z.union([
  z.tuple([]),
  z.tuple([z.literal("--check")]),
]);
const typescriptPathSchema = z
  .string()
  .min(1)
  .refine(
    (path) =>
      [".ts", ".tsx", ".mts", ".cts"].some((extension) =>
        path.endsWith(extension),
      ),
    "unsupported TypeScript path",
  );
const typescriptPathsSchema = z.array(typescriptPathSchema).min(1);

type FormatSource = (
  fileName: string,
  sourceText: string,
) => ReturnType<typeof format>;

type FormattingChange = Readonly<{
  path: string;
  source: string;
  formatted: string;
}>;

type FormatTypeScriptOptions = Readonly<{
  check: boolean;
  formatSource?: FormatSource;
  repositoryRoot?: string;
}>;

const invalidInvocationExitCode = 2;
const argumentOffset = 2;

function isOutside(root: string, path: string): boolean {
  const pathFromRoot = relative(root, path);
  return (
    pathFromRoot === ".." ||
    pathFromRoot.startsWith(`..${sep}`) ||
    isAbsolute(pathFromRoot)
  );
}

async function findTrackedTypeScriptPaths(): Promise<
  Readonly<{ paths: readonly string[] | undefined; status: number }>
> {
  const git = Bun.spawn(
    ["git", "ls-files", "-z", "--", "*.ts", "*.tsx", "*.mts", "*.cts"],
    {
      stderr: "inherit",
      stdout: "pipe",
    },
  );
  const [status, output] = await Promise.all([
    git.exited,
    new Response(git.stdout).arrayBuffer(),
  ]);
  if (status !== 0) {
    return { paths: undefined, status };
  }
  const serializedPaths = new TextDecoder("utf-8", { fatal: true }).decode(
    output,
  );
  if (serializedPaths.length === 0) {
    throw new Error("No tracked TypeScript files found.");
  }
  if (!serializedPaths.endsWith("\0")) {
    throw new Error("Git returned a malformed tracked TypeScript path list.");
  }
  const paths = typescriptPathsSchema.parse(
    serializedPaths.slice(0, -1).split("\0"),
  );
  return { paths, status };
}

async function readOwnedFile(root: string, path: string): Promise<string> {
  const absolutePath = resolve(root, path);
  if (
    isOutside(root, absolutePath) ||
    (await realpath(absolutePath)) !== absolutePath
  ) {
    throw new Error(
      `${path}: tracked TypeScript path is not confined to the repository.`,
    );
  }
  const file = await lstat(absolutePath);
  if (!file.isFile() || file.nlink !== 1) {
    throw new Error(
      `${path}: tracked TypeScript path is not an owned regular file.`,
    );
  }
  try {
    return new TextDecoder("utf-8", { fatal: true }).decode(
      await readFile(absolutePath),
    );
  } catch {
    throw new Error(`${path}: tracked TypeScript file is not valid UTF-8.`);
  }
}

async function createPatch(
  changes: readonly FormattingChange[],
): Promise<readonly number[]> {
  const temporaryRoot = await mkdtemp(join(tmpdir(), "dotfiles-oxfmt-"));
  try {
    for (const change of changes) {
      const beforePath = join(temporaryRoot, "before", change.path);
      const afterPath = join(temporaryRoot, "after", change.path);
      await Promise.all([
        mkdir(dirname(beforePath), { recursive: true }),
        mkdir(dirname(afterPath), { recursive: true }),
      ]);
      await Promise.all([
        writeFile(beforePath, change.source),
        writeFile(afterPath, change.formatted),
      ]);
    }
    const diff = Bun.spawn(
      [
        "git",
        "diff",
        "--no-index",
        "--binary",
        "--no-ext-diff",
        "--no-textconv",
        "--no-renames",
        "--",
        "before",
        "after",
      ],
      { cwd: temporaryRoot, stderr: "inherit", stdout: "pipe" },
    );
    const [status, patch] = await Promise.all([
      diff.exited,
      new Response(diff.stdout).arrayBuffer(),
    ]);
    if (status !== 1) {
      throw new Error(
        `Git could not build the TypeScript formatting patch (status ${status}).`,
      );
    }
    return [...new Uint8Array(patch)];
  } finally {
    await rm(temporaryRoot, { force: true, recursive: true });
  }
}

async function applyPatch(
  root: string,
  patch: readonly number[],
): Promise<void> {
  const apply = Bun.spawn(["git", "apply", "-p2", "--whitespace=nowarn"], {
    cwd: root,
    stderr: "inherit",
    stdin: new Blob([Uint8Array.from(patch)]),
    stdout: "inherit",
  });
  const status = await apply.exited;
  if (status !== 0) {
    throw new Error(
      `Git could not publish the TypeScript formatting patch (status ${status}).`,
    );
  }
}

async function formatTypeScriptPaths(
  paths: readonly string[],
  options: FormatTypeScriptOptions,
): Promise<string[]> {
  const {
    check,
    formatSource = (
      fileName: string,
      sourceText: string,
    ): ReturnType<typeof format> => format(fileName, sourceText, formatConfig),
    repositoryRoot = process.cwd(),
  } = options;
  const root = await realpath(repositoryRoot);
  const changes: FormattingChange[] = [];
  for (const path of paths) {
    const source = await readOwnedFile(root, path);
    const result = await formatSource(path, source);
    if (result.errors.length > 0) {
      const messages: string[] = [];
      for (const error of result.errors) {
        messages.push(error.message);
      }
      throw new Error(`${path}: ${messages.join("; ")}`);
    }
    if (result.code !== source) {
      changes.push({ formatted: result.code, path, source });
    }
  }
  if (!check && changes.length > 0) {
    await applyPatch(root, await createPatch(changes));
  }
  return changes.map((change) => change.path);
}

async function main(): Promise<number> {
  const invocation = invocationSchema.safeParse(Bun.argv.slice(argumentOffset));
  if (!invocation.success) {
    process.stderr.write("Usage: format-typescript [--check]\n");
    return invalidInvocationExitCode;
  }
  const discovery = await findTrackedTypeScriptPaths();
  if (!discovery.paths) {
    return discovery.status;
  }
  const check = invocation.data.length === 1;
  const different = await formatTypeScriptPaths(discovery.paths, { check });
  if (check && different.length > 0) {
    process.stderr.write(
      `TypeScript formatting differs: ${different.join(", ")}\n`,
    );
    return 1;
  }
  process.stdout.write(
    `${check ? "Checked" : "Formatted"} ${discovery.paths.length} TypeScript files.\n`,
  );
  return 0;
}

if (import.meta.main) {
  try {
    process.exitCode = await main();
  } catch (error) {
    process.stderr.write(
      `${error instanceof Error ? error.message : String(error)}\n`,
    );
    process.exitCode = 1;
  }
}

export { formatTypeScriptPaths };
