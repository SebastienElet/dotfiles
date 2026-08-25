import { type BigIntStats, constants } from "node:fs";
import {
  type FileHandle,
  link,
  lstat,
  mkdir,
  open,
  unlink,
} from "node:fs/promises";
import { createHash, randomUUID } from "node:crypto";
import { DictionaryInstallationError } from "./install-hunspell-dictionary-error.ts";
import { join } from "node:path";

type DirectoryIdentity = Readonly<{
  device: bigint;
  inode: bigint;
  path: string;
}>;

type DictionaryPublication = Readonly<{
  content: Readonly<ArrayLike<number>>;
  destination: string;
  expectedChecksum: string;
}>;

const privateFileMode = 0o600;
const publicFileMode = 0o644;

function hasCode(error: unknown, code: string): boolean {
  return error instanceof Error && "code" in error && error.code === code;
}

function errorMessage(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}

function sha256(content: Readonly<ArrayLike<number>>): string {
  return createHash("sha256").update(Uint8Array.from(content)).digest("hex");
}

async function inspectDirectory(
  path: string,
  label: "home" | "dictionary",
): Promise<DirectoryIdentity> {
  let status: BigIntStats | undefined = undefined;
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
    if (!hasCode(error, "EEXIST")) {
      throw error;
    }
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
  let file: FileHandle | undefined = undefined;
  try {
    file = await open(
      path,
      constants.O_RDONLY | constants.O_NOFOLLOW | constants.O_NONBLOCK,
    );
  } catch (error) {
    if (hasCode(error, "ENOENT")) {
      return undefined;
    }
    throw new DictionaryInstallationError(
      `Refusing non-regular dictionary destination: ${path}`,
    );
  }
  if (file === undefined) {
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
    await assertFileIdentity(path, before.dev, before.ino);
    return content;
  } finally {
    await file.close();
  }
}

async function assertFileIdentity(
  path: string,
  expectedDevice: bigint,
  expectedInode: bigint,
): Promise<void> {
  let after: BigIntStats | undefined = undefined;
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
    after.dev !== expectedDevice ||
    after.ino !== expectedInode
  ) {
    throw new DictionaryInstallationError(
      `Refusing replaced dictionary destination: ${path}`,
    );
  }
}

async function prepareSpellingDirectory(
  home: string,
): Promise<readonly DirectoryIdentity[]> {
  const homeIdentity = await inspectDirectory(home, "home");
  const libraryIdentity = await ensureDirectory(join(home, "Library"));
  const spellingIdentity = await ensureDirectory(
    join(home, "Library", "Spelling"),
  );
  return [homeIdentity, libraryIdentity, spellingIdentity];
}

async function assertDirectories(
  identities: readonly DirectoryIdentity[],
): Promise<void> {
  for (const identity of identities) {
    await assertDirectoryIdentity(identity);
  }
}

async function existingDictionaryMatches(
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
  content: Readonly<ArrayLike<number>>,
): Promise<void> {
  const file = await open(temporary, "wx", privateFileMode);
  try {
    await file.writeFile(Uint8Array.from(content));
    await file.sync();
    await file.chmod(publicFileMode);
  } finally {
    await file.close();
  }
}

async function linkVerifiedTemporary(
  temporary: string,
  publication: DictionaryPublication,
  directories: readonly DirectoryIdentity[],
): Promise<void> {
  const { destination, expectedChecksum } = publication;
  await assertDirectories(directories);
  try {
    await link(temporary, destination);
  } catch (error) {
    if (!hasCode(error, "EEXIST")) {
      throw new DictionaryInstallationError(
        `Dictionary publication failed: ${destination}: ${errorMessage(error)}`,
      );
    }
    let concurrentMatch: boolean | undefined = undefined;
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
    if (concurrentMatch === true) {
      return;
    }
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

async function publishDictionary(
  publication: DictionaryPublication,
  directories: readonly DirectoryIdentity[],
): Promise<void> {
  const { content } = publication;
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
    await linkVerifiedTemporary(temporary, publication, directories);
  } finally {
    await removeTemporary(temporary);
  }
}

export {
  assertDirectories,
  existingDictionaryMatches,
  prepareSpellingDirectory,
  publishDictionary,
};
