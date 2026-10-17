import { lstatSync, realpathSync } from "node:fs";
import { join } from "node:path";
import { z } from "zod";

const argumentOffset = 2;
const missingConfigurationExitCode = 1;
const oldInclude = "~/.gitconfig.delta";
const newInclude = "~/.config/git/config.delta";
const environmentSchema = z.object({
  HOME: z.string().min(1),
  XDG_CONFIG_HOME: z.string().optional(),
  GIT_CONFIG_GLOBAL: z.string().optional(),
});
const includesSchema = z
  .string()
  .endsWith("\0")
  .transform((output) => output.slice(0, -1).split("\0"));

function migrateGitIncludes(): void {
  const environment = environmentSchema.parse(process.env);
  const globalConfig = environment.GIT_CONFIG_GLOBAL;
  const xdgConfigRoot = environment.XDG_CONFIG_HOME;
  const configPaths =
    globalConfig !== undefined && globalConfig !== ""
      ? [globalConfig]
      : [
          join(
            xdgConfigRoot === undefined || xdgConfigRoot === ""
              ? join(environment.HOME, ".config")
              : xdgConfigRoot,
            "git",
            "config",
          ),
          join(environment.HOME, ".gitconfig"),
        ];
  const seenTargets = new Set<string>();
  const configurations = configPaths
    .filter((path) => configExists(path))
    .filter((path) => {
      const target = realpathSync(path);
      if (seenTargets.has(target)) {
        return false;
      }
      seenTargets.add(target);
      return true;
    })
    .map((path): Readonly<{ path: string; includes: readonly string[] }> => ({
      path,
      includes: readGitIncludes(path),
    }));
  if (!configurations.some(({ includes }) => includes.includes(newInclude))) {
    updateGitIncludes(["--global", "--add", "include.path", newInclude]);
  }
  for (const { path, includes } of configurations) {
    if (includes.includes(oldInclude)) {
      updateGitIncludes([
        "--file",
        path,
        "--unset-all",
        "include.path",
        "^~/[.]gitconfig[.]delta$",
      ]);
    }
  }
}

function configExists(path: string): boolean {
  try {
    lstatSync(path);
    return true;
  } catch (error) {
    if (error instanceof Error && "code" in error && error.code === "ENOENT") {
      return false;
    }
    throw error;
  }
}

function readGitIncludes(path: string): string[] {
  const result = Bun.spawnSync(
    ["git", "config", "--null", "--file", path, "--get-all", "include.path"],
    { stderr: "pipe", stdout: "pipe" },
  );
  if (result.exitCode === missingConfigurationExitCode) {
    return [];
  }
  if (result.exitCode !== 0) {
    throw new Error(
      `Could not read Git includes from ${path}: ${result.stderr.toString()}`,
    );
  }
  return includesSchema.parse(result.stdout.toString());
}

function updateGitIncludes(arguments_: readonly string[]): void {
  const result = Bun.spawnSync(["git", "config", ...arguments_], {
    stderr: "inherit",
    stdout: "inherit",
  });
  if (result.exitCode !== 0) {
    throw new Error(
      `Could not update Git includes with git config ${arguments_.join(" ")} (${result.exitCode})`,
    );
  }
}

if (import.meta.main) {
  try {
    z.tuple([]).parse(process.argv.slice(argumentOffset));
    migrateGitIncludes();
  } catch (error) {
    process.stderr.write(
      `Error: ${error instanceof Error ? error.message : String(error)}\n`,
    );
    process.exitCode = 1;
  }
}

export { migrateGitIncludes };
