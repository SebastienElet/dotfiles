import {
  type AtomicFileOptions,
  replaceMatchingFile,
} from "./harness-reflection-mutation-atomic-file.ts";
import type {
  MutationWorkflowAdapter,
  PreparedFile,
} from "./harness-reflection-mutation-workflow-types.ts";
import {
  closeSync,
  constants,
  fstatSync,
  lstatSync,
  openSync,
  readFileSync,
  realpathSync,
} from "node:fs";
import { dirname, isAbsolute, relative, resolve, sep, win32 } from "node:path";
import { createMutationLock } from "./harness-reflection-mutation-lock.ts";
import { replaceMatchingFiles } from "./harness-reflection-mutation-staged-files.ts";
import { resolveSupportedTarget } from "./harness-reflection-mutation-surfaces.ts";
import { validateInvariantRegistryText } from "./invariant-registry-repository-validator.ts";

const approvedSurfaceCount = 1;
const maximumRetirementSurfaceCount = 1;
type RepositoryMutationAdapterOptions = AtomicFileOptions;

const isOutside = (root: string, path: string): boolean => {
  const fromRoot = relative(root, path);
  return (
    fromRoot === ".." || fromRoot.startsWith(`..${sep}`) || isAbsolute(fromRoot)
  );
};

const errorCodeIs = (error: unknown, code: string): boolean =>
  error instanceof Error && Reflect.get(error, "code") === code;

const resolveMutationPath = (repositoryRoot: string, path: string): string => {
  const windowsPath = win32.normalize(path);
  if (
    path === "" ||
    isAbsolute(path) ||
    win32.parse(path).root !== "" ||
    windowsPath === ".." ||
    windowsPath.startsWith(`..${win32.sep}`)
  ) {
    throw new Error("mutation-path-outside-repository");
  }
  const target = resolve(repositoryRoot, path);
  const parent = realpathSync(dirname(target));
  if (isOutside(repositoryRoot, parent)) {
    throw new Error("mutation-path-outside-repository");
  }
  try {
    const metadata = lstatSync(target);
    if (!metadata.isFile() || metadata.isSymbolicLink()) {
      throw new Error("mutation-path-not-regular-file");
    }
    if (metadata.nlink > 1) {
      throw new Error("mutation-path-hard-linked");
    }
    if (isOutside(repositoryRoot, realpathSync(target))) {
      throw new Error("mutation-path-outside-repository");
    }
  } catch (error) {
    if (!errorCodeIs(error, "ENOENT")) {
      throw error;
    }
  }
  return target;
};

const readDescriptor = (descriptor: number): string => {
  const metadata = fstatSync(descriptor);
  if (!metadata.isFile()) {
    throw new Error("mutation-path-not-regular-file");
  }
  if (metadata.nlink > 1) {
    throw new Error("mutation-path-hard-linked");
  }
  try {
    return new TextDecoder("utf-8", { fatal: true }).decode(
      readFileSync(descriptor),
    );
  } catch {
    throw new Error("mutation-file-invalid-utf8");
  }
};

const readFile = (repositoryRoot: string, path: string): string | undefined => {
  const target = resolveMutationPath(repositoryRoot, path);
  try {
    const descriptor = openSync(
      target,
      constants.O_RDONLY | constants.O_NOFOLLOW,
    );
    try {
      return readDescriptor(descriptor);
    } finally {
      closeSync(descriptor);
    }
  } catch (error) {
    if (errorCodeIs(error, "ENOENT")) {
      return undefined;
    }
    throw error;
  }
};

const validateSurfaces = (
  repositoryRoot: string,
  files: readonly PreparedFile[],
  transition: Parameters<
    MutationWorkflowAdapter["validatePreparedSurfaces"]
  >[1],
): void => {
  const { kind, target } = transition;
  if (
    (kind === "approved-mutation" && files.length !== approvedSurfaceCount) ||
    (kind === "retirement" && files.length > maximumRetirementSurfaceCount)
  ) {
    throw new Error("prepared-surface-count-invalid");
  }
  if (files.length === 0) {
    return;
  }
  const supportedTarget = resolveSupportedTarget(target);
  for (const file of files) {
    resolveMutationPath(repositoryRoot, file.path);
    if (file.path !== supportedTarget.path) {
      throw new Error("unsupported-control-surface");
    }
    if (kind === "approved-mutation" && file.contents.trim() === "") {
      throw new Error("prepared-surface-empty");
    }
  }
};

const createRepositoryMutationAdapter = (
  root: string,
  options: RepositoryMutationAdapterOptions = {},
): MutationWorkflowAdapter => {
  const repositoryRoot = realpathSync(root);
  const lock = createMutationLock(repositoryRoot);
  return {
    applyMatchingBatch: (snapshots, onAttempt) => {
      if (!lock.isHeld()) {
        throw new Error("mutation-lock-required");
      }
      const changes = snapshots.map(({ before, contents, path }) => ({
        expected: before,
        replacement: contents,
        target: resolveMutationPath(repositoryRoot, path),
      }));
      return replaceMatchingFiles(
        changes,
        (index) => {
          const snapshot = snapshots[index];
          if (snapshot === undefined) {
            throw new Error("staged-snapshot-missing");
          }
          onAttempt(snapshot);
        },
        options,
      );
    },
    replaceMatching: (path, expected, replacement) => {
      if (!lock.isHeld()) {
        throw new Error("mutation-lock-required");
      }
      const target = resolveMutationPath(repositoryRoot, path);
      return replaceMatchingFile(target, { expected, replacement }, options);
    },
    read: (path) => readFile(repositoryRoot, path),
    withMutationLock: lock.withLock,
    validatePreparedRegistry: (contents) =>
      validateInvariantRegistryText(contents, repositoryRoot),
    validatePreparedSurfaces: (files, transition) => {
      validateSurfaces(repositoryRoot, files, transition);
    },
  };
};

export {
  createRepositoryMutationAdapter,
  type RepositoryMutationAdapterOptions,
};
