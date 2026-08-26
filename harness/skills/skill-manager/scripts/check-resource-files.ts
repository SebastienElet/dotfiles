import { isAbsolute, relative, resolve, sep } from "node:path";
import { readFile, realpath } from "node:fs/promises";
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
const repositoryPathsSchema = z.array(z.string().min(1)).min(1);
const repositoryRootSchema = z.string().min(1);

type ResourceFilePolicy = z.infer<typeof resourceFilePolicySchema>;
type UnexpectedResourceFile = Readonly<{
  convention: string;
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

async function runGit(
  arguments_: readonly string[],
  failureMessage: string,
): Promise<readonly number[]> {
  const git = Bun.spawn(["git", ...arguments_], {
    stderr: "inherit",
    stdout: "pipe",
  });
  const [status, output] = await Promise.all([
    git.exited,
    new Response(git.stdout).bytes(),
  ]);
  if (status !== successExitCode) {
    throw new Error(`${failureMessage} (${status}).`);
  }
  return [...output];
}

function decodeUtf8(output: readonly number[], description: string): string {
  try {
    return new TextDecoder("utf-8", { fatal: true }).decode(
      Uint8Array.from(output),
    );
  } catch {
    throw new Error(`Git returned invalid UTF-8 for ${description}.`);
  }
}

function parseRepositoryPaths(output: readonly number[]): readonly string[] {
  const serializedPaths = decodeUtf8(output, "skill files");
  if (!serializedPaths.endsWith("\0")) {
    throw new Error("Git returned an empty or malformed skill file list.");
  }
  return repositoryPathsSchema.parse(serializedPaths.slice(0, -1).split("\0"));
}

function isOutside(root: string, path: string): boolean {
  const pathFromRoot = relative(root, path);
  return (
    pathFromRoot === ".." ||
    pathFromRoot.startsWith(`..${sep}`) ||
    isAbsolute(pathFromRoot)
  );
}

async function listSkillPaths(
  requestedSkillRoot: string,
): Promise<readonly string[]> {
  const skillRoot = await realpath(requestedSkillRoot);
  const rootOutput = await runGit(
    ["-C", skillRoot, "rev-parse", "--show-toplevel"],
    "Git could not resolve the skill repository",
  );
  const repositoryRoot = await realpath(
    repositoryRootSchema.parse(
      decodeUtf8(rootOutput, "the repository root").replace(/\n$/u, ""),
    ),
  );
  if (isOutside(repositoryRoot, skillRoot)) {
    throw new Error("The skill root is outside its Git repository.");
  }
  const skillPath = relative(repositoryRoot, skillRoot);
  const repositoryOutput = await runGit(
    [
      "-C",
      repositoryRoot,
      "ls-files",
      "-z",
      "--cached",
      "--others",
      "--exclude-standard",
      "--deduplicate",
      "--",
      skillPath,
    ],
    "Git could not enumerate skill files",
  );
  const repositoryPaths = parseRepositoryPaths(repositoryOutput);
  const skillPaths = repositoryPaths.map((path) =>
    relative(skillRoot, resolve(repositoryRoot, path)).split(sep).join("/"),
  );
  if (skillPaths.some((path) => isOutside(".", path))) {
    throw new Error("Git returned a tracked path outside the skill root.");
  }
  if (!skillPaths.includes("SKILL.md")) {
    throw new Error("The skill root has no tracked SKILL.md.");
  }
  return skillPaths;
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
  const paths = await listSkillPaths(invocation.data[0]);
  const findings = findUnexpectedResourceFiles(paths, await loadPolicy());
  if (findings.length > 0) {
    for (const finding of findings) {
      process.stderr.write(
        `${finding.path}: unexpected file; ${finding.convention}.\n`,
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
