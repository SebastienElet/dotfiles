import { createHash, randomUUID } from "node:crypto";
import { constants, type BigIntStats } from "node:fs";
import {
  link,
  lstat,
  mkdir,
  open,
  unlink,
  type FileHandle,
} from "node:fs/promises";
import { join } from "node:path";
import { DictionaryInstallationError } from "./install-hunspell-dictionary-error.ts";

type DirectoryIdentity = Readonly<{
  device: bigint;
  inode: bigint;
  path: string;
}>;

function hasCode(error: unknown, code: string): boolean {
  return error instanceof Error && "code" in error && error.code === code;
}

function errorMessage(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}

function sha256(content: Uint8Array): string {
  return createHash("sha256").update(content).digest("hex");
}

async function inspectDirectory(
  path: string,
  label: "home" | "dictionary",
): Promise<DirectoryIdentity> {
  let status: BigIntStats | undefined;
  try {
    status = await lstat(path, { bigint: true });
  } catch (error) {
    if (!hasCode(error, "ENOENT")) {
      const directoryLabel = label === "home" ? "home" : "dictionary";
      throw new DictionaryInstallationError(
        `Cannot inspect ${directoryLabel} directory ${path}: ${errorMessage(error)}`,
      );
    }
  }
  if (
    status === undefined ||
    status.isSymbolicLink() ||
    !status.isDirectory()
  ) {
    throw new DictionaryInstallationError(
      label === "home"
        ? `Refusing non-regular home directory: ${path}`
        : `Refusing non-regular dictionary directory: ${path}`,
    );
  }
  return { device: status.dev, inode: status.ino, path };
}

async function ensureDirectory(path: string): Promise<DirectoryIdentity> {
  try {
    await mkdir(path);
  } catch (error) {
    if (!hasCode(error, "EEXIST")) throw error;
  }
  return inspectDirectory(path, "dictionary");
}

async function assertDirectoryIdentity(
  identity: DirectoryIdentity,
): Promise<void> {
  const current = await inspectDirectory(identity.path, "dictionary");
  if (current.device !== identity.device || current.inode !== identity.inode) {
    throw new DictionaryInstallationError(
      `Refusing replaced dictionary directory: ${identity.path}`,
    );
  }
}

async function readRegularFile(path: string): Promise<Uint8Array | undefined> {
  let file: FileHandle;
  try {
    file = await open(
      path,
      constants.O_RDONLY | constants.O_NOFOLLOW | constants.O_NONBLOCK,
    );
  } catch (error) {
    if (hasCode(error, "ENOENT")) return undefined;
    throw new DictionaryInstallationError(
      `Refusing non-regular dictionary destination: ${path}`,
    );
  }
  try {
    const before = await file.stat({ bigint: true });
    if (!before.isFile()) {
      throw new DictionaryInstallationError(
        `Refusing non-regular dictionary destination: ${path}`,
      );
    }
    const content = await file.readFile();
    let after: BigIntStats | undefined;
    try {
      after = await lstat(path, { bigint: true });
    } catch (error) {
      if (!hasCode(error, "ENOENT")) {
        throw new DictionaryInstallationError(
          `Cannot inspect dictionary destination ${path}: ${errorMessage(error)}`,
        );
      }
    }
    if (
      after === undefined ||
      after.isSymbolicLink() ||
      !after.isFile() ||
      after.dev !== before.dev ||
      after.ino !== before.ino
    ) {
      throw new DictionaryInstallationError(
        `Refusing replaced dictionary destination: ${path}`,
      );
    }
    return content;
  } finally {
    await file.close();
  }
}

export async function prepareSpellingDirectory(
  home: string,
): Promise<readonly DirectoryIdentity[]> {
  const homeIdentity = await inspectDirectory(home, "home");
  const libraryIdentity = await ensureDirectory(join(home, "Library"));
  const spellingIdentity = await ensureDirectory(
    join(home, "Library", "Spelling"),
  );
  return [homeIdentity, libraryIdentity, spellingIdentity];
}

export async function assertDirectories(
  identities: readonly DirectoryIdentity[],
): Promise<void> {
  for (const identity of identities) await assertDirectoryIdentity(identity);
}

export async function existingDictionaryMatches(
  destination: string,
  expectedChecksum: string,
): Promise<boolean | undefined> {
  const content = await readRegularFile(destination);
  return content === undefined
    ? undefined
    : sha256(content) === expectedChecksum;
}

async function writeTemporary(
  temporary: string,
  content: Uint8Array,
): Promise<void> {
  const file = await open(temporary, "wx", 0o600);
  try {
    await file.writeFile(content);
    await file.sync();
    await file.chmod(0o644);
  } finally {
    await file.close();
  }
}

async function linkVerifiedTemporary(
  temporary: string,
  destination: string,
  expectedChecksum: string,
  directories: readonly DirectoryIdentity[],
): Promise<void> {
  await assertDirectories(directories);
  try {
    await link(temporary, destination);
  } catch (error) {
    if (!hasCode(error, "EEXIST")) {
      throw new DictionaryInstallationError(
        `Dictionary publication failed: ${destination}: ${errorMessage(error)}`,
      );
    }
    let concurrentMatch: boolean | undefined;
    try {
      concurrentMatch = await existingDictionaryMatches(
        destination,
        expectedChecksum,
      );
    } catch (inspectionError) {
      throw new DictionaryInstallationError(
        `Refusing to replace concurrent dictionary destination: ${destination}: ${errorMessage(inspectionError)}`,
      );
    }
    if (concurrentMatch === true) return;
    throw new DictionaryInstallationError(
      `Refusing to replace concurrent dictionary destination: ${destination}`,
    );
  }
  if (
    (await existingDictionaryMatches(destination, expectedChecksum)) !== true
  ) {
    throw new DictionaryInstallationError(
      `Dictionary publication postcondition failed: ${destination}`,
    );
  }
}

async function removeTemporary(temporary: string): Promise<void> {
  try {
    await unlink(temporary);
  } catch (error) {
    if (!hasCode(error, "ENOENT")) {
      throw new DictionaryInstallationError(
        `Cannot remove temporary dictionary ${temporary}: ${errorMessage(error)}`,
      );
    }
  }
}

export async function publishDictionary(
  destination: string,
  content: Uint8Array,
  expectedChecksum: string,
  directories: readonly DirectoryIdentity[],
): Promise<void> {
  const spellingDirectory = directories.at(-1);
  if (spellingDirectory === undefined) {
    throw new DictionaryInstallationError("Missing spelling directory");
  }
  const temporary = join(
    spellingDirectory.path,
    `.hunspell-dictionary.${randomUUID()}`,
  );
  try {
    await writeTemporary(temporary, content);
    await linkVerifiedTemporary(
      temporary,
      destination,
      expectedChecksum,
      directories,
    );
  } finally {
    await removeTemporary(temporary);
  }
}
