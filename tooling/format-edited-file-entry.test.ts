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
import { join, resolve } from "node:path";
import { tmpdir } from "node:os";
import { z } from "zod";

const repositoryRoot = resolve(import.meta.dir, "..");
const entryPoint = join(repositoryRoot, "tooling/format-edited-file");
const cleanups: (() => Promise<void>)[] = [];
const executableMode = 0o755;

afterEach(async () => {
  for (const cleanup of cleanups.splice(0).toReversed()) {
    await cleanup();
  }
});

const hookOutputSchema = z.object({
  hookSpecificOutput: z.object({
    additionalContext: z.string(),
    hookEventName: z.literal("PostToolUse"),
  }),
});

function git(arguments_: readonly string[]): void {
  const result = Bun.spawnSync(["git", "-C", repositoryRoot, ...arguments_], {
    stderr: "pipe",
  });
  expect(result.exitCode, result.stderr.toString()).toBe(0);
}

async function emptyWorktree(): Promise<string> {
  const parent = await realpath(
    await mkdtemp(join(tmpdir(), "format-edited-file-worktree-")),
  );
  const worktree = join(parent, "checkout");
  git(["worktree", "add", "--quiet", "--no-checkout", "--detach", worktree]);
  cleanups.push(async () => {
    git(["worktree", "remove", "--force", worktree]);
    await rm(parent, { force: true, recursive: true });
  });
  return worktree;
}

async function temporaryDirectory(parent: string): Promise<string> {
  const directory = await realpath(
    await mkdtemp(join(parent, "format-edited-file-")),
  );
  cleanups.push(() => rm(directory, { force: true, recursive: true }));
  return directory;
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

async function spellingFakes(): Promise<string> {
  const base = await temporaryDirectory(await realpath(tmpdir()));
  const bin = join(base, "bin");
  await mkdir(bin);
  await executable(
    bin,
    "moon",
    `printf '%s' '${JSON.stringify({ fileGroups: { cspellFiles: { files: ["notes.md"], globs: [] } } })}'`,
  );
  await executable(bin, "bun", 'shift 4; exec cspell "$@"');
  await executable(
    bin,
    "cspell",
    String.raw`for argument; do file=$argument; done
cat "$file" > "$HOME/spelled"
if [ -f "$HOME/cspell-output" ]; then cat "$HOME/cspell-output"; exit 1; fi`,
  );
  return base;
}

async function run(
  stdin: string,
  home?: string,
): Promise<{ exitCode: number; stdout: string }> {
  const fakeHome = home ?? (await spellingFakes());
  const result = Bun.spawnSync([process.execPath, entryPoint], {
    env: {
      ...process.env,
      HOME: fakeHome,
      PATH: `${join(fakeHome, "bin")}:${process.env.PATH ?? ""}`,
    },
    stderr: "pipe",
    stdin: new TextEncoder().encode(stdin),
  });
  return { exitCode: result.exitCode, stdout: result.stdout.toString() };
}

function claudeEdit(cwd: string, filePath: string): string {
  return JSON.stringify({
    cwd,
    hook_event_name: "PostToolUse",
    tool_input: { file_path: filePath, new_string: "", old_string: "" },
    tool_name: "Edit",
  });
}

test("reformats a file edited in a worktree of this repository", async () => {
  const worktree = await emptyWorktree();
  const path = join(worktree, "notes.md");
  await writeFile(path, "*  item\n");

  const { exitCode, stdout } = await run(claudeEdit(worktree, path));

  expect(exitCode).toBe(0);
  expect(await readFile(path, "utf8")).toBe("- item\n");
  expect(
    hookOutputSchema.parse(JSON.parse(stdout)).hookSpecificOutput
      .additionalContext,
  ).toBe(
    "format-edited-file: notes.md was reformatted with prettier; read it again before editing it.",
  );
});

test("reformats a file named by a Codex patch", async () => {
  const worktree = await emptyWorktree();
  const path = join(worktree, "notes.md");
  await writeFile(path, "*  item\n");

  const { exitCode } = await run(
    JSON.stringify({
      cwd: worktree,
      hook_event_name: "PostToolUse",
      tool_input: {
        command:
          "*** Begin Patch\n*** Add File: notes.md\n+*  item\n*** End Patch",
      },
      tool_name: "apply_patch",
    }),
  );

  expect(exitCode).toBe(0);
  expect(await readFile(path, "utf8")).toBe("- item\n");
});

test("leaves a Git-ignored file of this repository untouched", async () => {
  const directory = await temporaryDirectory(
    join(repositoryRoot, "node_modules"),
  );
  const path = join(directory, "notes.md");
  await writeFile(path, "*  item\n");

  const { exitCode, stdout } = await run(claudeEdit(repositoryRoot, path));

  expect({ exitCode, stdout }).toEqual({ exitCode: 0, stdout: "" });
  expect(await readFile(path, "utf8")).toBe("*  item\n");
});

test("leaves a file outside this repository untouched", async () => {
  const directory = await temporaryDirectory(await realpath(tmpdir()));
  const path = join(directory, "notes.md");
  await writeFile(path, "*  item\n");

  const { exitCode, stdout } = await run(claudeEdit(directory, path));

  expect({ exitCode, stdout }).toEqual({ exitCode: 0, stdout: "" });
  expect(await readFile(path, "utf8")).toBe("*  item\n");
});

test("reports unreadable hook input in one line without failing", async () => {
  const { exitCode, stdout } = await run("not json");

  expect(exitCode).toBe(0);
  const context = hookOutputSchema.parse(JSON.parse(stdout)).hookSpecificOutput
    .additionalContext;
  expect(context).toStartWith("format-edited-file: unreadable hook input:");
  expect(context).not.toContain("\n");
});

test("checks the spelling of the reformatted content and reports both", async () => {
  const worktree = await emptyWorktree();
  const path = join(worktree, "notes.md");
  await writeFile(path, "*  wrld\n");
  const home = await spellingFakes();
  await writeFile(
    join(home, "cspell-output"),
    "notes.md:1:3 - Unknown word (wrld)\n",
  );

  const { exitCode, stdout } = await run(claudeEdit(worktree, path), home);

  expect(exitCode).toBe(0);
  expect(await readFile(join(home, "spelled"), "utf8")).toBe("- wrld\n");
  expect(
    hookOutputSchema.parse(JSON.parse(stdout)).hookSpecificOutput
      .additionalContext,
  ).toBe(
    [
      "format-edited-file: notes.md was reformatted with prettier; read it again before editing it.",
      "check-edited-file-spelling: cspell-check would reject notes.md; fix the spelling or add the term to home/.config/cspell/user.txt:",
      "- line 1: Unknown word (wrld)",
    ].join("\n"),
  );
});
