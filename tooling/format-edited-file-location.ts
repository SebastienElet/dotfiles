import { dirname, isAbsolute, join, relative, sep } from "node:path";
import { lstat, realpath } from "node:fs/promises";

type EditedFile = Readonly<{
  absolutePath: string;
  relativePath: string;
  root: string;
}>;

type IgnoreStatus = "ignored" | "tracked-or-untracked" | "unknown";

const gitCheckIgnoreMatched = 0;
const gitCheckIgnoreUnmatched = 1;

async function existingAncestor(path: string): Promise<string> {
  try {
    await lstat(path);
    return path;
  } catch {
    const parent = dirname(path);
    return parent === path ? path : existingAncestor(parent);
  }
}

function isInside(root: string, path: string): boolean {
  const fromRoot = relative(root, path);
  return (
    fromRoot !== "" &&
    fromRoot !== ".." &&
    !fromRoot.startsWith(`..${sep}`) &&
    !isAbsolute(fromRoot)
  );
}

async function gitLines(
  directory: string,
  arguments_: readonly string[],
): Promise<readonly string[] | undefined> {
  const git = Bun.spawn(["git", "-C", directory, ...arguments_], {
    stderr: "ignore",
    stdout: "pipe",
  });
  const [status, stdout] = await Promise.all([
    git.exited,
    new Response(git.stdout).text(),
  ]);
  return status === 0 ? stdout.trimEnd().split("\n") : undefined;
}

async function repositoryCommonDirectoryOf(
  directory: string,
): Promise<string | undefined> {
  const lines = await gitLines(directory, [
    "rev-parse",
    "--path-format=absolute",
    "--git-common-dir",
  ]);
  return lines?.[0];
}

async function locateInRepository(
  path: string,
  repositoryCommonDirectory: string,
): Promise<EditedFile | undefined> {
  const directory = await existingAncestor(dirname(path));
  const [root, commonDirectory] =
    (await gitLines(directory, [
      "rev-parse",
      "--path-format=absolute",
      "--show-toplevel",
      "--git-common-dir",
    ])) ?? [];
  if (
    root === undefined ||
    commonDirectory === undefined ||
    (await realpath(commonDirectory)) !==
      (await realpath(repositoryCommonDirectory))
  ) {
    return undefined;
  }
  const realRoot = await realpath(root);
  const absolutePath = join(
    await realpath(directory),
    relative(directory, path),
  );
  if (!isInside(realRoot, absolutePath)) {
    return undefined;
  }
  return {
    absolutePath,
    relativePath: relative(realRoot, absolutePath).split(sep).join("/"),
    root: realRoot,
  };
}

async function ignoreStatus(file: EditedFile): Promise<IgnoreStatus> {
  const check = Bun.spawn(
    [
      "git",
      "-C",
      file.root,
      "check-ignore",
      "--quiet",
      "--",
      file.relativePath,
    ],
    { stderr: "ignore", stdout: "ignore" },
  );
  const status = await check.exited;
  if (status === gitCheckIgnoreMatched) {
    return "ignored";
  }
  return status === gitCheckIgnoreUnmatched
    ? "tracked-or-untracked"
    : "unknown";
}

async function isConfinedRegularFile(file: EditedFile): Promise<boolean> {
  const stats = await lstat(file.absolutePath);
  return (
    stats.isFile() && isInside(file.root, await realpath(file.absolutePath))
  );
}

async function exists(path: string): Promise<boolean> {
  try {
    await lstat(path);
    return true;
  } catch {
    return false;
  }
}

export {
  exists,
  ignoreStatus,
  isConfinedRegularFile,
  locateInRepository,
  repositoryCommonDirectoryOf,
};
export type { EditedFile };
