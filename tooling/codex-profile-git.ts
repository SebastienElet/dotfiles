import { isAbsolute } from "node:path";
import { parseArgs } from "node:util";
import { z } from "zod";

const gitTimeoutMilliseconds = 1000;
const directoryCount = 2;
const directoriesSchema = z
  .array(z.string().refine(isAbsolute))
  .length(directoryCount);

function codexGitArguments(arguments_: readonly string[]): readonly string[] {
  try {
    const { values } = parseArgs({
      args: [...arguments_],
      strict: false,
      allowPositionals: true,
      options: { cd: { type: "string", short: "C", multiple: true } },
    });
    const directory = z.string().parse(values.cd?.at(-1) ?? process.cwd());
    const result = Bun.spawnSync(
      [
        "git",
        "-C",
        directory,
        "rev-parse",
        "--path-format=absolute",
        "--git-common-dir",
        "--absolute-git-dir",
      ],
      {
        stdout: "pipe",
        stderr: "ignore",
        timeout: gitTimeoutMilliseconds,
      },
    );
    if (result.exitCode !== 0) {
      return [];
    }
    const directories = directoriesSchema.parse(
      result.stdout.toString("utf8").trimEnd().split("\n"),
    );
    return [...new Set(directories)].flatMap((path) => ["--add-dir", path]);
  } catch {
    return [];
  }
}

export { codexGitArguments };
