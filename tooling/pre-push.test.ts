import { expect, test } from "bun:test";
import type { CommandRunner } from "./pre-push.ts";
import { main } from "./pre-push.ts";
import { z } from "zod";

const objectIdLength = 40;
const objectId = "1".repeat(objectIdLength);
const previousObjectId = "2".repeat(objectIdLength);
const input = `refs/heads/feature ${objectId} refs/heads/feature ${previousObjectId}\n`;
const resultSchema = z.object({
  status: z.number().int(),
  stderr: z.string(),
  stdout: z.string(),
});

function runner(
  overrides: Readonly<
    Record<
      string,
      Readonly<{ status: number; stderr?: string; stdout?: string }>
    >
  > = {},
): Readonly<{ calls: string[]; run: CommandRunner }> {
  const calls: string[] = [];
  return {
    calls,
    run(command, arguments_) {
      const rendered = [command, ...arguments_].join(" ");
      calls.push(rendered);
      return resultSchema.parse({
        status: 0,
        stderr: "",
        stdout:
          rendered === "git rev-parse --verify HEAD" ? `${objectId}\n` : "",
        ...overrides[rendered],
      });
    },
  };
}

test("runs the static CI barriers for the checked-out branch", () => {
  const fake = runner();

  const status = main(["origin", "git@example.test:repo.git"], input, fake.run);

  expect(status).toBe(0);
  expect(fake.calls).toEqual([
    "git rev-parse --verify HEAD",
    "git status --porcelain=v1 --untracked-files=no",
    "bun --config=/dev/null --no-env-file tooling/lint-typescript.ts",
    "bun --config=/dev/null --no-env-file run typecheck",
  ]);
});

test.each([
  [
    "a raw object ID",
    `HEAD ${objectId} refs/heads/feature ${previousObjectId}\n`,
  ],
  [
    "an ancestor expression",
    `HEAD~ ${objectId} refs/heads/feature ${previousObjectId}\n`,
  ],
  [
    "a tag",
    `refs/tags/v1 ${objectId} refs/heads/feature ${previousObjectId}\n`,
  ],
])("validates %s pushed to a remote branch", (_name, updates) => {
  const fake = runner();

  expect(main(["origin", "url"], updates, fake.run)).toBe(0);
  expect(fake.calls).toContain(
    "bun --config=/dev/null --no-env-file tooling/lint-typescript.ts",
  );
});

test.each([
  ["malformed input", "invalid\n"],
  [
    "another local head",
    `refs/heads/feature ${previousObjectId} refs/heads/feature ${objectId}\n`,
  ],
])("refuses %s", (_name, updates) => {
  expect(main(["origin", "url"], updates, runner().run)).toBe(1);
});

test("refuses tracked changes before validation", () => {
  const fake = runner({
    "git status --porcelain=v1 --untracked-files=no": {
      stdout: " M file\n",
      status: 0,
    },
  });

  expect(main(["origin", "url"], input, fake.run)).toBe(1);
  expect(fake.calls).not.toContain(
    "bun --config=/dev/null --no-env-file tooling/lint-typescript.ts",
  );
});

test("propagates static validation failure", () => {
  const fake = runner({
    "bun --config=/dev/null --no-env-file tooling/lint-typescript.ts": {
      status: 1,
      stderr: "lint failed\n",
    },
  });

  expect(main(["origin", "url"], input, fake.run)).toBe(1);
  expect(fake.calls).not.toContain(
    "bun --config=/dev/null --no-env-file run typecheck",
  );
});

test.each([
  ["no updates", ""],
  [
    "a branch pushed to a tag",
    `refs/heads/feature ${objectId} refs/tags/v1 ${previousObjectId}\n`,
  ],
  [
    "a deleted remote branch",
    `(delete) ${"0".repeat(objectIdLength)} refs/heads/feature ${objectId}\n`,
  ],
])("skips %s", (_name, updates) => {
  const fake = runner();

  expect(main(["origin", "url"], updates, fake.run)).toBe(0);
  expect(fake.calls).toEqual([]);
});
