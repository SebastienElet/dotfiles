import { deployLink, needsDeploymentLink } from "./deploy-link.ts";
import { isAbsolute, join } from "node:path";
import {
  lstatSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  renameSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { z } from "zod";

const revisionSchema = z
  .string()
  .trim()
  .regex(/^[a-f0-9]{40}$/u);
const argumentsSchema = z.tuple([
  z.string().refine(isAbsolute),
  z.string().refine(isAbsolute),
]);
const pluginSchema = z.object({
  plugin_id: z.literal("aigora.lantern"),
  plugin_root: z.string().refine(isAbsolute),
  source: z.object({
    kind: z.literal("github"),
    owner: z.literal("aigorahub"),
    repo: z.literal("herdr-lantern"),
    resolved_commit: revisionSchema,
  }),
});
const registrationSchema = z.object({
  result: z.object({ plugins: z.tuple([pluginSchema]) }),
});
const commandTimeout = 10_000;
const argumentOffset = 2;

function commandOutput(command: readonly string[]): Buffer {
  const result = Bun.spawnSync([...command], {
    stdout: "pipe",
    stderr: "pipe",
    timeout: commandTimeout,
  });
  if (result.exitCode !== 0 || result.signalCode !== undefined) {
    throw new Error(`Lantern inspection failed: ${command[0]}`);
  }
  return result.stdout;
}

function upstreamPrompt(repository: string): Buffer {
  const revision = revisionSchema.parse(
    readFileSync(join(repository, "harness/lantern-revision"), "utf8"),
  );
  const registration = registrationSchema.parse(
    JSON.parse(
      commandOutput([
        "herdr",
        "plugin",
        "list",
        "--plugin",
        "aigora.lantern",
        "--json",
      ]).toString(),
    ),
  );
  const [plugin] = registration.result.plugins;
  if (plugin.source.resolved_commit !== revision) {
    throw new Error(
      "Installed Lantern revision differs from the pinned revision",
    );
  }
  const committed = commandOutput([
    "git",
    "-C",
    plugin.plugin_root,
    "show",
    `${revision}:prompt.md`,
  ]);
  const installed = readFileSync(join(plugin.plugin_root, "prompt.md"));
  if (committed.length === 0 || !installed.equals(committed)) {
    throw new Error(
      "Installed Lantern prompt differs from its pinned Git content",
    );
  }
  return committed;
}

function generatedDirectory(repository: string): string {
  let directory = repository;
  for (const part of ["harness", "generated", "lantern"]) {
    directory = join(directory, part);
    const metadata = lstatSync(directory, { throwIfNoEntry: false });
    if (metadata !== undefined && !metadata.isDirectory()) {
      throw new Error(
        `Refusing non-directory Lantern generation path: ${directory}`,
      );
    }
    mkdirSync(directory, { recursive: true });
  }
  return directory;
}

function writePrompt(directory: string, bytes: readonly number[]): string {
  const content = Buffer.from(bytes);
  const destination = join(directory, "prompt.md");
  const metadata = lstatSync(destination, { throwIfNoEntry: false });
  if (metadata !== undefined && !metadata.isFile()) {
    throw new Error("Refusing non-file generated Lantern prompt");
  }
  if (metadata !== undefined && readFileSync(destination).equals(content)) {
    return destination;
  }
  const staging = mkdtempSync(join(directory, ".prompt-"));
  try {
    const temporary = join(staging, "prompt.md");
    writeFileSync(temporary, content);
    renameSync(temporary, destination);
  } finally {
    rmSync(staging, { recursive: true, force: true });
  }
  return destination;
}

function deployLantern(repository: string, home: string): void {
  const source = join(repository, "harness/generated/lantern/prompt.md");
  const destination = join(
    home,
    ".config/herdr/plugins/config/aigora.lantern/prompt.md",
  );
  needsDeploymentLink(source, destination);
  const content = Buffer.concat([
    upstreamPrompt(repository),
    Buffer.from("\n\n"),
    readFileSync(join(repository, "harness/lantern.md")),
  ]);
  writePrompt(generatedDirectory(repository), [...content]);
  deployLink(source, destination);
}

if (import.meta.main) {
  try {
    const [repository, home] = argumentsSchema.parse(
      process.argv.slice(argumentOffset),
    );
    deployLantern(repository, home);
  } catch (error) {
    process.stderr.write(
      `${error instanceof Error ? error.message : String(error)}\n`,
    );
    process.exitCode = 1;
  }
}
