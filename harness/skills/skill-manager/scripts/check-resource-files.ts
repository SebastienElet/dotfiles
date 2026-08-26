import {
  type PhysicalEntry,
  listSkillEntries,
} from "./physical-entry-audit.ts";
import {
  type ResourceFilePolicy,
  parseResourceFilePolicy,
} from "./resource-file-policy.ts";
import { readFile } from "node:fs/promises";
import { resolve } from "node:path";

type UnexpectedResourceFile = Readonly<{
  convention: string;
  path: string;
}>;
const failureExitCode = 1;
const invalidInvocationExitCode = 2;
const successExitCode = 0;
const argumentOffset = 2;

function joinAllowedNames(names: readonly string[]): string {
  if (names.length === 1) {
    return names[0] ?? "";
  }
  return `${names.slice(0, -1).join(", ")} and ${names.at(-1)}`;
}

function rootConvention(policy: ResourceFilePolicy): string {
  const directories = Object.keys(policy.resourceDirectories)
    .toSorted()
    .map((directory) => `${directory}/`);
  return `skill root admits only ${joinAllowedNames(policy.rootFiles)} and ${joinAllowedNames(directories)}`;
}

function closedDirectoryConvention(
  directory: string,
  files: readonly string[],
): string {
  return `${directory}/ admits only ${joinAllowedNames(files.toSorted())}`;
}

function findUnexpectedResourceFile(
  path: string,
  policy: ResourceFilePolicy,
): UnexpectedResourceFile | undefined {
  const [rootEntry, ...nestedEntries] = path.split("/");
  if (nestedEntries.length === 0 && policy.rootFiles.includes(path)) {
    return undefined;
  }
  if (rootEntry === undefined || rootEntry.length === 0) {
    return { convention: rootConvention(policy), path };
  }
  if (nestedEntries.length === 0) {
    return { convention: rootConvention(policy), path };
  }
  const directoryPolicy = Object.hasOwn(policy.resourceDirectories, rootEntry)
    ? policy.resourceDirectories[rootEntry]
    : undefined;
  if (directoryPolicy === undefined) {
    return { convention: rootConvention(policy), path };
  }
  if (directoryPolicy.mode === "open") {
    return undefined;
  }
  if (
    nestedEntries.length === 1 &&
    directoryPolicy.files.includes(nestedEntries[0] ?? "")
  ) {
    return undefined;
  }
  return {
    convention: closedDirectoryConvention(rootEntry, directoryPolicy.files),
    path,
  };
}

function findUnexpectedResourceFiles(
  paths: readonly string[],
  policy: ResourceFilePolicy,
): readonly UnexpectedResourceFile[] {
  return paths.flatMap((path) => {
    const finding = findUnexpectedResourceFile(path, policy);
    return finding === undefined ? [] : [finding];
  });
}

function findUnexpectedDirectory(
  path: string,
  policy: ResourceFilePolicy,
): UnexpectedResourceFile | undefined {
  const [rootEntry, ...nestedEntries] = path.slice(0, -1).split("/");
  const directory = rootEntry ?? "";
  const directoryPolicy = Object.hasOwn(policy.resourceDirectories, directory)
    ? policy.resourceDirectories[directory]
    : undefined;
  if (directoryPolicy === undefined) {
    return { convention: rootConvention(policy), path };
  }
  if (nestedEntries.length === 0 || directoryPolicy.mode === "open") {
    return undefined;
  }
  return {
    convention: closedDirectoryConvention(
      rootEntry ?? "",
      directoryPolicy.files,
    ),
    path,
  };
}

function findUnexpectedEntries(
  entries: readonly PhysicalEntry[],
  policy: ResourceFilePolicy,
): readonly UnexpectedResourceFile[] {
  return entries.flatMap((entry) => {
    if (entry.kind === "ignored") {
      return [
        {
          convention: "Git-ignored entries are forbidden inside skills",
          path: entry.path,
        },
      ];
    }
    if (entry.kind === "unsupported") {
      return [
        {
          convention: "skills admit only regular files and directories",
          path: entry.path,
        },
      ];
    }
    const finding =
      entry.kind === "directory"
        ? findUnexpectedDirectory(entry.path, policy)
        : findUnexpectedResourceFile(entry.path, policy);
    return finding === undefined ? [] : [finding];
  });
}

async function loadPolicy(): Promise<ResourceFilePolicy> {
  const policyPath = resolve(
    import.meta.dir,
    "../assets/resource-file-policy.json",
  );
  const serializedPolicy = new TextDecoder("utf-8", { fatal: true }).decode(
    await readFile(policyPath),
  );
  return parseResourceFilePolicy(JSON.parse(serializedPolicy));
}

async function main(): Promise<number> {
  const invocation = Bun.argv.slice(argumentOffset);
  const [skillRoot] = invocation;
  if (invocation.length !== 1 || skillRoot === undefined || skillRoot === "") {
    process.stderr.write("Usage: check-resource-files <skill-root>\n");
    return invalidInvocationExitCode;
  }
  const entries = await listSkillEntries(skillRoot);
  const findings = findUnexpectedEntries(entries, await loadPolicy());
  if (findings.length > 0) {
    for (const finding of findings) {
      process.stderr.write(
        `${finding.path}: unexpected entry; ${finding.convention}.\n`,
      );
    }
    return failureExitCode;
  }
  process.stdout.write("Resource files: PASS\n");
  return successExitCode;
}

if (import.meta.main) {
  try {
    process.exitCode = await main();
  } catch (error) {
    const message = error instanceof Error ? error.message : "Unknown failure.";
    process.stderr.write(`${message}\n`);
    process.exitCode = failureExitCode;
  }
}

export { findUnexpectedResourceFiles };
