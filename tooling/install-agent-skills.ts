import { readFileSync, statSync } from "node:fs";
import { deployLink } from "./deploy-link.ts";
import { join } from "node:path";
import { z } from "zod";

const agentSchema = z.enum(["claude", "codex", "cursor"]);
const argumentsSchema = z.tuple([
  z.string().min(1),
  z.string().min(1),
  agentSchema,
]);
const installationSchema = z
  .object({ agent: agentSchema, scope: z.enum(["user", "project"]) })
  .readonly();
const skillSchema = z
  .object({
    slug: z.string().regex(/^[a-z0-9]+(?:-[a-z0-9]+)*$/u),
    installations: z.array(installationSchema).readonly(),
  })
  .readonly();
const manifestSchema = z.looseObject({
  version: z.literal(1),
  skills: z.array(skillSchema).readonly(),
});
const skillDirectories = {
  claude: ".claude/skills",
  codex: ".agents/skills",
  cursor: ".cursor/skills",
} as const;
const argumentOffset = 2;
const failure = 1;

function installAgentSkills(
  repository: string,
  home: string,
  agent: z.infer<typeof agentSchema>,
): void {
  const manifest = manifestSchema.parse(
    Bun.YAML.parse(readFileSync(join(repository, "home/.arnes.yaml"), "utf8")),
  );
  const sources = manifest.skills
    .filter((skill) =>
      skill.installations.some(
        (installation) =>
          installation.agent === agent && installation.scope === "user",
      ),
    )
    .map((skill) => ({
      source: join(repository, "harness/skills", skill.slug),
      destination: join(home, skillDirectories[agent], skill.slug),
    }));
  for (const { source } of sources) {
    if (!statSync(join(source, "SKILL.md")).isFile()) {
      throw new Error(`Skill source is not a file: ${source}`);
    }
  }
  for (const { source, destination } of sources) {
    deployLink(source, destination);
  }
}

if (import.meta.main) {
  try {
    const [repository, home, agent] = argumentsSchema.parse(
      process.argv.slice(argumentOffset),
    );
    installAgentSkills(repository, home, agent);
  } catch (error) {
    process.stderr.write(
      `${error instanceof Error ? error.message : String(error)}\n`,
    );
    process.exitCode = failure;
  }
}
