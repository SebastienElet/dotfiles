import { CommandFailureError, runAppleScript as run } from "./notes-process.ts";
import {
  folderCreation,
  folderSpecifier,
  quoteAppleScript,
  replaceTitle,
} from "./notes-applescript.ts";
import {
  optionalArgument,
  optionalNonemptyArgument,
  requiredArgument,
  selectedAccount,
} from "./notes-arguments.ts";
import { exportAttachments } from "./notes-attachments.ts";

type MoveInput = Readonly<{
  destination: string;
  destinationPath: string;
  expectedBody: string | undefined;
  newTitle: string | undefined;
  source: string;
  title: string;
}>;

const accountIndex = 5;
const attachmentCountIndex = 2;
const commandIndex = 0;
const destinationIndex = 3;
const expectedMoveFactCount = 3;
const folderIndex = 1;
const genericFailureExitCode = 1;
const matchCountIndex = 1;
const noAttachments = 0;
const sharedIndex = 0;
const successfulExitCode = 0;
const titleIndex = 2;
const renamedTitleIndex = 4;
const usageExitCode = 64;
const usage = `usage:
  notes.sh folder <name> [account]
  notes.sh note <folder> <title> [account]
  notes.sh move <src/path> <title> <dst/path> [new-title] [account]
  notes.sh attachments <folder/path> <title> <dir> [account]`;
const parseMoveFacts = (
  output: string,
): Readonly<{
  attachmentCount: number;
  matchCount: number;
  shared: boolean;
}> => {
  const parts = output.split("\t");
  if (parts.length !== expectedMoveFactCount) {
    throw new Error(`unexpected AppleScript move evidence: ${output}`);
  }
  const attachments = parts.at(attachmentCountIndex) ?? "";
  const matches = parts.at(matchCountIndex) ?? "";
  const shared = parts.at(sharedIndex) ?? "";
  const attachmentCount = Number(attachments);
  const matchCount = Number(matches);
  if (
    (shared !== "true" && shared !== "false") ||
    !Number.isSafeInteger(matchCount) ||
    matchCount < successfulExitCode ||
    !Number.isSafeInteger(attachmentCount) ||
    attachmentCount < noAttachments
  ) {
    throw new Error(`unexpected AppleScript move evidence: ${output}`);
  }
  return { attachmentCount, matchCount, shared: shared === "true" };
};
const createFolder = (arguments_: readonly string[]): void => {
  const account = selectedAccount(arguments_, titleIndex);
  const name = quoteAppleScript(
    requiredArgument(arguments_, folderIndex, usage),
  );
  process.stdout.write(
    `${run(`tell application "Notes" to tell account "${account}"
if not (exists folder "${name}") then make new folder with properties {name:"${name}"}
return name of folder "${name}"
end tell\n`)}\n`,
  );
};
const createNote = (arguments_: readonly string[], body: string): void => {
  const account = selectedAccount(arguments_, destinationIndex);
  const folder = quoteAppleScript(
    requiredArgument(arguments_, folderIndex, usage),
  );
  const title = quoteAppleScript(
    requiredArgument(arguments_, titleIndex, usage),
  );
  if (body === "") {
    throw new Error("refusing empty note body");
  }
  const content = quoteAppleScript(body.replaceAll("\n", ""));
  process.stdout.write(
    `${run(`tell application "Notes" to tell account "${account}"
if not (exists folder "${folder}") then make new folder with properties {name:"${folder}"}
set n to make new note at folder "${folder}" with properties {body:"<div><h1>${title}</h1></div>" & "${content}"}
return name of n
end tell\n`)}\n`,
  );
};
const appendMoveRetitle = (mutation: string, input: MoveInput): string => {
  if (input.newTitle === undefined) {
    return mutation;
  }
  if (input.expectedBody === undefined) {
    return `${mutation}\nset name of n to "${quoteAppleScript(input.newTitle)}"`;
  }
  const rewritten = quoteAppleScript(
    replaceTitle(input.expectedBody, input.newTitle).replaceAll("\n", ""),
  );
  return `${mutation}\nset body of n to "${rewritten}"`;
};
const buildMoveMutation = (input: MoveInput): string => {
  let mutation = `if (get shared of ${input.source}) then error "source folder became shared before the move"
set sourceMatches to every note of ${input.source} whose name is "${input.title}"
if (count of sourceMatches) is not 1 then error "source note cardinality changed before the move"
set n to item 1 of sourceMatches`;
  if (input.expectedBody !== undefined) {
    mutation += `\nif (count of attachments of n) is not 0 then error "source note gained attachments before the move"
if (get body of n) is not "${quoteAppleScript(input.expectedBody)}" then error "source note changed before the move"`;
  }
  mutation += `\nset moveCompleted to false
try
${folderCreation(input.destinationPath)}
move n to ${input.destination}
set moveCompleted to true`;
  mutation = appendMoveRetitle(mutation, input);
  if (input.newTitle !== undefined) {
    mutation += "\nreturn name of n";
  }
  return `${mutation}
on error errorMessage number errorNumber
if moveCompleted then error "note moved but its post-move update failed; inspect the destination: " & errorMessage number errorNumber
error errorMessage number errorNumber
end try`;
};
const readExpectedBody = (
  input: Readonly<{
    account: string;
    attachmentCount: number;
    newTitle: string | undefined;
    source: string;
    title: string;
  }>,
): string | undefined => {
  if (input.newTitle === undefined || input.attachmentCount !== noAttachments) {
    return undefined;
  }
  const body = run(
    `tell application "Notes" to tell account "${input.account}" to get body of note "${quoteAppleScript(input.title)}" of ${input.source}\n`,
  );
  if (body === "") {
    throw new Error(
      `refusing to retitle "${input.title}": rewritten body came back empty — the note is unchanged`,
    );
  }
  return body;
};
const readMoveFacts = (
  account: string,
  source: string,
  title: string,
): ReturnType<typeof parseMoveFacts> =>
  parseMoveFacts(
    run(`tell application "Notes" to tell account "${account}"
set matches to every note of ${source} whose name is "${title}"
set matchCount to count of matches
if matchCount is not 1 then return ((get shared of ${source}) as text) & tab & matchCount & tab & 0
return ((get shared of ${source}) as text) & tab & matchCount & tab & (count of attachments of item 1 of matches)
end tell\n`),
  );
