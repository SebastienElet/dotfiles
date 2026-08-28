import { basename, dirname, join } from "node:path";
import {
  chmodSync,
  linkSync,
  lstatSync,
  mkdirSync,
  readFileSync,
  readdirSync,
  renameSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { randomUUID } from "node:crypto";

type FileVersion = Readonly<{
  content: readonly number[];
  device: number;
  inode: number;
  mode: number;
}>;
type VersionReplacement = Readonly<{
  beforePublish?: (() => void) | undefined;
  content: readonly number[];
  expected: FileVersion;
  label: string;
  mode: number;
  path: string;
  phase: string;
  publish?: ((source: string, destination: string) => void) | undefined;
}>;

function inspectFileVersion(path: string): FileVersion {
  const metadata = lstatSync(path);
  if (!metadata.isFile() || metadata.isSymbolicLink() || metadata.nlink > 1) {
    throw new Error(`configuration must remain one regular file: ${path}`);
  }
  return {
    content: [...readFileSync(path)],
    device: metadata.dev,
    inode: metadata.ino,
    mode: metadata.mode,
  };
}

function replaceFileVersion(input: VersionReplacement): FileVersion {
  mkdirSync(dirname(input.path), { recursive: true });
  const suffix = randomUUID();
  const temporary = `${input.path}.firecrawl-retirement.new.${suffix}`;
  const displaced = `${input.path}.firecrawl-retirement.old.${suffix}`;
  let removeDisplaced = false;
  try {
    writeFileSync(temporary, Uint8Array.from(input.content), {
      flag: "wx",
      mode: input.mode,
    });
    chmodSync(temporary, input.mode);
    const result = publishReplacement(input, temporary, displaced);
    removeDisplaced = true;
    return result;
  } catch (error) {
    removeDisplaced = restoreDisplacedVersion(displaced, input.path);
    throw error;
  } finally {
    rmSync(temporary, { force: true });
    if (removeDisplaced) {
      rmSync(displaced, { force: true });
    }
  }
}

function publishReplacement(
  input: VersionReplacement,
  temporary: string,
  displaced: string,
): FileVersion {
  const published = inspectFileVersion(temporary);
  renameSync(input.path, displaced);
  const actual = inspectFileVersion(displaced);
  if (!sameVersion(actual, input.expected)) {
    throw changedVersionError(input);
  }
  input.beforePublish?.();
  try {
    (input.publish ?? linkSync)(temporary, input.path);
  } catch (error) {
    if (isExistingFile(error)) {
      throw changedVersionError(input);
    }
    throw error;
  }
  return published;
}

function restoreDisplacedVersion(displaced: string, path: string): boolean {
  if (!exists(displaced)) {
    return true;
  }
  try {
    linkSync(displaced, path);
    return true;
  } catch (error) {
    if (isExistingFile(error)) {
      return true;
    }
    return false;
  }
}

function recoverInterruptedFileReplacement(path: string): void {
  const directory = dirname(path);
  if (!exists(directory)) {
    return;
  }
  const oldPrefix = `${basename(path)}.firecrawl-retirement.old.`;
  const newPrefix = `${basename(path)}.firecrawl-retirement.new.`;
  const entries = readdirSync(directory);
  const candidates = entries.filter((name) => name.startsWith(oldPrefix));
  const publications = entries.filter((name) => name.startsWith(newPrefix));
  if (exists(path)) {
    recoverPublishedReplacement({
      candidates,
      directory,
      newPrefix,
      oldPrefix,
      path,
      publications,
    });
    return;
  }
  if (candidates.length === 0) {
    return;
  }
  if (candidates.length !== 1) {
    throw new Error(`ambiguous interrupted configuration update: ${path}`);
  }
  const displaced = join(directory, candidates[0] ?? "");
  const suffix = displaced.slice(join(directory, oldPrefix).length);
  linkSync(displaced, path);
  rmSync(displaced);
  rmSync(`${path}.firecrawl-retirement.new.${suffix}`, { force: true });
}

function recoverPublishedReplacement(
  input: Readonly<{
    candidates: readonly string[];
    directory: string;
    newPrefix: string;
    oldPrefix: string;
    path: string;
    publications: readonly string[];
  }>,
): void {
  if (input.publications.length === 0) {
    return;
  }
  if (input.publications.length !== 1) {
    throw new Error(
      `ambiguous interrupted configuration update: ${input.path}`,
    );
  }
  const publication = join(input.directory, input.publications[0] ?? "");
  const suffix = publication.slice(
    join(input.directory, input.newPrefix).length,
  );
  const displaced = join(input.directory, `${input.oldPrefix}${suffix}`);
  const currentMetadata = lstatSync(input.path);
  const publicationMetadata = lstatSync(publication);
  if (
    !input.candidates.includes(`${input.oldPrefix}${suffix}`) ||
    currentMetadata.dev !== publicationMetadata.dev ||
    currentMetadata.ino !== publicationMetadata.ino
  ) {
    throw new Error(
      `ambiguous interrupted configuration update: ${input.path}`,
    );
  }
  rmSync(publication);
  rmSync(displaced);
}

function sameVersion(
  actual: Readonly<FileVersion>,
  expected: Readonly<FileVersion>,
): boolean {
  return (
    actual.device === expected.device &&
    actual.inode === expected.inode &&
    actual.mode === expected.mode &&
    Buffer.from(actual.content).equals(Buffer.from(expected.content))
  );
}

function changedVersionError(input: VersionReplacement): Error {
  return new Error(`${input.label} configuration changed ${input.phase}`);
}

function exists(path: string): boolean {
  try {
    lstatSync(path);
    return true;
  } catch (error) {
    if (isFileSystemError(error, "ENOENT")) {
      return false;
    }
    throw error;
  }
}

function isExistingFile(error: unknown): boolean {
  return isFileSystemError(error, "EEXIST");
}

function isFileSystemError(error: unknown, code: string): boolean {
  return error instanceof Error && "code" in error && error.code === code;
}

export {
  type FileVersion,
  inspectFileVersion,
  recoverInterruptedFileReplacement,
  replaceFileVersion,
};
