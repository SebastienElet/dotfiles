import { expect, test } from "bun:test";
import { readFileSync } from "node:fs";
import { join } from "node:path";
import { z } from "zod";

const dependabotConfigSchema = z.object({
  version: z.literal(2),
  updates: z
    .array(
      z
        .object({
          "package-ecosystem": z.literal("bun"),
          directory: z.literal("/"),
          schedule: z.object({
            interval: z.literal("weekly"),
            day: z.literal("monday"),
            time: z.literal("05:00"),
            timezone: z.literal("Europe/Paris"),
          }),
        })
        .passthrough(),
    )
    .length(1),
});

test("keeps every root Bun dependency eligible for weekly updates", () => {
  const contents = readFileSync(
    join(import.meta.dir, "../.github/dependabot.yml"),
    "utf8",
  );
  const config = dependabotConfigSchema.parse(Bun.YAML.parse(contents));
  const rootBunUpdate = config.updates[0];

  expect(rootBunUpdate).not.toHaveProperty("allow");
  expect(rootBunUpdate).not.toHaveProperty("ignore");
});
