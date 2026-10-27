import {
  closeSync,
  constants,
  fstatSync,
  lstatSync,
  mkdirSync,
  openSync,
  readFileSync,
  renameSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import type { Agent } from "./install-agent-hooks.ts";
import { join } from "node:path";

const indentation = 2;
const maximumBytes = 1_048_576;
const directoryMode = 0o700;
const fileMode = 0o600;
const permissionMask = 0o777;

function configurationBytes(
  path: string,
): Readonly<{ bytes: string; mode: number; identity: string }> | undefined {
  if (lstatSync(path, { throwIfNoEntry: false }) === undefined) {
    return undefined;
  }
  const descriptor = openSync(
    path,
    constants.O_RDONLY | constants.O_NOFOLLOW | constants.O_NONBLOCK,
  );
  try {
    const metadata = fstatSync(descriptor);
    if (
      !metadata.isFile() ||
      metadata.nlink !== 1 ||
      metadata.size > maximumBytes
    ) {
      throw new Error(
        "Hook configuration must be a bounded single-link regular file",
      );
    }
    return {
      bytes: new TextDecoder("utf-8", { fatal: true }).decode(
        readFileSync(descriptor),
      ),
      mode: metadata.mode & permissionMask,
      identity: `${metadata.dev}:${metadata.ino}`,
    };
  } finally {
    closeSync(descriptor);
  }
}

function directoryIdentity(path: string): string {
  const metadata = lstatSync(path);
  if (!metadata.isDirectory() || metadata.isSymbolicLink()) {
    throw new Error(
      `Refusing symbolic or non-directory hook destination: ${path}`,
    );
  }
  return `${metadata.dev}:${metadata.ino}`;
}

function updateHookConfiguration(
  home: string,
  agent: Agent,
  reconcile: (input: unknown) => unknown,
): void {
  const homeIdentity = directoryIdentity(home);
  const directory = join(home, `.${agent}`);
  mkdirSync(directory, { mode: directoryMode, recursive: true });
  const identity = directoryIdentity(directory);
  const destination = join(
    directory,
    agent === "claude" ? "settings.json" : "hooks.json",
  );
  const lock = `${destination}.dotfiles-lock`;
  mkdirSync(lock, { mode: directoryMode });
  const temporary = join(lock, "configuration.json");
  try {
    const previous = configurationBytes(destination);
    const next = JSON.stringify(
      reconcile(previous === undefined ? {} : JSON.parse(previous.bytes)),
      null,
      indentation,
    );
    if (Buffer.byteLength(next) > maximumBytes) {
      throw new Error("Hook configuration is oversized");
    }
    if (
      directoryIdentity(home) !== homeIdentity ||
      directoryIdentity(directory) !== identity ||
      JSON.stringify(configurationBytes(destination)) !==
        JSON.stringify(previous)
    ) {
      throw new Error("Hook configuration changed during installation");
    }
    if (previous?.bytes === next) {
      return;
    }
    writeFileSync(temporary, next, {
      flag: "wx",
      mode: previous?.mode ?? fileMode,
    });
    renameSync(temporary, destination);
  } finally {
    rmSync(lock, { recursive: true });
  }
}

export { updateHookConfiguration };
