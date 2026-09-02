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
import {
  parseInvariantRegistry,
  validateInvariantRegistry,
} from "./invariant-registry-contract.ts";
import type { OracleInspection } from "./invariant-registry-contract.ts";
import { ZodError } from "zod";

const isMissingPathError = (error: unknown): boolean =>
  error instanceof Error && Reflect.get(error, "code") === "ENOENT";

const parseInvariantRegistryJson = (source: string): unknown => {
  try {
    return JSON.parse(source);
  } catch {
    throw new Error("invariant registry must be valid JSON");
  }
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

export { parseInvariantRegistryJson, validateInvariantRegistryText };
