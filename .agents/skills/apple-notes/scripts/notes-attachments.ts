import { basename, dirname, join, resolve } from "node:path";
import { folderSpecifier, quoteAppleScript } from "./notes-applescript.ts";
import { lstat, mkdtemp, readdir, rm } from "node:fs/promises";
import { publishDirectoryExclusively } from "./notes-publish.ts";
import { runAppleScript } from "./notes-process.ts";

const accountIndex = 4;
const destinationIndex = 3;
const folderIndex = 1;
const noItems = 0;
const titleIndex = 2;
const pathIsAbsent = async (candidatePath: string): Promise<boolean> => {
  try {
    await lstat(candidatePath);
    return false;
  } catch (error) {
    if (error instanceof Error && "code" in error && error.code === "ENOENT") {
      return true;
    }
    throw error;
  }
};
const createExportStaging = async (
  candidatePath: string,
): Promise<Readonly<{ destination: string; temporary: string }>> => {
  const destination = resolve(candidatePath);
  const parentPath = dirname(destination);
  const parent = await lstat(parentPath);
  if (!parent.isDirectory() || parent.isSymbolicLink()) {
    throw new Error(
      `attachment destination parent is not a physical directory`,
    );
  }
  if (!(await pathIsAbsent(destination))) {
    throw new Error(`refusing existing attachment destination: ${destination}`);
  }
  return {
    destination,
    temporary: await mkdtemp(
      join(parentPath, `.${basename(destination)}.notes-export-`),
    ),
  };
};
const exportScript = (
  input: Readonly<{
    account: string;
    source: string;
    temporary: string;
    title: string;
  }>,
): string => `tell application "Notes"
set sourceFolder to ${input.source} of account "${input.account}"
set matches to every note of sourceFolder whose name is "${input.title}"
if (count of matches) is not 1 then error "expected exactly one source note"
set n to item 1 of matches
repeat with i from 1 to (count of attachments of n)
set a to attachment i of n
set nm to (get name of a)
if nm is missing value then set nm to "attachment"
set exportPath to "${quoteAppleScript(input.temporary)}/" & i & "-" & nm
save a in file ((POSIX file exportPath) as string)
log exportPath
end repeat
end tell\n`;
const publishExport = async (
  destination: string,
  temporary: string,
  title: string,
): Promise<void> => {
  const exported = await readdir(temporary);
  if (exported.length === noItems) {
    throw new Error(
      `nothing was exported from "${title}" — do not treat its attachments as backed up`,
    );
  }
  if (!(await pathIsAbsent(destination))) {
    throw new Error(`attachment destination appeared during export`);
  }
  publishDirectoryExclusively(temporary, destination);
};
const requiredArgument = (
  arguments_: readonly string[],
  index: number,
): string => {
  const value = arguments_.at(index);
  if (typeof value !== "string") {
    throw new TypeError(`missing attachment export argument`);
  }
  return value;
};

export const exportAttachments = async (
  arguments_: readonly string[],
): Promise<void> => {
  const account = quoteAppleScript(arguments_.at(accountIndex) ?? "iCloud");
  const folder = requiredArgument(arguments_, folderIndex);
  const titleValue = requiredArgument(arguments_, titleIndex);
  const source = folderSpecifier(folder);
  const title = quoteAppleScript(titleValue);
  const { destination, temporary } = await createExportStaging(
    requiredArgument(arguments_, destinationIndex),
  );
  let published = false;
  try {
    runAppleScript(exportScript({ account, source, temporary, title }));
    await publishExport(destination, temporary, titleValue);
    published = true;
  } finally {
    if (!published) {
      await rm(temporary, { force: true, recursive: true });
    }
  }
};
