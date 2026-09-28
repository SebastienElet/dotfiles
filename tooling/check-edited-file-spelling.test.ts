import { afterEach, expect, test } from "bun:test";
import {
  chmod,
  mkdir,
  mkdtemp,
  readFile,
  realpath,
  rm,
  writeFile,
} from "node:fs/promises";
import { dirname, join } from "node:path";
import { checkEditedFileSpelling } from "./check-edited-file-spelling.ts";
import { tmpdir } from "node:os";

const temporaryRoots: string[] = [];
const defaultTimeoutMilliseconds = 10_000;
const shortTimeoutMilliseconds = 300;
const executableMode = 0o755;

const coveredFileGroups = JSON.stringify({
  fileGroups: {
    cspellFiles: {
      files: ["harness/AGENTS.md"],
      globs: ["tooling/deployment-*.ts"],
    },
  },
});

afterEach(async () => {
  await Promise.all(
    temporaryRoots
      .splice(0)
      .map((root) => rm(root, { force: true, recursive: true })),
  );
});

type Fixture = Readonly<{ bin: string; home: string; root: string }>;

async function temporaryDirectory(): Promise<string> {
  const root = await realpath(
    await mkdtemp(join(tmpdir(), "check-edited-file-spelling-")),
  );
  temporaryRoots.push(root);
  return root;
}

async function executable(
  bin: string,
  name: string,
  body: string,
): Promise<void> {
  const path = join(bin, name);
  await writeFile(path, `#!/bin/sh\n${body}\n`);
  await chmod(path, executableMode);
}

async function writeFixture(
  root: string,
  path: string,
  content: string,
): Promise<string> {
  const absolutePath = join(root, path);
  await mkdir(dirname(absolutePath), { recursive: true });
  await writeFile(absolutePath, content);
  return absolutePath;
}

async function fixture(): Promise<Fixture> {
  const base = await temporaryDirectory();
  const root = join(base, "repository");
  const bin = join(base, "bin");
  const home = join(base, "home");
  await Promise.all([mkdir(bin), mkdir(home)]);
  expect(Bun.spawnSync(["git", "init", "--quiet", root]).exitCode).toBe(0);
  await executable(
    bin,
    "moon",
    `[ "$*" = "project repository --json" ] || exit 64\nprintf '%s' '${coveredFileGroups}'`,
  );
  await executable(bin, "bun", 'shift 4; exec cspell "$@"');
  await executable(
    bin,
    "cspell",
    String.raw`printf "%s\n" "$*" >> "$HOME/cspell-calls"`,
  );
  return { bin, home, root };
}

function searchPath(context: Fixture): string {
  return `${context.bin}:${dirname(Bun.which("git") ?? "/usr/bin/git")}`;
}

function check(
  context: Fixture,
  paths: readonly string[],
  overrides: Partial<{ path: string; timeoutMilliseconds: number }> = {},
): Promise<readonly string[]> {
  const {
    path = searchPath(context),
    timeoutMilliseconds = defaultTimeoutMilliseconds,
  } = overrides;
  return checkEditedFileSpelling(paths, {
    environment: { ...process.env, HOME: context.home, PATH: path },
    repositoryCommonDirectory: join(context.root, ".git"),
    timeoutMilliseconds,
  });
}

function cspellCalls(context: Fixture): Promise<string> {
  return readFile(join(context.home, "cspell-calls"), "utf8").catch(() => "");
}

test("reports every word cspell rejects in a covered file, with its line", async () => {
  const context = await fixture();
  const path = await writeFixture(context.root, "harness/AGENTS.md", "text\n");
  await executable(
    context.bin,
    "cspell",
    String.raw`printf "%s\n" "$*" >> "$HOME/cspell-calls"
printf "harness/AGENTS.md:3:7 - Unknown word (semctx)\nharness/AGENTS.md:12:1 - Unknown word (wrld)\n"
exit 1`,
  );

  const report = await check(context, [path]);

  expect(report).toEqual([
    [
      "check-edited-file-spelling: cspell-check would reject harness/AGENTS.md; fix the spelling or add the term to home/.config/cspell/user.txt:",
      "- line 3: Unknown word (semctx)",
      "- line 12: Unknown word (wrld)",
    ].join("\n"),
  ]);
  expect(await cspellCalls(context)).toBe(
    `lint --config ${context.home}/cspell.json --no-progress --no-summary --no-color --file harness/AGENTS.md\n`,
  );
});

