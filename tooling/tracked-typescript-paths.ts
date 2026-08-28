import { dirname, join, relative, resolve, sep } from "node:path";
import { lstat } from "node:fs/promises";
import { z } from "zod";

const successExitCode = 0;
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
const typescriptPathsSchema = z.array(typescriptPathSchema);

type TrackedTypeScriptPaths = Readonly<{
  paths: readonly string[];
  status: number;
  trackedCount: number;
}>;

async function discoverTrackedTypeScriptPaths(
  repositoryRoot: string,
): Promise<TrackedTypeScriptPaths> {
  const [tracked, deleted] = await Promise.all([
    queryGitPaths(repositoryRoot, ["--cached"]),
    queryGitPaths(repositoryRoot, ["--deleted"]),
  ]);
  const failure = [tracked, deleted].find(
    ({ status }) => status !== successExitCode,
  );
  if (failure !== undefined) {
    return { paths: [], status: failure.status, trackedCount: 0 };
  }
  const deletedPaths = new Set<string>();
  for (const path of deleted.paths) {
    if (await isPlainDeletion(repositoryRoot, path)) {
      deletedPaths.add(path);
    }
  }
  return {
    paths: tracked.paths.filter((path) => !deletedPaths.has(path)),
    status: successExitCode,
    trackedCount: tracked.paths.length,
  };
}

async function queryGitPaths(
  repositoryRoot: string,
  options: readonly string[],
): Promise<Readonly<{ paths: readonly string[]; status: number }>> {
  const git = Bun.spawn(
    [
      "git",
      "ls-files",
      "-z",
      ...options,
      "--",
      "*.ts",
      "*.tsx",
      "*.mts",
      "*.cts",
    ],
    { cwd: repositoryRoot, stderr: "inherit", stdout: "pipe" },
  );
  const [status, output] = await Promise.all([
    git.exited,
    new Response(git.stdout).arrayBuffer(),
  ]);
  if (status !== successExitCode) {
    return { paths: [], status };
  }
  const serialized = new TextDecoder("utf-8", { fatal: true }).decode(output);
  if (serialized.length === 0) {
    return { paths: [], status };
  }
  if (!serialized.endsWith("\0")) {
    throw new Error("Git returned a malformed tracked TypeScript path list.");
  }
  return {
    paths: typescriptPathsSchema.parse(serialized.slice(0, -1).split("\0")),
    status,
  };
}

async function isPlainDeletion(root: string, path: string): Promise<boolean> {
  const parentFromRoot = relative(root, dirname(resolve(root, path)));
  let current = root;
  for (const component of parentFromRoot.split(sep).filter(Boolean)) {
    current = join(current, component);
    try {
      const metadata = await lstat(current);
      if (!metadata.isDirectory() || metadata.isSymbolicLink()) {
        return false;
      }
    } catch (error) {
      if (isMissingFile(error)) {
        return true;
      }
      throw error;
    }
  }
  try {
    await lstat(resolve(root, path));
    return false;
  } catch (error) {
    if (isMissingFile(error)) {
      return true;
    }
    throw error;
  }
}

function isMissingFile(error: unknown): boolean {
  return error instanceof Error && "code" in error && error.code === "ENOENT";
}

export { discoverTrackedTypeScriptPaths };