const assertMoveAllowed = (facts: ReturnType<typeof parseMoveFacts>): void => {
  if (facts.shared) {
    throw new Error(
      "refusing to move a note out of a shared folder — sharing cannot be restored by script",
    );
  }
  if (facts.matchCount !== genericFailureExitCode) {
    throw new Error(
      `expected exactly one source note, found ${facts.matchCount}`,
    );
  }
};
const writeAttachmentRenameWarning = (
  attachmentCount: number,
  newTitle: string | undefined,
): void => {
  if (newTitle !== undefined && attachmentCount > noAttachments) {
    process.stderr.write(
      `renamed via 'set name' (${attachmentCount} attachment(s) preserved); the body's first line still shows the old title\n`,
    );
  }
};
const moveNote = (arguments_: readonly string[]): void => {
  const account = selectedAccount(arguments_, accountIndex);
  const destinationPath = requiredArgument(arguments_, destinationIndex, usage);
  const newTitle = optionalNonemptyArgument(arguments_, renamedTitleIndex);
  const sourcePath = requiredArgument(arguments_, folderIndex, usage);
  const titleValue = requiredArgument(arguments_, titleIndex, usage);
  const destination = folderSpecifier(destinationPath);
  const source = folderSpecifier(sourcePath);
  const title = quoteAppleScript(titleValue);
  const facts = readMoveFacts(account, source, title);
  assertMoveAllowed(facts);
  const expectedBody = readExpectedBody({
    account,
    attachmentCount: facts.attachmentCount,
    newTitle,
    source,
    title: titleValue,
  });
  const mutation = buildMoveMutation({
    destination,
    destinationPath,
    expectedBody,
    newTitle,
    source,
    title,
  });
  const output = run(`tell application "Notes" to tell account "${account}"
${mutation}
end tell\n`);
  if (output !== "") {
    process.stdout.write(`${output}\n`);
  }
  writeAttachmentRenameWarning(facts.attachmentCount, newTitle);
};
const runSelectedCommand = async (
  arguments_: readonly string[],
  body: string,
): Promise<void> => {
  const command = optionalArgument(arguments_, commandIndex);
  if (command === "folder") {
    createFolder(arguments_);
    return;
  }
  if (command === "note") {
    createNote(arguments_, body);
    return;
  }
  if (command === "move") {
    moveNote(arguments_);
    return;
  }
  if (command === "attachments") {
    requiredArgument(arguments_, folderIndex, usage);
    requiredArgument(arguments_, titleIndex, usage);
    requiredArgument(arguments_, destinationIndex, usage);
    await exportAttachments(arguments_);
    return;
  }
  throw new Error(usage);
};
const reportFailure = (error: unknown): number => {
  if (error instanceof CommandFailureError) {
    return error.status;
  }
  let message = String(error);
  if (error instanceof Error) {
    ({ message } = error);
  }
  process.stderr.write(`${message}\n`);
  if (message === usage) {
    return usageExitCode;
  }
  return genericFailureExitCode;
};
const runNotesCommand = async (
  arguments_: readonly string[],
  body: string,
): Promise<number> => {
  try {
    await runSelectedCommand(arguments_, body);
    return successfulExitCode;
  } catch (error) {
    return reportFailure(error);
  }
};

export { runNotesCommand };
