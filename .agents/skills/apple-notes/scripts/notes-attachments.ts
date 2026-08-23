import { lstat, mkdtemp, readdir, rm } from "node:fs/promises";
import { basename, dirname, join, resolve } from "node:path";
import { folderSpecifier, quoteAppleScript } from "./notes-applescript.ts";
import { runAppleScript } from "./notes-process.ts";
import { publishDirectoryExclusively } from "./notes-publish.ts";

async function pathIsAbsent(path: string): Promise<boolean> {
  try {
    await lstat(path);
    return false;
  } catch (error) {
    if (error instanceof Error && "code" in error && error.code === "ENOENT")
      return true;
    throw error;
  }
}

async function createExportStaging(
  path: string,
): Promise<Readonly<{ destination: string; temporary: string }>> {
  const destination = resolve(path);
  const parentPath = dirname(destination);
  const parent = await lstat(parentPath);
  if (!parent.isDirectory() || parent.isSymbolicLink())
    throw new Error(
      `attachment destination parent is not a physical directory`,
    );
  if (!(await pathIsAbsent(destination)))
    throw new Error(`refusing existing attachment destination: ${destination}`);
  return {
    destination,
    temporary: await mkdtemp(
      join(parentPath, `.${basename(destination)}.notes-export-`),
    ),
  };
}

export async function exportAttachments(
  arguments_: readonly string[],
): Promise<void> {
  const account = quoteAppleScript(arguments_[4] ?? "iCloud");
  const source = folderSpecifier(arguments_[1]!);
  const title = quoteAppleScript(arguments_[2]!);
  const { destination, temporary } = await createExportStaging(arguments_[3]!);
  let published = false;
  try {
    runAppleScript(`tell application "Notes"
set sourceFolder to ${source} of account "${account}"
set matches to every note of sourceFolder whose name is "${title}"
if (count of matches) is not 1 then error "expected exactly one source note"
set n to item 1 of matches
repeat with i from 1 to (count of attachments of n)
set a to attachment i of n
set nm to (get name of a)
if nm is missing value then set nm to "attachment"
set exportPath to "${quoteAppleScript(temporary)}/" & i & "-" & nm
save a in file ((POSIX file exportPath) as string)
log exportPath
end repeat
end tell\n`);
    const exported = await readdir(temporary);
    if (exported.length === 0)
      throw new Error(
        `nothing was exported from "${arguments_[2]}" — do not treat its attachments as backed up`,
      );
    if (!(await pathIsAbsent(destination)))
      throw new Error(`attachment destination appeared during export`);
    publishDirectoryExclusively(temporary, destination);
    published = true;
  } finally {
    if (!published) await rm(temporary, { recursive: true, force: true });
  }
}
