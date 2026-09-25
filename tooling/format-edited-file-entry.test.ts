import { afterEach, expect, test } from "bun:test";
import { join, resolve } from "node:path";
import { mkdtemp, readFile, realpath, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { z } from "zod";

const repositoryRoot = resolve(import.meta.dir, "..");
const entryPoint = join(repositoryRoot, "tooling/format-edited-file");
const cleanups: (() => Promise<void>)[] = [];

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

function run(stdin: string): { exitCode: number; stdout: string } {
  const result = Bun.spawnSync([entryPoint], {
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

  const result = run(claudeEdit(worktree, path));

  expect(result.exitCode).toBe(0);
  expect(await readFile(path, "utf8")).toBe("- item\n");
  expect(
    hookOutputSchema.parse(JSON.parse(result.stdout)).hookSpecificOutput
      .additionalContext,
  ).toBe(
    "format-edited-file: notes.md was reformatted with prettier; read it again before editing it.",
  );
});

test("reformats a file named by a Codex patch", async () => {
  const worktree = await emptyWorktree();
  const path = join(worktree, "notes.md");
  await writeFile(path, "*  item\n");

  const result = run(
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

  expect(result.exitCode).toBe(0);
  expect(await readFile(path, "utf8")).toBe("- item\n");
});

test("leaves a Git-ignored file of this repository untouched", async () => {
  const directory = await temporaryDirectory(
    join(repositoryRoot, "node_modules"),
  );
  const path = join(directory, "notes.md");
  await writeFile(path, "*  item\n");

  const result = run(claudeEdit(repositoryRoot, path));

  expect(result).toEqual({ exitCode: 0, stdout: "" });
  expect(await readFile(path, "utf8")).toBe("*  item\n");
});

test("leaves a file outside this repository untouched", async () => {
  const directory = await temporaryDirectory(await realpath(tmpdir()));
  const path = join(directory, "notes.md");
  await writeFile(path, "*  item\n");

  const result = run(claudeEdit(directory, path));

  expect(result).toEqual({ exitCode: 0, stdout: "" });
  expect(await readFile(path, "utf8")).toBe("*  item\n");
});

test("reports unreadable hook input in one line without failing", () => {
  const result = run("not json");

  expect(result.exitCode).toBe(0);
  const context = hookOutputSchema.parse(JSON.parse(result.stdout))
    .hookSpecificOutput.additionalContext;
  expect(context).toStartWith("format-edited-file: unreadable hook input:");
  expect(context).not.toContain("\n");
});
