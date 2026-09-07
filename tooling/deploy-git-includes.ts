import { z } from "zod";

const argumentOffset = 2;
const missingConfigurationExitCode = 1;

function migrateGitIncludes(): void {
  const result = Bun.spawnSync(
    ["git", "config", "--global", "--get-all", "include.path"],
    { stderr: "pipe", stdout: "pipe" },
  );
  if (
    result.exitCode !== 0 &&
    result.exitCode !== missingConfigurationExitCode
  ) {
    throw new Error(`Could not read Git includes: ${result.stderr.toString()}`);
  }
  const includes = z
    .string()
    .transform((output) => output.split("\n"))
    .parse(result.stdout.toString());
  if (!includes.includes("~/.config/git/config.delta")) {
    updateGitIncludes(["--add", "include.path", "~/.config/git/config.delta"]);
  }
  if (includes.includes("~/.gitconfig.delta")) {
    updateGitIncludes([
      "--unset-all",
      "include.path",
      "^~/[.]gitconfig[.]delta$",
    ]);
  }
}

function updateGitIncludes(arguments_: readonly string[]): void {
  const result = Bun.spawnSync(["git", "config", "--global", ...arguments_], {
    stderr: "inherit",
    stdout: "inherit",
  });
  if (result.exitCode !== 0) {
    throw new Error(`Could not update Git includes (${result.exitCode})`);
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
