import {
  type Stats,
  lstatSync,
  mkdtempSync,
  readlinkSync,
  renameSync,
  rmdirSync,
  symlinkSync,
} from "node:fs";
import { dirname, isAbsolute, join } from "node:path";
import { z } from "zod";

type CommandLinkFileSystem = Readonly<{
  lstat: (path: string) => Stats;
  makeTemporaryDirectory: (prefix: string) => string;
  readlink: (path: string) => string;
  rename: (source: string, destination: string) => void;
  removeDirectory: (path: string) => void;
  symlink: (target: string, path: string) => void;
}>;

const absolutePathSchema = z
  .string()
  .min(1)
  .refine(isAbsolute, "paths must be absolute");
const argumentsSchema = z.union([
  z.tuple([absolutePathSchema, absolutePathSchema]),
  z.tuple([absolutePathSchema, absolutePathSchema, absolutePathSchema]),
]);
const RETIRE_ARGUMENT_COUNT = 2;
const systemFileSystem: CommandLinkFileSystem = {
  lstat: lstatSync,
  makeTemporaryDirectory: mkdtempSync,
  readlink: readlinkSync,
  removeDirectory: rmdirSync,
  rename: renameSync,
  symlink: symlinkSync,
};

function retireCommandLink(
  expectedTarget: string,
  destination: string,
  fileSystem: CommandLinkFileSystem = systemFileSystem,
): "absent" | "preserved" | "removed" {
  const metadata = inspectDestination(destination, fileSystem);
  if (metadata === undefined) {
    return "absent";
  }
  if (!metadata.isSymbolicLink()) {
    return "preserved";
  }
  const currentTarget = fileSystem.readlink(destination);
  if (currentTarget !== expectedTarget) {
    return "preserved";
  }

  const quarantine = fileSystem.makeTemporaryDirectory(
    join(dirname(destination), ".retire-command-link-"),
  );
  const quarantined = join(quarantine, "entry");
  try {
    fileSystem.rename(destination, quarantined);
  } catch (error) {
    fileSystem.removeDirectory(quarantine);
    throw new Error(`could not quarantine ${destination}`, { cause: error });
  }

  const quarantinedMetadata = fileSystem.lstat(quarantined);
  if (
    quarantinedMetadata.isSymbolicLink() &&
    fileSystem.readlink(quarantined) === expectedTarget
  ) {
    return "removed";
  }

  restoreUnexpectedEntry({ destination, quarantined }, fileSystem);
  throw new Error(`${destination} changed during retirement and was restored`);
}

function ensureCommandLink(
  expectedTarget: string,
  destination: string,
  fileSystem: CommandLinkFileSystem = systemFileSystem,
): "created" | "current" {
  const metadata = inspectDestination(destination, fileSystem);
  if (metadata === undefined) {
    try {
      fileSystem.symlink(expectedTarget, destination);
      return "created";
    } catch (error) {
      if (hasExpectedCommandLink(expectedTarget, destination, fileSystem)) {
        return "current";
      }
      throw new Error(`could not create ${destination}`, { cause: error });
    }
  }
  if (hasExpectedCommandLink(expectedTarget, destination, fileSystem)) {
    return "current";
  }
  throw new Error(`${destination} is not the expected symbolic link`);
}

function main(arguments_: readonly string[]): void {
  try {
    const parsedArguments = argumentsSchema.parse(arguments_);
    if (parsedArguments.length === RETIRE_ARGUMENT_COUNT) {
      retireCommandLink(...parsedArguments);
      return;
    }
    const [retiredTarget, expectedTarget, destination] = parsedArguments;
    retireCommandLink(retiredTarget, destination);
    ensureCommandLink(expectedTarget, destination);
  } catch (error) {
    process.stderr.write(`retire-command-link: ${errorMessage(error)}\n`);
    process.exitCode = 1;
  }
}

function hasExpectedCommandLink(
  expectedTarget: string,
  destination: string,
  fileSystem: CommandLinkFileSystem,
): boolean {
  const metadata = inspectDestination(destination, fileSystem);
  return (
    metadata?.isSymbolicLink() === true &&
    fileSystem.readlink(destination) === expectedTarget
  );
}

function inspectDestination(
  destination: string,
  fileSystem: CommandLinkFileSystem,
): Stats | undefined {
  try {
    return fileSystem.lstat(destination);
  } catch (error) {
    if (isMissing(error)) {
      return undefined;
    }
    throw new Error(`could not inspect ${destination}`, { cause: error });
  }
}

function restoreUnexpectedEntry(
  paths: Readonly<{
    destination: string;
    quarantined: string;
  }>,
  fileSystem: CommandLinkFileSystem,
): void {
  const { destination, quarantined } = paths;
  const metadata = fileSystem.lstat(quarantined);
  if (!metadata.isSymbolicLink()) {
    throw new Error(
      `${destination} changed during retirement; recover it from ${quarantined}`,
    );
  }
  const target = fileSystem.readlink(quarantined);
  try {
    fileSystem.symlink(target, destination);
  } catch (error) {
    throw new Error(
      `${destination} changed during retirement; recover it from ${quarantined}`,
      { cause: error },
    );
  }
}

function isMissing(error: unknown): boolean {
  return error instanceof Error && "code" in error && error.code === "ENOENT";
}

function errorMessage(error: unknown): string {
  if (error instanceof z.ZodError) {
    return z.prettifyError(error);
  }
  return error instanceof Error ? error.message : String(error);
}

export {
  type CommandLinkFileSystem,
  ensureCommandLink,
  main,
  retireCommandLink,
};
