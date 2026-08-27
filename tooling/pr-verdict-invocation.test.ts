import { expect, test } from "bun:test";
import { join } from "node:path";
import { readFileSync } from "node:fs";
import { z } from "zod";

const repositoryRoot = join(import.meta.dir, "..");
const prVerdictRoot = join(repositoryRoot, "harness/skills/pr-verdict");
const explicitInvocationPattern = /^(?:\$|\/)pr-verdict(?:\s|$)/u;
const frontmatterPattern = /^---\n(?<yaml>[\s\S]*?)\n---\n/u;

const prVerdictFrontmatterSchema = z.looseObject({
  description: z.string(),
  "disable-model-invocation": z.literal(true),
});
const openAiMetadataSchema = z.looseObject({
  policy: z.object({ allow_implicit_invocation: z.literal(false) }).strict(),
});
const querySchema = z
  .object({
    query: z.string(),
    reason: z.string(),
    should_activate: z.boolean(),
  })
  .strict();
const evalsSchema = z
  .object({
    queries: z.array(querySchema).min(1),
    skill: z.literal("pr-verdict"),
    version: z.string(),
  })
  .strict();

function readYaml(path: string): unknown {
  return Bun.YAML.parse(readFileSync(path, "utf8"));
}

function readFrontmatter(path: string): unknown {
  const contents = readFileSync(path, "utf8");
  const yaml = frontmatterPattern.exec(contents)?.groups?.yaml;

  if (yaml === undefined) {
    throw new Error(`Missing YAML frontmatter in ${path}`);
  }
  return Bun.YAML.parse(yaml);
}

test("keeps pr-verdict manual-only across supported agents", () => {
  const frontmatter = prVerdictFrontmatterSchema.parse(
    readFrontmatter(join(prVerdictRoot, "SKILL.md")),
  );
  const openAiMetadata = openAiMetadataSchema.parse(
    readYaml(join(prVerdictRoot, "agents/openai.yaml")),
  );
  const evals = evalsSchema.parse(
    JSON.parse(
      readFileSync(join(prVerdictRoot, "evals/trigger-queries.json"), "utf8"),
    ),
  );
  const implicitActivations = evals.queries.filter(
    ({ query, should_activate }) =>
      should_activate && !explicitInvocationPattern.test(query),
  );
  const explicitActivations = evals.queries
    .filter(({ should_activate }) => should_activate)
    .map(({ query }) => query);
  const alwaysLoadedInstructions = ["AGENTS.md", "USER.md"]
    .map((name) => readFileSync(join(repositoryRoot, "harness", name), "utf8"))
    .join("\n");
  const prFix = readFileSync(
    join(repositoryRoot, "harness/skills/pr-fix/SKILL.md"),
    "utf8",
  );

  expect(frontmatter["disable-model-invocation"]).toBeTrue();
  expect(frontmatter.description).toContain("explicitly invokes");
  expect(openAiMetadata.policy.allow_implicit_invocation).toBeFalse();
  expect(implicitActivations).toEqual([]);
  expect(explicitActivations).toContain("$pr-verdict 1042");
  expect(explicitActivations).toContain("/pr-verdict 1042");
  expect(alwaysLoadedInstructions).not.toContain("pr-verdict");
  expect(prFix).toContain("Activate `pr-verdict`");
});

test("doctors the manual-only host metadata as one paired exception", () => {
  const skillManagerReferences = join(
    repositoryRoot,
    "harness/skills/skill-manager/references",
  );
  const conventions = readFileSync(
    join(skillManagerReferences, "conventions.md"),
    "utf8",
  );
  const doctor = readFileSync(
    join(skillManagerReferences, "doctor.md"),
    "utf8",
  );

  expect(conventions).toContain(
    "`disable-model-invocation: true` is the sole approved top-level host extension",
  );
  expect(conventions).toContain("`policy.allow_implicit_invocation: false`");
  expect(doctor).toContain("approved extension: disable-model-invocation");
  expect(doctor).toContain("true eval queries use only `$<slug>` or `/<slug>`");
});
