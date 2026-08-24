import {
  lstat,
  mkdir,
  mkdtemp,
  readFile,
  realpath,
  rm,
  writeFile,
} from "node:fs/promises";
import { tmpdir } from "node:os";
import { dirname, isAbsolute, join, relative, resolve, sep } from "node:path";
import { format, type FormatConfig } from "oxfmt";
import { z } from "zod";
import formatConfig from "../oxfmt.config.ts";

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
  options?: FormatConfig,
) => ReturnType<typeof format>;

type FormattingChange = {
  path: string;
  source: string;
  formatted: string;
};

function isOutside(root: string, path: string) {
  const pathFromRoot = relative(root, path);
  return (
    pathFromRoot === ".." ||
    pathFromRoot.startsWith(`..${sep}`) ||
    isAbsolute(pathFromRoot)
  );
}

async function findTrackedTypeScriptPaths() {
  const git = Bun.spawn(
    ["git", "ls-files", "-z", "--", "*.ts", "*.tsx", "*.mts", "*.cts"],
    {
      stdout: "pipe",
      stderr: "inherit",
    },
  );
  const [status, output] = await Promise.all([
    git.exited,
    new Response(git.stdout).arrayBuffer(),
  ]);
  if (status !== 0) {
    return { status, paths: undefined };
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
  return { status, paths };
}

async function readOwnedFile(root: string, path: string) {
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

async function createPatch(changes: FormattingChange[]) {
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
      { cwd: temporaryRoot, stdout: "pipe", stderr: "inherit" },
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
    return new Uint8Array(patch);
  } finally {
    await rm(temporaryRoot, { recursive: true, force: true });
  }
}

async function applyPatch(root: string, patch: Uint8Array) {
  const apply = Bun.spawn(["git", "apply", "-p2", "--whitespace=nowarn"], {
    cwd: root,
    stdin: new Blob([patch]),
    stdout: "inherit",
    stderr: "inherit",
  });
  const status = await apply.exited;
  if (status !== 0) {
    throw new Error(
      `Git could not publish the TypeScript formatting patch (status ${status}).`,
    );
  }
}

export async function formatTypeScriptPaths(
  paths: string[],
  check: boolean,
  formatSource: FormatSource = format,
  repositoryRoot: string = process.cwd(),
) {
  const root = await realpath(repositoryRoot);
  const changes: FormattingChange[] = [];
  for (const path of paths) {
    const source = await readOwnedFile(root, path);
    const result = await formatSource(path, source, formatConfig);
    if (result.errors.length > 0) {
      throw new Error(
        `${path}: ${result.errors.map((error) => error.message).join("; ")}`,
      );
    }
    if (result.code !== source) {
      changes.push({ path, source, formatted: result.code });
    }
  }
  if (!check && changes.length > 0) {
    await applyPatch(root, await createPatch(changes));
  }
  return changes.map((change) => change.path);
}

async function main() {
  const invocation = invocationSchema.safeParse(Bun.argv.slice(2));
  if (!invocation.success) {
    console.error("Usage: format-typescript [--check]");
    return 2;
  }
  const discovery = await findTrackedTypeScriptPaths();
  if (!discovery.paths) {
    return discovery.status;
  }
  const check = invocation.data.length === 1;
  const different = await formatTypeScriptPaths(discovery.paths, check);
  if (check && different.length > 0) {
    console.error(`TypeScript formatting differs: ${different.join(", ")}`);
    return 1;
  }
  console.log(
    `${check ? "Checked" : "Formatted"} ${discovery.paths.length} TypeScript files.`,
  );
  return 0;
}

if (import.meta.main) {
  try {
    process.exitCode = await main();
  } catch (error) {
    console.error(error instanceof Error ? error.message : String(error));
    process.exitCode = 1;
  }
}
