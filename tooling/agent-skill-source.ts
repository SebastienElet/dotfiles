import { join } from "node:path";
import { z } from "zod";

const skillSourceSchema = z
  .string()
  .min(1)
  .refine(
    (source) =>
      !source.includes("\0") &&
      source
        .split("/")
        .every((part) => part !== "" && part !== "." && part !== ".."),
    "skill source must stay within the repository",
  )
  .optional();

function skillSource(
  repository: string,
  skill: Readonly<{ slug: string; source?: string | undefined }>,
): string {
  return join(repository, skill.source ?? join("harness/skills", skill.slug));
}

export { skillSource, skillSourceSchema };
