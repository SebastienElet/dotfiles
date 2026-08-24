import { constants } from "node:fs";
import { lstat, open, realpath, type FileHandle } from "node:fs/promises";
import { isAbsolute, relative, resolve, sep } from "node:path";
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

async function openOwnedFile(root: string, path: string) {
  const absolutePath = resolve(root, path);
  if (
    isOutside(root, absolutePath) ||
    (await realpath(absolutePath)) !== absolutePath
  ) {
    throw new Error(
      `${path}: tracked TypeScript path is not confined to the repository.`,
    );
  }
  const beforeOpen = await lstat(absolutePath);
  if (!beforeOpen.isFile() || beforeOpen.nlink !== 1) {
    throw new Error(
      `${path}: tracked TypeScript path is not an owned regular file.`,
    );
  }
  const file = await open(
    absolutePath,
    constants.O_RDWR | constants.O_NOFOLLOW,
  );
  const opened = await file.stat();
  if (
    !opened.isFile() ||
    opened.nlink !== 1 ||
    opened.dev !== beforeOpen.dev ||
    opened.ino !== beforeOpen.ino
  ) {
    await file.close();
    throw new Error(
      `${path}: tracked TypeScript file changed while opening it.`,
    );
  }
  return file;
}

async function readUtf8(file: FileHandle, path: string) {
  try {
    return new TextDecoder("utf-8", { fatal: true }).decode(
      await file.readFile(),
    );
  } catch {
    throw new Error(`${path}: tracked TypeScript file is not valid UTF-8.`);
  }
}

async function writeFile(file: FileHandle, code: string) {
  const output = Buffer.from(code);
  await file.write(output, 0, output.length, 0);
  await file.truncate(output.length);
}

export async function formatTypeScriptPaths(
  paths: string[],
  check: boolean,
  formatSource: FormatSource = format,
  repositoryRoot: string = process.cwd(),
) {
  const root = await realpath(repositoryRoot);
  const different: string[] = [];
  for (const path of paths) {
    const file = await openOwnedFile(root, path);
    try {
      const source = await readUtf8(file, path);
      const result = await formatSource(path, source, formatConfig);
      if (result.errors.length > 0) {
        throw new Error(
          `${path}: ${result.errors.map((error) => error.message).join("; ")}`,
        );
      }
      if (result.code !== source) {
        different.push(path);
        if (!check) {
          await writeFile(file, result.code);
        }
      }
    } finally {
      await file.close();
    }
  }
  return different;
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
