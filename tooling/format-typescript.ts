import { lstat, realpath } from "node:fs/promises";
import { isAbsolute, relative, resolve, sep } from "node:path";
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

async function requireConfinedRegularFiles(paths: string[]) {
  const root = await realpath(process.cwd());
  for (const path of paths) {
    const absolutePath = resolve(root, path);
    if (isOutside(root, absolutePath)) {
      throw new Error(
        `${path}: tracked TypeScript path escapes the repository.`,
      );
    }
    const file = await lstat(absolutePath);
    if (!file.isFile() || file.nlink !== 1) {
      throw new Error(
        `${path}: tracked TypeScript path is not an owned regular file.`,
      );
    }
    if (isOutside(root, await realpath(absolutePath))) {
      throw new Error(
        `${path}: tracked TypeScript path resolves outside the repository.`,
      );
    }
  }
}

async function formatTypeScript(paths: string[], check: boolean) {
  const options = check ? ["--check"] : [];
  for (let index = 0; index < paths.length; index += 500) {
    const formatter = Bun.spawn(
      ["oxfmt", ...options, "--", ...paths.slice(index, index + 500)],
      {
        stdout: "inherit",
        stderr: "inherit",
      },
    );
    const status = await formatter.exited;
    if (status !== 0) {
      return status;
    }
  }
  return 0;
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
  await requireConfinedRegularFiles(discovery.paths);
  return formatTypeScript(discovery.paths, invocation.data.length === 1);
}

try {
  process.exitCode = await main();
} catch (error) {
  console.error(error instanceof Error ? error.message : String(error));
  process.exitCode = 1;
}
