import {
  commitSha,
  gitOutput,
  gitPaths,
  rejectPrivatePaths,
} from "./secret-scan-git.ts";
import { introducedRange, rejectIntroducedPaths } from "./secret-scan-ci.ts";
import { mkdtemp, rm, writeFile } from "node:fs/promises";
import { join } from "node:path";
import { runScanner } from "./secret-scan-scanner.ts";
import { scanBlobs } from "./secret-scan-snapshot.ts";
import { tmpdir } from "node:os";
import { z } from "zod";

const invocation = z.tuple([z.enum(["staged", "current", "ci", "history"])]);
const argumentOffset = 2;

async function scanCurrent(
  root: string,
  directory: string,
  head: string,
): Promise<void> {
  const paths = gitPaths(root, ["ls-tree", "-r", "--name-only", "-z", head]);
  rejectPrivatePaths(paths);
  await scanBlobs(root, directory, { paths, ref: head });
}

async function scanStaged(root: string, directory: string): Promise<void> {
  rejectPrivatePaths(gitPaths(root, ["ls-files", "--cached", "-z"]));
  await runScanner(root, directory, ["git", "--staged", root]);
  const paths = gitPaths(root, [
    "diff",
    "--staged",
    "--no-ext-diff",
    "--no-textconv",
    "--no-renames",
    "--name-only",
    "-z",
    "--diff-filter=ACMRT",
  ]);
  await scanBlobs(root, directory, { paths, ref: "" });
}

async function scan(
  mode: "staged" | "current" | "ci" | "history",
): Promise<void> {
  const root = gitOutput(process.cwd(), ["rev-parse", "--show-toplevel"])
    .toString()
    .trim();
  const directory = await mkdtemp(join(tmpdir(), "dotfiles-secret-scan-"));
  try {
    const configRef =
      mode === "staged" ? ":.gitleaks.toml" : "HEAD:.gitleaks.toml";
    await writeFile(
      join(directory, "config.toml"),
      gitOutput(root, ["show", configRef]),
    );
    await writeFile(join(directory, "ignore"), "");
    if (mode === "staged") {
      await scanStaged(root, directory);
      return;
    }
    const head = commitSha(root, "HEAD");
    if (mode === "current" || mode === "ci") {
      await scanCurrent(root, directory, head);
    }
    if (mode === "history") {
      await runScanner(root, directory, [
        "git",
        "--log-opts=--all --full-history -m --text --no-textconv --no-ext-diff --no-renames",
        root,
      ]);
    }
    if (mode === "ci") {
      const range = await introducedRange(root);
      rejectIntroducedPaths(root, range);
      await runScanner(root, directory, [
        "git",
        `--log-opts=${range} --full-history -m --text --no-textconv --no-ext-diff --no-renames`,
        root,
      ]);
    }
  } finally {
    await rm(directory, { recursive: true, force: true });
  }
}

if (import.meta.main) {
  try {
    const [mode] = invocation.parse(Bun.argv.slice(argumentOffset));
    await scan(mode);
  } catch (error) {
    process.stderr.write(
      `${error instanceof Error && !(error instanceof z.ZodError) ? error.message : "Secret scan: invalid invocation, event or report."}\n`,
    );
    process.exitCode = 1;
  }
}
