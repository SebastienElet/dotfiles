import { expect, test } from "bun:test";
import { join } from "node:path";
import { readFileSync } from "node:fs";
import { z } from "zod";

const cooldownDays = 3;
const dependabotVersion = 2;
const firstUpdateIndex = 0;
const shorterCooldownDays = 2;

const rootBunUpdate = {
  cooldown: {
    "default-days": cooldownDays,
  },
  directory: "/",
  "package-ecosystem": "bun",
  schedule: {
    day: "monday",
    interval: "weekly",
    time: "05:00",
    timezone: "Europe/Paris",
  },
} as const;

const rootBunUpdateSchema = z
  .object({
    cooldown: z
      .object({
        "default-days": z.number().int().min(cooldownDays),
      })
      .strict(),
    directory: z.literal("/"),
    "package-ecosystem": z.literal("bun"),
    schedule: z
      .object({
        day: z.literal("monday"),
        interval: z.literal("weekly"),
        time: z.literal("05:00"),
        timezone: z.literal("Europe/Paris"),
      })
      .strict(),
  })
  .strict();

const dependabotConfigSchema = z
  .object({
    updates: z.array(rootBunUpdateSchema).length(firstUpdateIndex + 1),
    version: z.literal(dependabotVersion),
  })
  .strict();

test("keeps every root Bun dependency eligible for weekly updates", () => {
  const contents = readFileSync(
    join(import.meta.dir, "../.github/dependabot.yml"),
    "utf8",
  );
  const config = dependabotConfigSchema.parse(Bun.YAML.parse(contents));
  const firstUpdate = config.updates.at(firstUpdateIndex);

  if (firstUpdate === undefined) {
    throw new Error("Dependabot root update is missing");
  }
  expect(firstUpdate.cooldown["default-days"]).toBeGreaterThanOrEqual(
    cooldownDays,
  );
});

test.each([
  ["a disabled pull request queue", { "open-pull-requests-limit": 0 }],
  ["an allow filter", { allow: [] }],
  ["an ignore filter", { ignore: [] }],
  [
    "a cooldown shorter than three days",
    { cooldown: { "default-days": shorterCooldownDays } },
  ],
  [
    "a cooldown exclusion",
    { cooldown: { "default-days": cooldownDays, exclude: ["*"] } },
  ],
] as const)("rejects %s", (caseName, contractOverride) => {
  expect(caseName).not.toBeEmpty();
  expect(() =>
    rootBunUpdateSchema.parse({ ...rootBunUpdate, ...contractOverride }),
  ).toThrow();
});
