import { delimiter, isAbsolute, join, relative, sep } from "node:path";
import {
  lstatSync,
  mkdirSync,
  mkdtempSync,
  realpathSync,
  renameSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { z } from "zod";

const argumentsSchema = z.tuple([z.string().min(1)]);
const environmentSchema = z.object({
  PATH: z.string().optional(),
  HOMEBREW_PREFIX: z
    .string()
    .refine(isAbsolute, "HOMEBREW_PREFIX must be absolute")
    .optional(),
});
const frontmatterSchema = z.object({
  name: z.literal("herdr"),
  description: z.string().trim().min(1),
});
const skillTimeoutMilliseconds = 10_000;
const argumentOffset = 2;

function findInstalledHerdr(): string | null {
  const environment = environmentSchema.parse(process.env);
  const homebrewBins =
    environment.HOMEBREW_PREFIX === undefined
      ? ["/opt/homebrew/bin", "/usr/local/bin"]
      : [join(environment.HOMEBREW_PREFIX, "bin")];
  return Bun.which("herdr", {
    PATH: [environment.PATH ?? "", ...homebrewBins].join(delimiter),
  });
}

function prepareHerdrSkillDirectory(repository: string): string {
  const root = realpathSync(repository);
  let directory = root;
  for (const segment of ["harness", "generated", "skills", "herdr"]) {
    const next = join(directory, segment);
    mkdirSync(next, { recursive: true });
    const resolved = realpathSync(next);
    const relativePath = relative(root, resolved);
    if (
      relativePath === ".." ||
      relativePath.startsWith(`..${sep}`) ||
      isAbsolute(relativePath)
    ) {
      throw new Error(
        `herdr skill source must stay within the repository: ${next}`,
      );
    }
    directory = resolved;
  }
  return directory;
}

function generateHerdrSkill(repository: string): void {
  const herdr = findInstalledHerdr();
  if (herdr === null) {
    throw new Error("herdr not found, unable to generate its skill");
  }
  const result = Bun.spawnSync([herdr, "--skill"], {
    stdout: "pipe",
    stderr: "pipe",
    timeout: skillTimeoutMilliseconds,
  });
  if (result.exitCode !== 0 || result.signalCode !== undefined) {
    throw new Error(
      `herdr --skill failed (${result.signalCode ?? result.exitCode}): ${result.stderr.toString().trim()}`,
    );
  }
  const content = new TextDecoder("utf-8", {
    fatal: true,
    ignoreBOM: true,
  }).decode(result.stdout);
  const frontmatter =
    /^---\r?\n(?<frontmatter>[\s\S]*?)\r?\n---(?:\r?\n|$)/u.exec(content)
      ?.groups?.frontmatter;
  if (frontmatter === undefined) {
    throw new Error(
      "herdr --skill: expected YAML frontmatter with name: herdr and description",
    );
  }
  frontmatterSchema.parse(Bun.YAML.parse(frontmatter));
  const directory = prepareHerdrSkillDirectory(repository);
  if (
    lstatSync(join(directory, "SKILL.md"), {
      throwIfNoEntry: false,
    })?.isSymbolicLink() === true
  ) {
    throw new Error("refusing symbolic generated herdr SKILL.md");
  }
  const staging = mkdtempSync(join(directory, ".skill-"));
  try {
    const staged = join(staging, "SKILL.md");
    writeFileSync(staged, result.stdout);
    renameSync(staged, join(directory, "SKILL.md"));
  } finally {
    rmSync(staging, { recursive: true, force: true });
  }
}

if (import.meta.main) {
  try {
    const [repository] = argumentsSchema.parse(
      process.argv.slice(argumentOffset),
    );
    generateHerdrSkill(repository);
  } catch (error) {
    process.stderr.write(
      `${error instanceof Error ? error.message : String(error)}\n`,
    );
    process.exitCode = 1;
  }
}

export { generateHerdrSkill };
