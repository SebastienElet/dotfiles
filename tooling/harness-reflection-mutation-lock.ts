import {
  closeSync,
  constants,
  fstatSync,
  fsyncSync,
  lstatSync,
  openSync,
  unlinkSync,
  writeFileSync,
} from "node:fs";
import { resolve } from "node:path";

type MaybePromise<Value> = Value | Promise<Value>;
type MutationLock = Readonly<{
  isHeld: () => boolean;
  withLock: <Value>(action: () => MaybePromise<Value>) => Promise<Value>;
}>;

const lockFileMode = 0o600;

const errorCodeIs = (error: unknown, code: string): boolean =>
  error instanceof Error && Reflect.get(error, "code") === code;

const removeOwnedLock = (path: string, descriptor: number): void => {
  const opened = fstatSync(descriptor, { bigint: true });
  closeSync(descriptor);
  try {
    const current = lstatSync(path, { bigint: true });
    if (
      current.isFile() &&
      !current.isSymbolicLink() &&
      opened.dev === current.dev &&
      opened.ino === current.ino
    ) {
      unlinkSync(path);
    }
  } catch (error) {
    if (!errorCodeIs(error, "ENOENT")) {
      throw error;
    }
  }
};

const createMutationLock = (repositoryRoot: string): MutationLock => {
  const path = resolve(repositoryRoot, ".harness-reflection-mutation.lock");
  let held = false;
  const withLock = async <Value>(
    action: () => MaybePromise<Value>,
  ): Promise<Value> => {
    let descriptor: number | undefined = undefined;
    try {
      descriptor = openSync(
        path,
        constants.O_CREAT |
          constants.O_EXCL |
          constants.O_WRONLY |
          constants.O_NOFOLLOW,
        lockFileMode,
      );
    } catch (error) {
      if (errorCodeIs(error, "EEXIST")) {
        throw new Error("mutation-lock-unavailable", { cause: error });
      }
      throw error;
    }
    if (descriptor === undefined) {
      throw new Error("mutation-lock-open-failed");
    }
    try {
      writeFileSync(
        descriptor,
        JSON.stringify({
          createdAt: new Date().toISOString(),
          pid: process.pid,
        }),
        "utf8",
      );
      fsyncSync(descriptor);
      held = true;
      return await action();
    } finally {
      held = false;
      removeOwnedLock(path, descriptor);
    }
  };
  return { isHeld: () => held, withLock };
};

export { createMutationLock, type MutationLock };
