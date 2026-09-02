import type {
  MutationWorkflowAdapter,
  PreparedFile,
} from "./harness-reflection-mutation-workflow-types.ts";
import {
  closeSync,
  constants,
  fstatSync,
  fsyncSync,
  ftruncateSync,
  lstatSync,
  openSync,
  readFileSync,
  realpathSync,
  unlinkSync,
  writeFileSync,
  writeSync,
} from "node:fs";
import { dirname, isAbsolute, relative, resolve, sep, win32 } from "node:path";
import { validateInvariantRegistryText } from "./invariant-registry-cli.ts";

const approvedSurfaceCount = 1;
const maximumRetirementSurfaceCount = 1;
const newFileMode = 0o600;

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

const createFile = (
  target: string,
  replacement: string | undefined,
): boolean => {
  if (replacement === undefined) {
    return true;
  }
  try {
    const descriptor = openSync(
      target,
      constants.O_CREAT |
        constants.O_EXCL |
        constants.O_WRONLY |
        constants.O_NOFOLLOW,
      newFileMode,
    );
    try {
      writeFileSync(descriptor, replacement, "utf8");
      fsyncSync(descriptor);
      return true;
    } finally {
      closeSync(descriptor);
    }
  } catch (error) {
    if (errorCodeIs(error, "EEXIST")) {
      return false;
    }
    throw error;
  }
};

const removeOpenedFile = (target: string, descriptor: number): boolean => {
  const opened = fstatSync(descriptor, { bigint: true });
  closeSync(descriptor);
  const current = lstatSync(target, { bigint: true });
  if (
    !current.isFile() ||
    current.isSymbolicLink() ||
    opened.dev !== current.dev ||
    opened.ino !== current.ino
  ) {
    return false;
  }
  unlinkSync(target);
  return true;
};

const replaceFile = (
  target: string,
  expected: string,
  replacement: string | undefined,
): boolean => {
  try {
    const descriptor = openSync(
      target,
      constants.O_RDWR | constants.O_NOFOLLOW,
    );
    let open = true;
    try {
      if (readDescriptor(descriptor) !== expected) {
        return false;
      }
      if (replacement === undefined) {
        open = false;
        return removeOpenedFile(target, descriptor);
      }
      ftruncateSync(descriptor, 0);
      writeSync(descriptor, replacement, 0, "utf8");
      fsyncSync(descriptor);
      return true;
    } finally {
      if (open) {
        closeSync(descriptor);
      }
    }
  } catch (error) {
    if (errorCodeIs(error, "ENOENT")) {
      return false;
    }
    throw error;
  }
};

const validateSurfaces = (
  repositoryRoot: string,
  files: readonly PreparedFile[],
  kind: "approved-mutation" | "retirement",
): void => {
  if (
    (kind === "approved-mutation" && files.length !== approvedSurfaceCount) ||
    (kind === "retirement" && files.length > maximumRetirementSurfaceCount)
  ) {
    throw new Error("prepared-surface-count-invalid");
  }
  for (const file of files) {
    resolveMutationPath(repositoryRoot, file.path);
    if (kind === "approved-mutation" && file.contents.trim() === "") {
      throw new Error("prepared-surface-empty");
    }
  }
};

const createRepositoryMutationAdapter = (
  root: string,
): MutationWorkflowAdapter => {
  const repositoryRoot = realpathSync(root);
  return {
    compareAndSwap: (path, expected, replacement) => {
      const target = resolveMutationPath(repositoryRoot, path);
      return expected === undefined
        ? createFile(target, replacement)
        : replaceFile(target, expected, replacement);
    },
    read: (path) => readFile(repositoryRoot, path),
    validatePreparedRegistry: (contents) =>
      validateInvariantRegistryText(contents, repositoryRoot),
    validatePreparedSurfaces: (files, kind) => {
      validateSurfaces(repositoryRoot, files, kind);
    },
  };
};

export { createRepositoryMutationAdapter };
