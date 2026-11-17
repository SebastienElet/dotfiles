import { afterEach, expect, test } from "bun:test";
import { mkdtempSync, realpathSync, rmSync, symlinkSync } from "node:fs";
import { join } from "node:path";
import { tmpdir } from "node:os";
import { z } from "zod";

const directories: string[] = [];
const entry = join(import.meta.dir, "codex-profile.ts");
const agentExitCode = 17;
const reportSchema = z.object({
  arguments: z.array(z.string()),
  term: z.string(),
});

afterEach(() => {
  for (const directory of directories.splice(0)) {
    rmSync(directory, { recursive: true, force: true });
  }
});

function git(arguments_: readonly string[]): void {
  expect(Bun.spawnSync(["git", ...arguments_]).exitCode).toBe(0);
}

function fixture(): Readonly<{
  directory: string;
  repository: string;
  worktree: string;
}> {
  const directory = realpathSync(
    mkdtempSync(join(tmpdir(), "codex-profile-git-")),
  );
  directories.push(directory);
  symlinkSync(
    join(import.meta.dir, "codex-profile-test-command.ts"),
    join(directory, "codex"),
  );
  const repository = join(directory, "main repository");
  const worktree = join(directory, "linked worktree");
  git(["init", "--quiet", "--initial-branch=main", repository]);
  git([
    "-C",
    repository,
    "-c",
    "core.hooksPath=/dev/null",
    "-c",
    "user.name=Test",
    "-c",
    "user.email=test@example.com",
    "commit",
    "--quiet",
    "--allow-empty",
    "-m",
    "fixture",
  ]);
  git([
    "-C",
    repository,
    "-c",
    "core.hooksPath=/dev/null",
    "worktree",
    "add",
    "--quiet",
    "-b",
    "linked",
    worktree,
  ]);
  return { directory, repository, worktree };
}

test("preserves the Fish wrapper terminal and Git metadata access for an explicit worktree", () => {
  const { directory, repository, worktree } = fixture();
  for (const target of [
    ["--cd", worktree],
    [`--cd=${worktree}`],
    ["-C", worktree],
  ]) {
    const result = Bun.spawnSync(
      [
        process.execPath,
        entry,
        "gpt-6.1-sol",
        "high",
        "exec",
        ...target,
        "prompt with spaces",
      ],
      {
        cwd: directory,
        env: {
          ...process.env,
          PATH: `${directory}:${process.env.PATH ?? ""}`,
          TERM: "xterm-256color",
        },
      },
    );
    const report = reportSchema.parse(JSON.parse(result.stdout.toString()));
    expect(result.exitCode).toBe(agentExitCode);
    expect(report.term).toBe("dumb");
    expect(report.arguments).toContain(join(repository, ".git"));
    expect(report.arguments).toContain(
      join(repository, ".git/worktrees/linked-worktree"),
    );
  }
});

test("keeps native arguments when Git cannot provide metadata or a prompt contains cd", () => {
  const { directory, worktree } = fixture();
  symlinkSync(process.execPath, join(directory, "bun"));
  for (const forwarded of [
    ["exec", "--cd", join(directory, "missing"), "prompt"],
    ["exec", "--", "--cd", worktree],
    ["exec", "-C"],
  ]) {
    const result = Bun.spawnSync(
      [process.execPath, entry, "gpt-6.1-sol", "high", ...forwarded],
      {
        cwd: directory,
        env: { ...process.env, PATH: directory },
      },
    );
    const report = reportSchema.parse(JSON.parse(result.stdout.toString()));
    expect(result.exitCode).toBe(agentExitCode);
    expect(report.arguments).not.toContain("--add-dir");
    expect(report.arguments.slice(-forwarded.length)).toEqual(forwarded);
  }
});
