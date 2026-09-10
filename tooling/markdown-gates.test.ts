import { afterEach, expect, test } from "bun:test";
import {
  clearGateFixtures,
  executable,
  gateFixture,
} from "./gate-test-support.ts";
import { delimiter, dirname, join } from "node:path";
import { mkdirSync, rmSync, symlinkSync, writeFileSync } from "node:fs";
import { z } from "zod";

const repository = join(import.meta.dir, "..");
const taskSchema = z.object({ command: z.string(), args: z.array(z.string()) });
const prettierTask = Bun.spawnSync(
  ["moon", "task", "repository:prettier-check", "--json"],
  { cwd: repository },
);
if (prettierTask.exitCode !== 0) {
  throw new Error(prettierTask.stderr.toString());
}
const prettier = taskSchema.parse(JSON.parse(prettierTask.stdout.toString()));

afterEach(clearGateFixtures);

function fixture(): ReturnType<typeof gateFixture> {
  const context = gateFixture();
  symlinkSync(join(repository, "tooling"), join(context.root, "tooling"));
  symlinkSync(
    join(repository, "node_modules"),
    join(context.root, "node_modules"),
  );
  executable(
    context.bin,
    "moon",
    'mkdir -p "$HOME/.config/cspell"\ncp "$GATE_FIXTURE/config-fixture" "$HOME/cspell.json"\ncp "$GATE_FIXTURE/user.txt" "$HOME/.config/cspell/user.txt"',
  );
  writeFileSync(
    join(context.root, "config-fixture"),
    JSON.stringify({
      version: "0.2",
      dictionaryDefinitions: [
        { name: "user-dictionary", path: "~/.config/cspell/user.txt" },
      ],
      dictionaries: ["user-dictionary"],
    }),
  );
  writeFileSync(join(context.root, "user.txt"), "rclone\n");
  run(context, ["git", "init", "--quiet"]);
  add(context, "known.md", "# Valid document\n");
  add(context, "harness/skills/example/SKILL.md", "# Example skill\n");
  return context;
}

function run(
  context: ReturnType<typeof gateFixture>,
  command: readonly string[],
): Bun.SyncSubprocess<"pipe", "pipe"> {
  return Bun.spawnSync([...command], {
    cwd: context.root,
    env: {
      ...process.env,
      PATH: [context.bin, dirname(process.execPath), process.env.PATH].join(
        delimiter,
      ),
      GATE_FIXTURE: context.root,
    },
  });
}

function add(
  context: ReturnType<typeof gateFixture>,
  path: string,
  contents: string,
): void {
  mkdirSync(dirname(join(context.root, path)), { recursive: true });
  writeFileSync(join(context.root, path), contents);
  const result = run(context, ["git", "add", "--", path]);
  expect(result.exitCode).toBe(0);
}

function spelling(
  context: ReturnType<typeof gateFixture>,
  files: readonly string[] = ["known.md"],
): Bun.SyncSubprocess<"pipe", "pipe"> {
  return run(context, [
    process.execPath,
    join(import.meta.dir, "check-cspell.ts"),
    ...files,
  ]);
}

test.each([
  "harness/skills/example/references/new resource.md",
  ".agents/skills/example/assets/nested/new.md",
])("both gates cover a newly indexed Markdown resource: %s", (path) => {
  const context = fixture();
  add(context, path, "# New document\n\nzzqxmisspelledword\n");
  const misspelled = spelling(context);
  expect(misspelled.exitCode).not.toBe(0);
  expect(misspelled.stdout.toString() + misspelled.stderr.toString()).toContain(
    path,
  );

  writeFileSync(join(context.root, path), "# New document\n\n-   valid text\n");
  const unformatted = run(context, [
    "bash",
    "-c",
    [prettier.command, ...prettier.args].join(" "),
  ]);
  expect(unformatted.exitCode).not.toBe(0);
  expect(
    unformatted.stdout.toString() + unformatted.stderr.toString(),
  ).toContain(path);

  writeFileSync(join(context.root, path), "# New document\n\n- valid text\n");
  expect(spelling(context).exitCode).toBe(0);
  expect(
    run(context, ["bash", "-c", [prettier.command, ...prettier.args].join(" ")])
      .exitCode,
  ).toBe(0);
});

test.each(["deleted", "moved"])(
  "reports a %s resource still expected by the index",
  (change) => {
    const context = fixture();
    const path = "harness/skills/example/references/expected.md";
    add(context, path, "# Expected document\n");
    rmSync(join(context.root, path));
    if (change === "moved") {
      add(
        context,
        "harness/skills/example/references/moved.md",
        "# Expected document\n",
      );
    }
    const result = spelling(context);
    expect(result.exitCode).not.toBe(0);
    expect(result.stdout.toString() + result.stderr.toString()).toContain(path);
    const formatted = run(context, [
      "bash",
      "-c",
      [prettier.command, ...prettier.args].join(" "),
    ]);
    expect(formatted.exitCode).not.toBe(0);
    expect(formatted.stdout.toString() + formatted.stderr.toString()).toContain(
      path,
    );
  },
);

test.each(["removed.md", "removed/*.md"])(
  "reports a missing explicit selection beside a valid file: %s",
  (path) => {
    const context = fixture();
    const result = spelling(context, ["known.md", path]);
    expect(result.exitCode).not.toBe(0);
    expect(result.stdout.toString() + result.stderr.toString()).toContain(path);
  },
);

test("does not admit untracked Markdown or new non-Markdown resources", () => {
  const context = fixture();
  writeFileSync(
    join(context.root, "harness/skills/example/runtime.md"),
    "zzqxmisspelledword\n",
  );
  add(
    context,
    "harness/skills/example/data.json",
    '{"word":"zzqxmisspelledword"}\n',
  );
  expect(spelling(context).exitCode).toBe(0);
});

test("reports indexed Markdown omitted by a native formatter exclusion", () => {
  const context = fixture();
  const path = "harness/skills/example/assets/node_modules/reference.md";
  add(context, path, "# Valid document\n\n-   valid text\n");
  const result = run(context, [
    "bash",
    "-c",
    [prettier.command, ...prettier.args].join(" "),
  ]);
  expect(result.exitCode).not.toBe(0);
  expect(result.stdout.toString() + result.stderr.toString()).toContain(path);
});
