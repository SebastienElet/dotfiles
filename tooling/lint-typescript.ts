import { access, readFile } from "node:fs/promises";
import { resolve } from "node:path";
import { z } from "zod";

const EXPECTED_ARGUMENT_COUNT = 2;
const FAILURE = 1;
const FIRST_CHARACTER = 0;
const LAST_CHARACTER = -1;
const SUCCESS = 0;
const USAGE_ERROR = 2;
const typescriptPathSchema = z
  .string()
  .min(FAILURE)
  .refine(
    (candidate) =>
      [".ts", ".tsx", ".mts", ".cts"].some((extension) =>
        candidate.endsWith(extension),
      ),
    "unsupported TypeScript path",
  );
const typescriptPathsSchema = z.array(typescriptPathSchema).min(FAILURE);

const rejectSuppressionDirectives = async (
  repositoryRoot: string,
  paths: readonly string[],
): Promise<void> => {
  const suppressionDirective =
    /(?:\/\/|\/\*)\s*(?:eslint|oxlint)-disable(?:-next-line|-line)?\b/u;
  await Promise.all(
    paths.map(async (candidate) => {
      const contents = new TextDecoder("utf-8", { fatal: true }).decode(
        await readFile(resolve(repositoryRoot, candidate)),
      );
      if (suppressionDirective.test(contents)) {
        throw new Error(`${candidate} contains a lint suppression directive.`);
      }
    }),
  );
};

const findTrackedTypeScriptPaths = async (
  repositoryRoot: string,
): Promise<readonly string[]> => {
  const git = Bun.spawn(
    ["git", "ls-files", "-z", "--", "*.ts", "*.tsx", "*.mts", "*.cts"],
    { cwd: repositoryRoot, stderr: "inherit", stdout: "pipe" },
  );
  const [status, output] = await Promise.all([
    git.exited,
    new Response(git.stdout).arrayBuffer(),
  ]);
  if (status !== SUCCESS) {
    throw new Error(`Git could not list tracked TypeScript files (${status}).`);
  }

  const serializedPaths = new TextDecoder("utf-8", { fatal: true }).decode(
    output,
  );
  if (!serializedPaths.endsWith("\0")) {
    throw new Error("Git returned an empty or malformed TypeScript path list.");
  }
  return typescriptPathsSchema.parse(
    serializedPaths.slice(FIRST_CHARACTER, LAST_CHARACTER).split("\0"),
  );
};

const lintTrackedTypeScript = async (
  repositoryRoot: string,
  oxlint: string,
): Promise<Readonly<{ status: number; stderr: string; stdout: string }>> => {
  await access(oxlint);
  const paths = await findTrackedTypeScriptPaths(repositoryRoot);
  await rejectSuppressionDirectives(repositoryRoot, paths);
  const configuration = resolve(repositoryRoot, ".oxlintrc.json");
  const lint = Bun.spawn(
    [
      oxlint,
      "--config",
      configuration,
      "--disable-nested-config",
      "--deny-warnings",
      "--no-ignore",
      "--",
      ...paths,
    ],
    {
      cwd: repositoryRoot,
      stderr: "pipe",
      stdout: "pipe",
    },
  );
  const [status, stderr, stdout] = await Promise.all([
    lint.exited,
    new Response(lint.stderr).text(),
    new Response(lint.stdout).text(),
  ]);
  return { status, stderr, stdout };
};

const errorMessage = (error: unknown): string => {
  if (error instanceof Error) {
    return error.message;
  }
  return "Unknown lint failure.";
};
const main = async (): Promise<number> => {
  if (Bun.argv.length !== EXPECTED_ARGUMENT_COUNT) {
    process.stderr.write("Usage: lint-typescript\n");
    return USAGE_ERROR;
  }
  const repositoryRoot = resolve(import.meta.dir, "..");
  const oxlint = resolve(repositoryRoot, "node_modules/.bin/oxlint");
  const result = await lintTrackedTypeScript(repositoryRoot, oxlint);
  process.stderr.write(result.stderr);
  process.stdout.write(result.stdout);
  return result.status;
};

if (import.meta.main) {
  try {
    process.exitCode = await main();
  } catch (error) {
    process.stderr.write(`${errorMessage(error)}\n`);
    process.exitCode = FAILURE;
  }
}

export { findTrackedTypeScriptPaths, lintTrackedTypeScript };
