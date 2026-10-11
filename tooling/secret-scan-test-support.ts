import {
  copyFile,
  mkdir,
  mkdtemp,
  rm,
  symlink,
  writeFile,
} from "node:fs/promises";
import { join, resolve } from "node:path";
import { afterEach } from "bun:test";
import { tmpdir } from "node:os";

const roots: string[] = [];
const sourceRoot = resolve(import.meta.dir, "..");
const command = join(sourceRoot, "tooling/secret-scan.ts");
const tokenWordRepetitions = 4;
const token = `ghp_${"fictitious".repeat(tokenWordRepetitions)}`;
const gitEnvironment = {
  ...process.env,
  GIT_CONFIG_GLOBAL: "/dev/null",
  GIT_CONFIG_NOSYSTEM: "1",
};

function git(root: string, ...args: readonly string[]): string {
  const result = Bun.spawnSync(["git", "-C", root, ...args], {
    env: gitEnvironment,
  });
  if (result.exitCode !== 0) {
    throw new Error("Fixture Git command failed.");
  }
  return result.stdout.toString().trim();
}

async function repository(): Promise<string> {
  const root = await mkdtemp(join(tmpdir(), "dotfiles-secret-test-"));
  roots.push(root);
  git(root, "init", "-q");
  git(root, "config", "user.email", "fixture@example.invalid");
  git(root, "config", "user.name", "Fictitious fixture");
  await mkdir(join(root, "tooling"));
  await Promise.all(
    [
      ".gitleaks.toml",
      "tooling/secret-scan.ts",
      "tooling/secret-scan-git.ts",
      "tooling/secret-scan-ci.ts",
      "tooling/secret-scan-scanner.ts",
      "tooling/secret-scan-snapshot.ts",
    ].map((path) => copyFile(join(sourceRoot, path), join(root, path))),
  );
  await symlink(
    join(sourceRoot, "node_modules"),
    join(root, "node_modules"),
    "dir",
  );
  git(root, "add", ".gitleaks.toml", "tooling");
  git(root, "commit", "-qm", "clean base");
  return root;
}

function scan(
  root: string,
  mode = "staged",
  env: Readonly<NodeJS.ProcessEnv> = {},
): Bun.SyncSubprocess<"pipe", "pipe"> {
  return Bun.spawnSync([process.execPath, "--no-env-file", command, mode], {
    cwd: root,
    env: { ...gitEnvironment, ...env },
  });
}

function commit(root: string): Bun.SyncSubprocess<"pipe", "pipe"> {
  return Bun.spawnSync(
    ["git", "-C", root, "commit", "-qm", "fictitious fixture"],
    { env: gitEnvironment },
  );
}

async function event(root: string, data: unknown): Promise<NodeJS.ProcessEnv> {
  const path = join(root, "event.json");
  await writeFile(path, JSON.stringify(data));
  return { GITHUB_EVENT_NAME: "pull_request", GITHUB_EVENT_PATH: path };
}

afterEach(async () => {
  await Promise.all(
    roots.splice(0).map((root) => rm(root, { recursive: true, force: true })),
  );
});

export {
  commit,
  event,
  git,
  gitEnvironment,
  repository,
  scan,
  sourceRoot,
  token,
};
