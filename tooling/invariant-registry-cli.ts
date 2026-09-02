import {
  type FileSnapshot,
  inspectOracleWithProbes,
} from "./invariant-registry-oracle-inspection.ts";
import {
  closeSync,
  constants,
  fstatSync,
  lstatSync,
  openSync,
  realpathSync,
} from "node:fs";
import { isAbsolute, relative, resolve, sep, win32 } from "node:path";
import {
  parseInvariantRegistry,
  validateInvariantRegistry,
} from "./invariant-registry-contract.ts";
import type { OracleInspection } from "./invariant-registry-contract.ts";
import { ZodError } from "zod";
import { realpath } from "node:fs/promises";

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

const parseInvariantRegistryJson = (source: string): unknown => {
  try {
    return JSON.parse(source);
  } catch {
    throw new Error("invariant registry must be valid JSON");
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

const parseGitIndexMode = (
  output: string,
  path: string,
): string | undefined => {
  if (output === "") {
    return undefined;
  }
  const match =
    /^(?<mode>[0-9]{6}) [0-9a-f]{40,64} 0\t(?<path>[^\n]+)\n?$/u.exec(output);
  if (match?.groups?.path !== path) {
    throw new Error("Git index oracle probe was malformed.");
  }
  return match.groups.mode;
};

const gitIndexMode = (root: string, path: string): string | undefined => {
  const result = Bun.spawnSync([
    "git",
    "-C",
    root,
    "ls-files",
    "-s",
    "--",
    path,
  ]);
  if (result.exitCode !== 0) {
    throw new Error("Git index oracle probe failed.");
  }
  const output = new TextDecoder("utf-8", { fatal: true }).decode(
    result.stdout,
  );
  return parseGitIndexMode(output, path);
};

const fileSnapshot = (path: string): FileSnapshot => {
  try {
    const stats = lstatSync(path, { bigint: true });
    return stats.isFile()
      ? { device: stats.dev, inode: stats.ino, kind: "regular-file" }
      : { kind: stats.isSymbolicLink() ? "symlink" : "non-regular" };
  } catch (error) {
    if (isMissingPathError(error)) {
      return { kind: "missing" };
    }
    throw error;
  }
};

const descriptorSnapshot = (descriptor: number): FileSnapshot => {
  const stats = fstatSync(descriptor, { bigint: true });
  return stats.isFile()
    ? { device: stats.dev, inode: stats.ino, kind: "regular-file" }
    : { kind: "non-regular" };
};

const openNoFollow = (path: string): number =>
  openSync(path, constants.O_RDONLY | constants.O_NOFOLLOW);

const inspectOracle = (
  root: string,
  path: string,
  invocation: readonly string[],
): OracleInspection => {
  try {
    return inspectOracleWithProbes(
      { invocation, path, root },
      {
        close: closeSync,
        fstat: descriptorSnapshot,
        gitIndexMode,
        lstat: fileSnapshot,
        openNoFollow,
        realpath: realpathSync,
      },
    );
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
      const diagnostics: string[] = [];
      for (const issue of error.issues) {
        diagnostics.push(
          `${issue.path.join(".") || "registry"}: Invalid value.`,
        );
      }
      throw new TypeError(diagnostics.join("\n"), { cause: error });
    }
    throw error;
  }
};

const validateInvariantRegistryText = (
  source: string,
  root: string,
): ReturnType<typeof parseInvariantRegistry> => {
  const registry = parseRegistry(parseInvariantRegistryJson(source));
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
  return registry;
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
  validateInvariantRegistryText(
    await decodeInvariantRegistry(registryPath),
    root,
  );
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

export { loadInvariantRegistry, validateInvariantRegistryText };
