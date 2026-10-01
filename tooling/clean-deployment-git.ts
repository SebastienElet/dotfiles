import { join } from "node:path";
import { lstatSync } from "node:fs";
import { verifyParents } from "./clean-deployment-paths.ts";
import { z } from "zod";

type GitUpdate = Readonly<{ path: string; apply: () => void }>;
const missingKeyExitCode = 1;
const ownedInclude = "~/.config/git/config.delta";
const outputSchema = z.string().transform((value) => value.split("\n"));

function git(
  path: string,
  args: readonly string[],
): Bun.SyncSubprocess<"pipe", "pipe"> {
  const result = Bun.spawnSync(["git", "config", "--file", path, ...args], {
    stdout: "pipe",
    stderr: "pipe",
  });
  return result;
}

function gitIncludeUpdates(
  repository: string,
  home: string,
): readonly GitUpdate[] {
  return [".gitconfig", ".config/git/config"].flatMap((relativePath) => {
    const path = join(home, relativePath);
    verifyParents(home, repository, path);
    const metadata = lstatSync(path, { throwIfNoEntry: false });
    if (metadata === undefined) {
      return [];
    }
    if (!metadata.isFile()) {
      throw new Error(
        `Shared Git configuration must be a regular file: ${path}`,
      );
    }
    const read = git(path, ["--get-all", "include.path"]);
    if (read.exitCode === missingKeyExitCode) {
      return [];
    }
    if (read.exitCode !== 0) {
      throw new Error(
        `Cannot inspect Git includes: ${path} (${read.exitCode})`,
      );
    }
    if (!outputSchema.parse(read.stdout.toString()).includes(ownedInclude)) {
      return [];
    }
    return [
      {
        path,
        apply: (): void => {
          removeGitInclude(path, { home, repository });
        },
      },
    ];
  });
}

function removeGitInclude(
  path: string,
  roots: Readonly<{ home: string; repository: string }>,
): void {
  verifyParents(roots.home, roots.repository, path);
  if (!lstatSync(path).isFile()) {
    throw new Error(`Shared Git configuration changed during cleanup: ${path}`);
  }
  const removed = git(path, [
    "--fixed-value",
    "--unset-all",
    "include.path",
    ownedInclude,
  ]);
  if (removed.exitCode !== 0) {
    throw new Error(
      `Cannot remove owned Git includes: ${path} (${removed.exitCode})`,
    );
  }
}

export { gitIncludeUpdates };
