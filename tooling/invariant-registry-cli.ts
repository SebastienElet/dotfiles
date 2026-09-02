import { isAbsolute, relative, resolve, sep, win32 } from "node:path";
import {
  parseInvariantRegistry,
  validateInvariantRegistry,
} from "./invariant-registry-contract.ts";
import { existsSync } from "node:fs";

const argumentOffset = 2;
const repositoryRoot = resolve(import.meta.dir, "..");
const defaultRegistryPath = "harness/invariants/registry.json";

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

const resolveRegistryPath = (path: string): string => {
  const windowsPath = win32.normalize(path);
  if (
    isAbsolute(path) ||
    win32.parse(path).root !== "" ||
    windowsPath === ".." ||
    windowsPath.startsWith(`..${win32.sep}`)
  ) {
    throw new Error("invariant registry path must stay within the repository");
  }
  const resolvedPath = resolve(repositoryRoot, path);
  const pathFromRoot = relative(repositoryRoot, resolvedPath);
  if (
    pathFromRoot === ".." ||
    pathFromRoot.startsWith(`..${sep}`) ||
    isAbsolute(pathFromRoot)
  ) {
    throw new Error("invariant registry path must stay within the repository");
  }
  return resolvedPath;
};

const main = async (): Promise<number> => {
  const cliArguments = Bun.argv.slice(argumentOffset);
  if (cliArguments.length > 1) {
    throw new Error("Usage: invariant-registry-cli [registry-path]");
  }
  const displayPath = cliArguments[0] ?? defaultRegistryPath;
  const registryPath = resolveRegistryPath(displayPath);
  const registry = parseInvariantRegistry(
    await loadInvariantRegistry(registryPath),
  );
  const diagnostics = validateInvariantRegistry(registry, {
    pathExists: (testPath): boolean =>
      existsSync(resolve(repositoryRoot, testPath)),
    repositoryRoot,
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
