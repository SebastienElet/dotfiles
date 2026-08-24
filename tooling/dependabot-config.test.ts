import { expect, test } from "bun:test";
import { readFileSync } from "node:fs";
import { join } from "node:path";
import { z } from "zod";

const rootBunUpdate = {
  "package-ecosystem": "bun",
  directory: "/",
  schedule: {
    interval: "weekly",
    day: "monday",
    time: "05:00",
    timezone: "Europe/Paris",
  },
} as const;

const rootBunUpdateSchema = z
  .object({
    "package-ecosystem": z.literal("bun"),
    directory: z.literal("/"),
    schedule: z
      .object({
        interval: z.literal("weekly"),
        day: z.literal("monday"),
        time: z.literal("05:00"),
        timezone: z.literal("Europe/Paris"),
      })
      .strict(),
  })
  .strict();

const dependabotConfigSchema = z
  .object({
    version: z.literal(2),
    updates: z.array(rootBunUpdateSchema).length(1),
  })
  .strict();

test("keeps every root Bun dependency eligible for weekly updates", () => {
  const contents = readFileSync(
    join(import.meta.dir, "../.github/dependabot.yml"),
    "utf8",
  );
  const config = dependabotConfigSchema.parse(Bun.YAML.parse(contents));

  expect(config.updates).toEqual([rootBunUpdate]);
});

test.each([
  ["a disabled pull request queue", { "open-pull-requests-limit": 0 }],
  ["an allow filter", { allow: [] }],
  ["an ignore filter", { ignore: [] }],
])("rejects %s", (_, excludedOption) => {
  expect(() =>
    rootBunUpdateSchema.parse({ ...rootBunUpdate, ...excludedOption }),
  ).toThrow();
});
