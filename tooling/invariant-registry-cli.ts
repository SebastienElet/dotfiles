import { isAbsolute, relative, resolve, sep, win32 } from "node:path";
import {
  parseInvariantRegistryJson,
  validateInvariantRegistryText,
} from "./invariant-registry-repository-validator.ts";
import { realpath } from "node:fs/promises";
import { runVerifiedInvariantOracles } from "./invariant-registry-runtime-oracles.ts";

const argumentOffset = 2;
const repositoryRoot = resolve(import.meta.dir, "..");
const defaultRegistryPath = "harness/invariants/registry.json";

const isOutside = (root: string, path: string): boolean => {
  const pathFromRoot = relative(root, path);
  return (
    pathFromRoot === ".." ||
    pathFromRoot.startsWith(`..${sep}`) ||
    isAbsolute(pathFromRoot)
  );
};

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

const loadInvariantRegistry = async (path: string): Promise<unknown> =>
  parseInvariantRegistryJson(await decodeInvariantRegistry(path));

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
  const registry = validateInvariantRegistryText(
    await decodeInvariantRegistry(registryPath),
    root,
  );
  await runVerifiedInvariantOracles(registry, root);
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
