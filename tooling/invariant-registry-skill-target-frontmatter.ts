import { z } from "zod";

const localDescriptionLimit = 400;
const maxCompatibilityLength = 500;
const maxDescriptionLength = 1024;
const maxSkillNameLength = 64;
const skillCategories: ReadonlySet<string> = new Set([
  "dev",
  "support",
  "product",
  "ops",
]);
const semanticStringSchema = z.string().regex(/\S/u);
const skillNameSchema = z
  .string()
  .min(1)
  .max(maxSkillNameLength)
  .regex(/^[a-z0-9]+(?:-[a-z0-9]+)*$/u);
const metadataSchema = z
  .record(z.string(), z.string())
  .readonly()
  .refine((metadata) => skillCategories.has(metadata.category ?? ""));
const frontmatterSchema = z
  .object({
    name: skillNameSchema,
    description: semanticStringSchema.max(maxDescriptionLength),
    license: semanticStringSchema.optional(),
    compatibility: semanticStringSchema.max(maxCompatibilityLength).optional(),
    "allowed-tools": semanticStringSchema.optional(),
    metadata: metadataSchema,
    "disable-model-invocation": z.literal(true).optional(),
  })
  .strict()
  .readonly();
const frontmatterPattern = /^---\r?\n(?<yaml>[\s\S]*?)\r?\n---(?:\r?\n|$)/u;

type FrontmatterInspection = Readonly<{
  descriptionTriggerable: boolean;
  frontmatterValid: boolean;
  name: string | undefined;
}>;

const invalidFrontmatter = (): FrontmatterInspection => ({
  descriptionTriggerable: false,
  frontmatterValid: false,
  name: undefined,
});

const parseYaml = (source: string): unknown => {
  try {
    return Bun.YAML.parse(source);
  } catch {
    return undefined;
  }
};

const inspectSkillFrontmatter = (source: string): FrontmatterInspection => {
  const yaml = frontmatterPattern.exec(source)?.groups?.yaml;
  if (yaml === undefined) {
    return invalidFrontmatter();
  }
  const parsed = frontmatterSchema.safeParse(parseYaml(yaml));
  if (!parsed.success) {
    return invalidFrontmatter();
  }
  const { description, name } = parsed.data;
  const descriptionTriggerable =
    parsed.data["disable-model-invocation"] !== true &&
    description.length < localDescriptionLimit &&
    description.includes("Use when ") &&
    /Make sure to use (?:it|this skill) whenever /u.test(description);
  return { descriptionTriggerable, frontmatterValid: true, name };
};

export { inspectSkillFrontmatter };
