import {
  folderCreation,
  folderSpecifier,
  quoteAppleScript,
  replaceTitle,
} from "./notes-applescript.ts";
import { exportAttachments } from "./notes-attachments.ts";
import { CommandFailure, runAppleScript as run } from "./notes-process.ts";

const usage = `usage:
  notes.sh folder <name> [account]
  notes.sh note <folder> <title> [account]
  notes.sh move <src/path> <title> <dst/path> [new-title] [account]
  notes.sh attachments <folder/path> <title> <dir> [account]`;

function selectedAccount(arguments_: readonly string[], index: number): string {
  return quoteAppleScript(arguments_[index] ?? "iCloud");
}

function parseMoveFacts(output: string): Readonly<{
  shared: boolean;
  matchCount: number;
  attachmentCount: number;
}> {
  const [shared, matches, attachments, extra] = output.split("\t");
  const matchCount = Number(matches);
  const attachmentCount = Number(attachments);
  if (
    (shared !== "true" && shared !== "false") ||
    extra !== undefined ||
    !Number.isSafeInteger(matchCount) ||
    matchCount < 0 ||
    !Number.isSafeInteger(attachmentCount) ||
    attachmentCount < 0
  )
    throw new Error(`unexpected AppleScript move evidence: ${output}`);
  return { shared: shared === "true", matchCount, attachmentCount };
}

function createFolder(arguments_: readonly string[]): void {
  if (!arguments_[1]) throw new Error(usage);
  const name = quoteAppleScript(arguments_[1]);
  const account = selectedAccount(arguments_, 2);
  process.stdout.write(
    `${run(`tell application "Notes" to tell account "${account}"
if not (exists folder "${name}") then make new folder with properties {name:"${name}"}
return name of folder "${name}"
end tell\n`)}\n`,
  );
}

function createNote(arguments_: readonly string[], body: string): void {
  if (!arguments_[1] || !arguments_[2]) throw new Error(usage);
  if (body === "") throw new Error("refusing empty note body");
  const account = selectedAccount(arguments_, 3);
  const folder = quoteAppleScript(arguments_[1]);
  const title = quoteAppleScript(arguments_[2]);
  const content = quoteAppleScript(body.replaceAll("\n", ""));
  process.stdout.write(
    `${run(`tell application "Notes" to tell account "${account}"
if not (exists folder "${folder}") then make new folder with properties {name:"${folder}"}
set n to make new note at folder "${folder}" with properties {body:"<div><h1>${title}</h1></div>" & "${content}"}
return name of n
end tell\n`)}\n`,
  );
}

function buildMoveMutation(
  input: Readonly<{
    source: string;
    destination: string;
    destinationPath: string;
    title: string;
    newTitle: string | undefined;
    expectedBody: string | undefined;
  }>,
): string {
  let mutation = `if (get shared of ${input.source}) then error "source folder became shared before the move"
set sourceMatches to every note of ${input.source} whose name is "${input.title}"
if (count of sourceMatches) is not 1 then error "source note cardinality changed before the move"
set n to item 1 of sourceMatches`;
  if (input.expectedBody !== undefined)
    mutation += `\nif (count of attachments of n) is not 0 then error "source note gained attachments before the move"
if (get body of n) is not "${quoteAppleScript(input.expectedBody)}" then error "source note changed before the move"`;
  mutation += `\nset moveCompleted to false
try
${folderCreation(input.destinationPath)}
move n to ${input.destination}
set moveCompleted to true`;
  if (input.newTitle && input.expectedBody === undefined)
    mutation += `\nset name of n to "${quoteAppleScript(input.newTitle)}"`;
  if (input.newTitle && input.expectedBody !== undefined) {
    const rewritten = quoteAppleScript(
      replaceTitle(input.expectedBody, input.newTitle).replaceAll("\n", ""),
    );
    mutation += `\nset body of n to "${rewritten}"`;
  }
  if (input.newTitle) mutation += "\nreturn name of n";
  return `${mutation}
on error errorMessage number errorNumber
if moveCompleted then error "note moved but its post-move update failed; inspect the destination: " & errorMessage number errorNumber
error errorMessage number errorNumber
end try`;
}

function readExpectedBody(
  newTitle: string | undefined,
  attachmentCount: number,
  account: string,
  title: string,
  source: string,
): string | undefined {
  if (!newTitle || attachmentCount !== 0) return undefined;
  const body = run(
    `tell application "Notes" to tell account "${account}" to get body of note "${quoteAppleScript(title)}" of ${source}\n`,
  );
  if (body === "")
    throw new Error(
      `refusing to retitle "${title}": rewritten body came back empty — the note is unchanged`,
    );
  return body;
}

function moveNote(arguments_: readonly string[]): void {
  if (!arguments_[1] || !arguments_[2] || !arguments_[3])
    throw new Error(usage);
  const account = selectedAccount(arguments_, 5);
  const source = folderSpecifier(arguments_[1]);
  const destination = folderSpecifier(arguments_[3]);
  const title = quoteAppleScript(arguments_[2]);
  const facts = parseMoveFacts(
    run(`tell application "Notes" to tell account "${account}"
set matches to every note of ${source} whose name is "${title}"
set matchCount to count of matches
if matchCount is not 1 then return ((get shared of ${source}) as text) & tab & matchCount & tab & 0
return ((get shared of ${source}) as text) & tab & matchCount & tab & (count of attachments of item 1 of matches)
end tell\n`),
  );
  if (facts.shared)
    throw new Error(
      "refusing to move a note out of a shared folder — sharing cannot be restored by script",
    );
  if (facts.matchCount !== 1)
    throw new Error(
      `expected exactly one source note, found ${facts.matchCount}`,
    );
  const newTitle = arguments_[4];
  const expectedBody = readExpectedBody(
    newTitle,
    facts.attachmentCount,
    account,
    arguments_[2],
    source,
  );
  const mutation = buildMoveMutation({
    source,
    destination,
    destinationPath: arguments_[3],
    title,
    newTitle,
    expectedBody,
  });
  const output = run(`tell application "Notes" to tell account "${account}"
${mutation}
end tell\n`);
  if (output !== "") process.stdout.write(`${output}\n`);
  if (newTitle && facts.attachmentCount > 0)
    process.stderr.write(
      `renamed via 'set name' (${facts.attachmentCount} attachment(s) preserved); the body's first line still shows the old title\n`,
    );
}

export async function runNotesCommand(
  arguments_: readonly string[],
  body: string,
): Promise<number> {
  try {
    if (arguments_[0] === "folder") createFolder(arguments_);
    else if (arguments_[0] === "note") createNote(arguments_, body);
    else if (arguments_[0] === "move") moveNote(arguments_);
    else if (arguments_[0] === "attachments") {
      if (!arguments_[1] || !arguments_[2] || !arguments_[3])
        throw new Error(usage);
      await exportAttachments(arguments_);
    } else throw new Error(usage);
    return 0;
  } catch (error) {
    if (error instanceof CommandFailure) return error.status;
    const message = error instanceof Error ? error.message : String(error);
    process.stderr.write(`${message}\n`);
    return message === usage ? 64 : 1;
  }
}
