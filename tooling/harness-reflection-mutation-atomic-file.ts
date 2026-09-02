import { basename, dirname, join } from "node:path";
import {
  closeSync,
  constants,
  fchmodSync,
  fstatSync,
  fsyncSync,
  lstatSync,
  openSync,
  readFileSync,
  renameSync,
  unlinkSync,
  writeFileSync,
} from "node:fs";
import { randomUUID } from "node:crypto";

type AtomicFileOptions = Readonly<{
  renameFile?: (source: string, target: string) => void;
}>;
type OpenedFile = Readonly<{
  contents: string;
  descriptor: number;
  mode: number;
}>;
type PreparedReplacement = Readonly<{
  expected: string | undefined;
  mode: number;
  renameFile: (source: string, target: string) => void;
  replacement: string;
  target: string;
}>;
type MatchingReplacement = Readonly<{
  expected: string | undefined;
  replacement: string | undefined;
}>;

const defaultFileMode = 0o600;
const permissionMask = 0o777;

const errorCodeIs = (error: unknown, code: string): boolean =>
  error instanceof Error && Reflect.get(error, "code") === code;

const decodeDescriptor = (descriptor: number): string => {
  try {
    return new TextDecoder("utf-8", { fatal: true }).decode(
      readFileSync(descriptor),
    );
  } catch {
    throw new Error("mutation-file-invalid-utf8");
  }
};

const openRegularFile = (target: string): OpenedFile | undefined => {
  let descriptor: number | undefined = undefined;
  try {
    descriptor = openSync(target, constants.O_RDONLY | constants.O_NOFOLLOW);
  } catch (error) {
    if (errorCodeIs(error, "ENOENT")) {
      return undefined;
    }
    throw error;
  }
  if (descriptor === undefined) {
    return undefined;
  }
  try {
    const metadata = fstatSync(descriptor);
    if (!metadata.isFile()) {
      throw new Error("mutation-path-not-regular-file");
    }
    if (metadata.nlink > 1) {
      throw new Error("mutation-path-hard-linked");
    }
    return {
      contents: decodeDescriptor(descriptor),
      descriptor,
      mode: metadata.mode & permissionMask,
    };
  } catch (error) {
    closeSync(descriptor);
    throw error;
  }
};

const closeOpenedFile = (opened: OpenedFile | undefined): void => {
  if (opened !== undefined) {
    closeSync(opened.descriptor);
  }
};

const currentContentsMatch = (
  target: string,
  expected: string | undefined,
): boolean => {
  const opened = openRegularFile(target);
  try {
    return opened?.contents === expected;
  } finally {
    closeOpenedFile(opened);
  }
};

const removeOwnedPath = (path: string, descriptor: number): void => {
  const opened = fstatSync(descriptor, { bigint: true });
  try {
    const current = lstatSync(path, { bigint: true });
    if (
      current.isFile() &&
      !current.isSymbolicLink() &&
      current.dev === opened.dev &&
      current.ino === opened.ino
    ) {
      unlinkSync(path);
    }
  } catch (error) {
    if (!errorCodeIs(error, "ENOENT")) {
      throw error;
    }
  }
};

const temporaryPath = (target: string): string =>
  join(dirname(target), `.${basename(target)}.${randomUUID()}.tmp`);

const replaceFromTemporaryFile = (prepared: PreparedReplacement): boolean => {
  const { expected, mode, renameFile, replacement, target } = prepared;
  const temporary = temporaryPath(target);
  const descriptor = openSync(
    temporary,
    constants.O_CREAT |
      constants.O_EXCL |
      constants.O_WRONLY |
      constants.O_NOFOLLOW,
    mode,
  );
  let renamed = false;
  try {
    fchmodSync(descriptor, mode);
    writeFileSync(descriptor, replacement, "utf8");
    fsyncSync(descriptor);
    if (!currentContentsMatch(target, expected)) {
      return false;
    }
    renameFile(temporary, target);
    renamed = true;
    return true;
  } finally {
    if (!renamed) {
      removeOwnedPath(temporary, descriptor);
    }
    closeSync(descriptor);
  }
};

const removeExpectedFile = (target: string, expected: string): boolean => {
  const opened = openRegularFile(target);
  if (opened?.contents !== expected) {
    closeOpenedFile(opened);
    return false;
  }
  const identity = fstatSync(opened.descriptor, { bigint: true });
  closeSync(opened.descriptor);
  const current = lstatSync(target, { bigint: true });
  if (
    !current.isFile() ||
    current.isSymbolicLink() ||
    current.dev !== identity.dev ||
    current.ino !== identity.ino
  ) {
    return false;
  }
  unlinkSync(target);
  return true;
};

const replaceMatchingFile = (
  target: string,
  change: MatchingReplacement,
  options: AtomicFileOptions = {},
): boolean => {
  const { expected, replacement } = change;
  if (replacement === undefined) {
    return expected === undefined || removeExpectedFile(target, expected);
  }
  const opened = openRegularFile(target);
  if (opened?.contents !== expected) {
    closeOpenedFile(opened);
    return false;
  }
  const mode = opened?.mode ?? defaultFileMode;
  closeOpenedFile(opened);
  return replaceFromTemporaryFile({
    expected,
    mode,
    renameFile: options.renameFile ?? renameSync,
    replacement,
    target,
  });
};

export { replaceMatchingFile, type AtomicFileOptions };
