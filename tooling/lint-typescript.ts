import { access, lstat, readFile, realpath } from "node:fs/promises";
import { isAbsolute, relative, resolve, sep } from "node:path";
import { discoverTrackedTypeScriptPaths } from "./tracked-typescript-paths.ts";

const EXPECTED_ARGUMENT_COUNT = 2;
const FAILURE = 1;
const OWNED_LINK_COUNT = 1;
const SUCCESS = 0;
const USAGE_ERROR = 2;
type TypeScriptSource = Readonly<{ contents: string; path: string }>;

const isOutside = (root: string, path: string): boolean => {
  const pathFromRoot = relative(root, path);
  return (
    pathFromRoot === ".." ||
    pathFromRoot.startsWith(`..${sep}`) ||
    isAbsolute(pathFromRoot)
  );
};

const assertOwnedTrackedFile = async (
  repositoryRoot: string,
  candidate: string,
): Promise<string> => {
  const canonicalRoot = await realpath(repositoryRoot);
  const absolutePath = resolve(canonicalRoot, candidate);
  if (
    isOutside(canonicalRoot, absolutePath) ||
    (await realpath(absolutePath)) !== absolutePath
  ) {
    throw new Error(
      `${candidate}: tracked TypeScript path is not confined to the repository.`,
    );
  }
  const metadata = await lstat(absolutePath);
  if (!metadata.isFile() || metadata.nlink !== OWNED_LINK_COUNT) {
    throw new Error(
      `${candidate}: tracked TypeScript path is not an owned regular file.`,
    );
  }
  return absolutePath;
};

const readTrackedSources = (
  repositoryRoot: string,
  paths: readonly string[],
): Promise<readonly TypeScriptSource[]> => {
  const suppressionDirective =
    /(?:\/\/|\/\*)\s*(?:eslint|oxlint)-disable(?:-next-line|-line)?\b/u;
  return Promise.all(
    paths.map(async (candidate) => {
      const ownedPath = await assertOwnedTrackedFile(repositoryRoot, candidate);
      const contents = new TextDecoder("utf-8", { fatal: true }).decode(
        await readFile(ownedPath),
      );
      if (suppressionDirective.test(contents)) {
        throw new Error(`${candidate} contains a lint suppression directive.`);
      }
      return { contents, path: candidate };
    }),
  );
};

const findTrackedTypeScriptPaths = async (
  repositoryRoot: string,
): Promise<readonly string[]> => {
  const result = await discoverTrackedTypeScriptPaths(repositoryRoot);
  if (result.status !== SUCCESS) {
    throw new Error(
      `Git could not list tracked TypeScript files (${result.status}).`,
    );
  }
  if (result.trackedCount === 0) {
    throw new Error("Git returned an empty or malformed TypeScript path list.");
  }
  return result.paths;
};

const assertSourcesUnchanged = async (
  repositoryRoot: string,
  sources: readonly TypeScriptSource[],
): Promise<void> => {
  const currentPaths = await findTrackedTypeScriptPaths(repositoryRoot);
  if (
    currentPaths.length !== sources.length ||
    currentPaths.some((path, index) => path !== sources[index]?.path)
  ) {
    throw new Error("Tracked TypeScript paths changed while lint ran.");
  }
  const currentSources = await readTrackedSources(repositoryRoot, currentPaths);
  if (
    currentSources.some(
      (source, index) => source.contents !== sources[index]?.contents,
    )
  ) {
    throw new Error("Tracked TypeScript contents changed while lint ran.");
  }
};

const lintTrackedTypeScript = async (
  repositoryRoot: string,
  oxlint: string,
  commandPrefix: readonly string[] = [oxlint],
): Promise<Readonly<{ status: number; stderr: string; stdout: string }>> => {
  await access(oxlint);
  const paths = await findTrackedTypeScriptPaths(repositoryRoot);
  const sources = await readTrackedSources(repositoryRoot, paths);
  const configuration = resolve(repositoryRoot, ".oxlintrc.json");
  const lint = Bun.spawn(
    [
      ...commandPrefix,
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
  await assertSourcesUnchanged(repositoryRoot, sources);
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