test("reports nothing when cspell accepts a covered file", async () => {
  const context = await fixture();
  const path = await writeFixture(context.root, "harness/AGENTS.md", "text\n");

  const report = await check(context, [path]);

  expect(report).toEqual([]);
  expect(await cspellCalls(context)).not.toBe("");
});

test("checks a file matched by a glob of the cspell file group", async () => {
  const context = await fixture();
  const path = await writeFixture(
    context.root,
    "tooling/deployment-links.ts",
    "export {};\n",
  );

  expect(await check(context, [path])).toEqual([]);
  expect(await cspellCalls(context)).toEndWith(
    "--file tooling/deployment-links.ts\n",
  );
});

test("force-checks skill Markdown as the gate does", async () => {
  const context = await fixture();
  const path = await writeFixture(
    context.root,
    "harness/skills/example/SKILL.md",
    "text\n",
  );

  expect(await check(context, [path])).toEqual([]);
  expect(await cspellCalls(context)).toEndWith(
    "--force-check --file harness/skills/example/SKILL.md\n",
  );
});

test("leaves a file the gate does not cover unchecked", async () => {
  const context = await fixture();
  const path = await writeFixture(context.root, "docs/notes.md", "wrld\n");

  expect(await check(context, [path])).toEqual([]);
  expect(await cspellCalls(context)).toBe("");
});

test("leaves a file outside the repository unchecked", async () => {
  const context = await fixture();
  const outside = await temporaryDirectory();
  const path = await writeFixture(outside, "harness/AGENTS.md", "wrld\n");

  expect(await check(context, [path])).toEqual([]);
  expect(await cspellCalls(context)).toBe("");
});

test("reports a covered file deleted before the check in one line", async () => {
  const context = await fixture();

  const report = await check(context, [
    join(context.root, "harness/AGENTS.md"),
  ]);

  expect(report).toEqual([
    "check-edited-file-spelling: harness/AGENTS.md was not checked: the file no longer exists.",
  ]);
  expect(await cspellCalls(context)).toBe("");
});

test("reports a missing cspell runner in one line", async () => {
  const context = await fixture();
  await rm(join(context.bin, "bun"));
  const path = await writeFixture(context.root, "harness/AGENTS.md", "text\n");

  expect(await check(context, [path])).toEqual([
    "check-edited-file-spelling: harness/AGENTS.md was not checked: bun is not installed.",
  ]);
});

test("reports a failing cspell in one line", async () => {
  const context = await fixture();
  await executable(
    context.bin,
    "cspell",
    'printf "Configuration Error: Failed to read config file\\n  detail\\n" >&2\nexit 1',
  );
  const path = await writeFixture(context.root, "harness/AGENTS.md", "text\n");

  expect(await check(context, [path])).toEqual([
    "check-edited-file-spelling: harness/AGENTS.md was not checked: cspell failed: Configuration Error: Failed to read config file detail.",
  ]);
});

test("reports a cspell exceeding its deadline in one line", async () => {
  const context = await fixture();
  await executable(context.bin, "cspell", "exec /bin/sleep 5");
  const path = await writeFixture(context.root, "harness/AGENTS.md", "text\n");

  expect(
    await check(context, [path], {
      timeoutMilliseconds: shortTimeoutMilliseconds,
    }),
  ).toEqual([
    `check-edited-file-spelling: harness/AGENTS.md was not checked: cspell timed out after ${shortTimeoutMilliseconds} ms.`,
  ]);
});

test("reports an unreadable cspell coverage in one line", async () => {
  const context = await fixture();
  await executable(context.bin, "moon", 'echo "no workspace" >&2\nexit 1');
  const path = await writeFixture(context.root, "harness/AGENTS.md", "text\n");

  expect(await check(context, [path])).toEqual([
    "check-edited-file-spelling: harness/AGENTS.md was not checked: moon failed: no workspace.",
  ]);
  expect(await cspellCalls(context)).toBe("");
});

test("reports a malformed cspell coverage in one line", async () => {
  const context = await fixture();
  await executable(context.bin, "moon", "printf '{}'");
  const path = await writeFixture(context.root, "harness/AGENTS.md", "text\n");

  const [report, ...others] = await check(context, [path]);

  expect(others).toEqual([]);
  expect(report).toStartWith(
    "check-edited-file-spelling: harness/AGENTS.md was not checked: moon returned an unexpected project description:",
  );
  expect(report).not.toContain("\n");
});
