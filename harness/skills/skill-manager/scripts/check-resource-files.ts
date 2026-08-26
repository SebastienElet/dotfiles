import { readFile, readdir, realpath } from "node:fs/promises";
import { relative, resolve, sep } from "node:path";
import { z } from "zod";

const fileNameSchema = z
  .string()
  .min(1)
  .refine((value) => !value.includes("/") && !value.includes("\\"));
const closedDirectoryPolicySchema = z
  .object({
    files: z.array(fileNameSchema).min(1).readonly(),
    mode: z.literal("closed"),
  })
  .strict()
  .readonly();
const openDirectoryPolicySchema = z
  .object({ mode: z.literal("open") })
  .strict()
  .readonly();
const resourceFilePolicySchema = z
  .object({
    resourceDirectories: z
      .record(
        fileNameSchema,
        z.discriminatedUnion("mode", [
          closedDirectoryPolicySchema,
          openDirectoryPolicySchema,
        ]),
      )
      .readonly(),
    rootFiles: z.array(fileNameSchema).min(1).readonly(),
    version: z.literal(1),
  })
  .strict()
  .readonly();
type ResourceFilePolicy = z.infer<typeof resourceFilePolicySchema>;
type UnexpectedResourceFile = Readonly<{
  convention: string;
  path: string;
}>;
type PhysicalEntry = Readonly<{
  kind: "directory" | "file" | "unsupported";
  path: string;
}>;

const failureExitCode = 1;
const invalidInvocationExitCode = 2;
const successExitCode = 0;
const argumentOffset = 2;

function parseResourceFilePolicy(input: unknown): ResourceFilePolicy {
  return resourceFilePolicySchema.parse(input);
}

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
  const directoryPolicy = policy.resourceDirectories[rootEntry];
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
  const directoryPolicy = policy.resourceDirectories[rootEntry ?? ""];
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

async function listPhysicalEntries(
  skillRoot: string,
  directory: string,
): Promise<readonly PhysicalEntry[]> {
  const entries = await readdir(directory, { withFileTypes: true });
  const paths = await Promise.all(
    entries.map(async (entry: Readonly<(typeof entries)[number]>) => {
      const absolutePath = resolve(directory, entry.name);
      const path = relative(skillRoot, absolutePath).split(sep).join("/");
      if (entry.isDirectory()) {
        const nestedPaths = await listPhysicalEntries(skillRoot, absolutePath);
        const directoryEntry: PhysicalEntry = {
          kind: "directory",
          path: `${path}/`,
        };
        return nestedPaths.toSpliced(0, 0, directoryEntry);
      }
      return [{ kind: entry.isFile() ? "file" : "unsupported", path } as const];
    }),
  );
  return paths
    .flat()
    .toSorted((left, right) => left.path.localeCompare(right.path));
}

async function listSkillEntries(
  requestedSkillRoot: string,
): Promise<readonly PhysicalEntry[]> {
  const skillRoot = await realpath(requestedSkillRoot).catch(() => {
    throw new Error("The skill root could not be resolved.");
  });
  const skillEntries = await listPhysicalEntries(skillRoot, skillRoot);
  if (
    !skillEntries.some(
      (entry) => entry.kind === "file" && entry.path === "SKILL.md",
    )
  ) {
    throw new Error("The skill root has no regular SKILL.md.");
  }
  return skillEntries;
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
  const invocation = z
    .tuple([z.string().min(1)])
    .safeParse(Bun.argv.slice(argumentOffset));
  if (!invocation.success) {
    process.stderr.write("Usage: check-resource-files <skill-root>\n");
    return invalidInvocationExitCode;
  }
  const entries = await listSkillEntries(invocation.data[0]);
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

export { findUnexpectedResourceFiles, parseResourceFilePolicy };
