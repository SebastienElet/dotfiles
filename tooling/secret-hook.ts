import { chmod, lstat, mkdir, readFile, writeFile } from "node:fs/promises";
import { dirname } from "node:path";
import { gitOutput } from "./secret-scan-git.ts";

const hook = `#!/usr/bin/env bash
set -e
repository_root=$(git rev-parse --show-toplevel)
export PATH="$repository_root/.moon/cache/gitleaks:$PATH:/opt/homebrew/bin:/usr/local/bin"
exec bun --config=/dev/null --no-env-file --no-install "$repository_root/tooling/secret-scan.ts" staged
`;
const executableMode = 0o755;

async function installHook(root: string): Promise<void> {
  const configured = Bun.spawnSync([
    "git",
    "-C",
    root,
    "config",
    "--get",
    "core.hooksPath",
  ]);
  if (configured.exitCode !== 1) {
    throw new Error(
      "Secret hook: core.hooksPath is configured or unreadable; preserve it and integrate the staged command into your existing hook.",
    );
  }
  const path = gitOutput(root, [
    "rev-parse",
    "--path-format=absolute",
    "--git-path",
    "hooks/pre-commit",
  ])
    .toString()
    .trim();
  const existing = await lstat(path).catch((error: unknown) => {
    if (error instanceof Error && "code" in error && error.code === "ENOENT") {
      return;
    }
    throw error;
  });
  if (existing) {
    if (!existing.isFile() || (await readFile(path, "utf8")) !== hook) {
      throw new Error(
        "Secret hook: a third-party pre-commit exists; preserve it and integrate the staged command manually.",
      );
    }
    const ownerExecuteMask = 0o100;
    if ((existing.mode & ownerExecuteMask) === 0) {
      await chmod(path, executableMode);
    }
    return;
  }
  await mkdir(dirname(path), { recursive: true });
  await writeFile(path, hook, { flag: "wx", mode: executableMode });
}

if (import.meta.main) {
  try {
    await installHook(process.cwd());
  } catch (error) {
    process.stderr.write(
      `${error instanceof Error ? error.message : "Secret hook preparation failed."}\n`,
    );
    process.exitCode = 1;
  }
}

export { installHook };
