import { isAbsolute, relative, resolve, sep, win32 } from "node:path";
import {
  parseInvariantRegistry,
  validateInvariantRegistry,
} from "./invariant-registry-contract.ts";
import { realpathSync, statSync } from "node:fs";
import type { OracleInspection } from "./invariant-registry-contract.ts";
import { ZodError } from "zod";
import { realpath } from "node:fs/promises";

const argumentOffset = 2;
const repositoryRoot = resolve(import.meta.dir, "..");
const defaultRegistryPath = "harness/invariants/registry.json";
const oracleInvocationLength = 3;

const isOutside = (root: string, path: string): boolean => {
  const pathFromRoot = relative(root, path);
  return (
    pathFromRoot === ".." ||
    pathFromRoot.startsWith(`..${sep}`) ||
    isAbsolute(pathFromRoot)
  );
};

const isMissingPathError = (error: unknown): boolean =>
  error instanceof Error && Reflect.get(error, "code") === "ENOENT";

const resolveRepositoryRoot = async (): Promise<string> => {
  try {
    return await realpath(repositoryRoot);
  } catch {
    throw new Error("unable to resolve invariant registry root");
  }
};

const readInvariantRegistryBytes = async (
  path: string,
): Promise<Uint8Array> => {
  try {
    return await Bun.file(path).bytes();
  } catch {
    throw new Error("unable to read invariant registry");
  }
};

const decodeInvariantRegistry = async (path: string): Promise<string> => {
  const bytes = await readInvariantRegistryBytes(path);
  try {
    return new TextDecoder("utf-8", { fatal: true }).decode(bytes);
  } catch {
    throw new Error("invariant registry must be valid UTF-8");
  }
};

const loadInvariantRegistry = async (path: string): Promise<unknown> => {
  const source = await decodeInvariantRegistry(path);
  try {
    return JSON.parse(source);
  } catch {
    throw new Error("invariant registry must be valid JSON");
  }
};

const resolveRegistryPath = (root: string, path: string): string => {
  const windowsPath = win32.normalize(path);
  if (
    isAbsolute(path) ||
    win32.parse(path).root !== "" ||
    windowsPath === ".." ||
    windowsPath.startsWith(`..${win32.sep}`)
  ) {
    throw new Error("invariant registry path must stay within the repository");
  }
  const resolvedPath = resolve(root, path);
  if (isOutside(root, resolvedPath)) {
    throw new Error("invariant registry path must stay within the repository");
  }
  return resolvedPath;
};

const resolveExistingRegistryPath = async (path: string): Promise<string> => {
  try {
    return await realpath(path);
  } catch {
    throw new Error("unable to read invariant registry");
  }
};

const resolveRegistryTarget = async (
  root: string,
  path: string,
): Promise<string> => {
  const target = await resolveExistingRegistryPath(path);
  if (isOutside(root, target)) {
    throw new Error("invariant registry path must stay within the repository");
  }
  return target;
};

const gitTracksPath = (root: string, path: string): boolean =>
  Bun.spawnSync([
    "git",
    "-C",
    root,
    "ls-files",
    "--error-unmatch",
    "--",
    relative(root, path),
  ]).exitCode === 0;

const inspectOracle = (
  root: string,
  path: string,
  invocation: readonly string[],
): OracleInspection => {
  try {
    const target = realpathSync(path);
    if (isOutside(root, target)) {
      return { discovered: false, kind: "missing", tracked: false };
    }
    if (!statSync(target).isFile()) {
      return { discovered: false, kind: "non-regular", tracked: false };
    }
    const tracked = gitTracksPath(root, path);
    const repositoryPath = relative(root, path);
    const discovered =
      repositoryPath.endsWith(".test.ts") &&
      invocation.length === oracleInvocationLength &&
      invocation[0] === "bun" &&
      invocation[1] === "test" &&
      invocation[2] === repositoryPath;
    return { discovered, kind: "regular-file", tracked };
  } catch (error) {
    if (isMissingPathError(error)) {
      return { discovered: false, kind: "missing", tracked: false };
    }
    throw new Error("Oracle test path could not be checked.", { cause: error });
  }
};

const parseRegistry = (
  input: unknown,
): ReturnType<typeof parseInvariantRegistry> => {
  try {
    return parseInvariantRegistry(input);
  } catch (error) {
    if (error instanceof ZodError) {
      throw new TypeError("invalid invariant registry", { cause: error });
    }
    throw error;
  }
};

const main = async (): Promise<number> => {
  const cliArguments = Bun.argv.slice(argumentOffset);
  if (cliArguments.length > 1) {
    throw new Error("Usage: invariant-registry-cli [registry-path]");
  }
  const displayPath = cliArguments[0] ?? defaultRegistryPath;
  const root = await resolveRepositoryRoot();
  const registryPath = await resolveRegistryTarget(
    root,
    resolveRegistryPath(root, displayPath),
  );
  const registry = parseRegistry(await loadInvariantRegistry(registryPath));
  const diagnostics = validateInvariantRegistry(registry, {
    inspectOracle: (testPath, invocation): OracleInspection =>
      inspectOracle(root, testPath, invocation),
    repositoryRoot: root,
  });
  if (diagnostics.length > 0) {
    throw new Error(
      diagnostics.map(({ path, message }) => `${path}: ${message}`).join("\n"),
    );
  }
  process.stdout.write(`Invariant registry passed: ${displayPath}\n`);
  return 0;
};

if (import.meta.main) {
  try {
    process.exitCode = await main();
  } catch (error) {
    process.stderr.write(
      `invariant-registry: ${error instanceof Error ? error.message : String(error)}\n`,
    );
    process.exitCode = 1;
  }
}

export { loadInvariantRegistry };
