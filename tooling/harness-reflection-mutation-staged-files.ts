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
import type { AtomicFileOptions } from "./harness-reflection-mutation-atomic-file.ts";
import { randomUUID } from "node:crypto";

type StagedFileChange = Readonly<{
  expected: string | undefined;
  replacement: string;
  target: string;
}>;
type StagedFile = StagedFileChange &
  Readonly<{
    descriptor: number;
    temporary: string;
  }>;

const defaultFileMode = 0o600;
const permissionMask = 0o777;

const errorCodeIs = (error: unknown, code: string): boolean =>
  error instanceof Error && Reflect.get(error, "code") === code;

const inspectCurrent = (
  target: string,
): Readonly<{ contents: string; mode: number }> | undefined => {
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
    const contents = new TextDecoder("utf-8", { fatal: true }).decode(
      readFileSync(descriptor),
    );
    return { contents, mode: metadata.mode & permissionMask };
  } catch (error) {
    if (error instanceof TypeError) {
      throw new TypeError("mutation-file-invalid-utf8", { cause: error });
    }
    throw error;
  } finally {
    closeSync(descriptor);
  }
};

const temporaryPath = (target: string): string =>
  join(dirname(target), `.${basename(target)}.${randomUUID()}.tmp`);

const stageFile = (change: StagedFileChange): StagedFile => {
  const mode = inspectCurrent(change.target)?.mode ?? defaultFileMode;
  const temporary = temporaryPath(change.target);
  const descriptor = openSync(
    temporary,
    constants.O_CREAT |
      constants.O_EXCL |
      constants.O_WRONLY |
      constants.O_NOFOLLOW,
    mode,
  );
  try {
    fchmodSync(descriptor, mode);
    writeFileSync(descriptor, change.replacement, "utf8");
    fsyncSync(descriptor);
    return { ...change, descriptor, temporary };
  } catch (error) {
    closeSync(descriptor);
    unlinkSync(temporary);
    throw error;
  }
};

const removeOwnedTemporary = (file: StagedFile): void => {
  const opened = fstatSync(file.descriptor, { bigint: true });
  try {
    const current = lstatSync(file.temporary, { bigint: true });
    if (
      current.isFile() &&
      !current.isSymbolicLink() &&
      current.dev === opened.dev &&
      current.ino === opened.ino
    ) {
      unlinkSync(file.temporary);
    }
  } catch (error) {
    if (!errorCodeIs(error, "ENOENT")) {
      throw error;
    }
  }
};

const releaseStagedFile = (file: StagedFile, renamed: boolean): void => {
  if (!renamed) {
    removeOwnedTemporary(file);
  }
  closeSync(file.descriptor);
};

const replaceMatchingFiles = (
  changes: readonly StagedFileChange[],
  onAttempt: (index: number) => void,
  options: AtomicFileOptions = {},
): boolean => {
  const staged: StagedFile[] = [];
  const renamed = new Set<number>();
  try {
    for (const change of changes) {
      staged.push(stageFile(change));
    }
    if (
      !staged.every(
        ({ expected, target }) => inspectCurrent(target)?.contents === expected,
      )
    ) {
      return false;
    }
    const renameFile = options.renameFile ?? renameSync;
    for (const [index, file] of staged.entries()) {
      onAttempt(index);
      renameFile(file.temporary, file.target);
      renamed.add(index);
    }
    return true;
  } finally {
    for (const [index, file] of staged.entries()) {
      releaseStagedFile(file, renamed.has(index));
    }
  }
};

export { replaceMatchingFiles, type StagedFileChange };
