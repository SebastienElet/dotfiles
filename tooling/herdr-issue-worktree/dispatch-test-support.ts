import { mkdtempSync, realpathSync, rmSync } from "node:fs";
import type { IssueSelection } from "./selection.ts";
import { inspectRepository } from "./repository.ts";
import { join } from "node:path";
import { parseIssueClick } from "./click.ts";
import { tmpdir } from "node:os";

type IssueFixture = Readonly<{
  checkout: string;
  dispose: () => void;
  selection: IssueSelection;
}>;

function issueFixture(): IssueFixture {
  const temporaryRoot = mkdtempSync(join(tmpdir(), "herdr-issue-dispatch-"));
  const root = join(temporaryRoot, "source");
  const checkout = join(temporaryRoot, "work");
  createGitFixture(root, checkout);
  return {
    checkout: realpathSync(checkout),
    dispose: () => {
      rmSync(temporaryRoot, { force: true, recursive: true });
    },
    selection: {
      agent: "codex",
      click: parseIssueClick(
        JSON.stringify({
          clicked_url: "https://github.com/fixture/project/issues/17",
          invocation_source: "link_click",
        }),
      ),
      repository: inspectRepository(root),
    },
  };
}

function createGitFixture(root: string, checkout: string): void {
  for (const gitArguments of [
    ["init", "-q", "-b", "main", root],
    [
      "-C",
      root,
      "-c",
      "user.email=test@example.com",
      "-c",
      "user.name=Test",
      "commit",
      "-q",
      "--allow-empty",
      "-m",
      "initial",
    ],
    [
      "-C",
      root,
      "remote",
      "add",
      "origin",
      "git@github.com:fixture/project.git",
    ],
    ["-C", root, "worktree", "add", "-q", "-b", "issue-work", checkout],
  ]) {
    const result = Bun.spawnSync(["git", ...gitArguments], {
      stderr: "pipe",
      stdout: "pipe",
    });
    if (result.exitCode !== 0) {
      throw new Error(result.stderr.toString());
    }
  }
}

export { issueFixture };
export type { IssueFixture };
