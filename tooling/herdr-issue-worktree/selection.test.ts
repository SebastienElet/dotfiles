import { afterEach, expect, test } from "bun:test";
import { mkdtempSync, realpathSync, rmSync } from "node:fs";
import type { IssueClick } from "./click.ts";
import { join } from "node:path";
import { parseIssueClick } from "./click.ts";
import { selectIssueWork } from "./selection.ts";
import { tmpdir } from "node:os";

const temporaryRoots: string[] = [];

afterEach(() => {
  for (const root of temporaryRoots.splice(0)) {
    rmSync(root, { force: true, recursive: true });
  }
});

function repository(remote = "git@github.com:fixture/project.git"): string {
  const root = mkdtempSync(join(tmpdir(), "herdr-issue-selection-"));
  temporaryRoots.push(root);
  for (const gitArguments of [
    ["init", "-q", "-b", "main"],
    [
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
    ["remote", "add", "origin", remote],
  ]) {
    const result = Bun.spawnSync(["git", "-C", root, ...gitArguments], {
      stderr: "pipe",
      stdout: "pipe",
    });
    expect(result.exitCode).toBe(0);
  }
  return realpathSync(root);
}

function click(
  directory: string,
  url = "https://github.com/fixture/project/issues/17",
): IssueClick {
  return parseIssueClick(
    JSON.stringify({
      clicked_url: url,
      invocation_source: "link_click",
      workspace_cwd: directory,
    }),
  );
}

function answers(
  ...values: readonly (string | null)[]
): (question: string) => Promise<string | null> {
  const remaining = [...values];
  return (_question: string): Promise<string | null> =>
    Promise.resolve(remaining.shift() ?? null);
}

test("cancelling the agent choice does not require or inspect a repository", async () => {
  const issueClick = parseIssueClick(
    JSON.stringify({
      clicked_url: "https://github.com/fixture/project/issues/17",
      invocation_source: "link_click",
      workspace_cwd: "/does/not/exist",
    }),
  );

  expect(await selectIssueWork(issueClick, answers(null))).toBeNull();
});

test("confirms a matching current repository before selecting Codex", async () => {
  const root = repository();
  const selection = await selectIssueWork(click(root), answers("2", ""));

  expect(selection).toMatchObject({
    agent: "codex",
    repository: { branch: "main", commonDirectory: join(root, ".git"), root },
  });
});

test("asks for a repository when the current remote belongs to another project", async () => {
  const root = repository("git@github.com:fixture/another-project.git");

  expect(await selectIssueWork(click(root), answers("1", ""))).toBeNull();
});

test("refuses an explicitly chosen repository that does not match the issue URL", () => {
  const root = repository("git@github.com:fixture/another-project.git");

  expect(selectIssueWork(click(root), answers("1", root))).rejects.toThrow(
    "Selected repository does not match the clicked issue",
  );
});

test("requires an explicit repository for Linear before selecting Claude", async () => {
  const root = repository();
  const linear = click(
    root,
    "https://linear.app/fixture/issue/TST-482/current-title",
  );

  expect(await selectIssueWork(linear, answers("1", ""))).toBeNull();
  expect(await selectIssueWork(linear, answers("1", root))).toMatchObject({
    agent: "claude",
    click: { issue: { identity: "https://linear.app/fixture/issue/TST-482" } },
    repository: { root },
  });
});

test("cancels after agent selection without creating a checkout", async () => {
  const root = repository();

  expect(await selectIssueWork(click(root), answers("2", null))).toBeNull();
  const worktrees = Bun.spawnSync([
    "git",
    "-C",
    root,
    "worktree",
    "list",
    "--porcelain",
  ]);
  expect(worktrees.exitCode).toBe(0);
  expect(worktrees.stdout.toString().match(/^worktree /gmu)).toHaveLength(1);
});

test("rejects an unknown agent choice before inspecting an unavailable repository", () => {
  expect(
    selectIssueWork(click("/does/not/exist"), answers("other")),
  ).rejects.toThrow("Select Claude or Codex");
});

test("allows explicit selection when the current workspace path is unavailable", async () => {
  const root = repository();

  expect(
    await selectIssueWork(click("/does/not/exist"), answers("2", root)),
  ).toMatchObject({
    agent: "codex",
    repository: { root },
  });
});

test.each([
  "file://git@github.com:fixture/project.git",
  "https://user@github.com:fixture/project.git",
])(
  "never offers an implicit current repository for a misleading remote: %s",
  async (remote) => {
    const root = repository(remote);

    expect(await selectIssueWork(click(root), answers("2", ""))).toBeNull();
  },
);

test("never confirms a repository on another hosting endpoint", async () => {
  const root = repository("https://gitlab.example.test:8443/team/project.git");
  const issueClick = click(
    root,
    "https://gitlab.example.test/team/project/-/issues/17",
  );

  expect(await selectIssueWork(issueClick, answers("2", ""))).toBeNull();
});

test("confirms GitHub repository paths despite owner and project casing differences", async () => {
  const root = repository("git@github.com:Fixture/Project.git");
  const issueClick = click(
    root,
    "https://github.com/fIXTURE/pROJECT/issues/17",
  );
  expect(await selectIssueWork(issueClick, answers("2", ""))).toMatchObject({
    repository: { root },
    click: { context: issueClick.context },
  });
});

test("does not case-fold a GitLab repository path", async () => {
  const root = repository("git@gitlab.example.test:Fixture/Project.git");
  const issueClick = click(
    root,
    "https://gitlab.example.test/fixture/project/-/issues/17",
  );
  const result = await selectIssueWork(issueClick, answers("2", root)).catch(
    (error: unknown) => error,
  );
  expect(result).toMatchObject({
    message: "Selected repository does not match the clicked issue",
  });
});

test.each([
  [
    "git@GitHub.COM:Fixture/Project.git",
    "https://github.com/fixture/project/issues/17",
  ],
  [
    "git@GitLab.EXAMPLE.test:team/project.git",
    "https://gitlab.example.test/team/project/-/issues/17",
  ],
])("confirms a valid mixed-case SCP hostname: %s", async (remote, url) => {
  const root = repository(remote);
  expect(
    await selectIssueWork(click(root, url), answers("2", "")),
  ).toMatchObject({ repository: { root } });
});

test("confirms a GitLab repository at the exact custom HTTPS endpoint", async () => {
  const root = repository("https://gitlab.example.test:8443/team/project.git");
  const selected = await selectIssueWork(
    click(root, "https://gitlab.example.test:8443/team/project/-/issues/17"),
    answers("2", ""),
  );
  expect(selected).toMatchObject({ agent: "codex", repository: { root } });
});

test("refuses a GitLab repository at a different HTTPS port", async () => {
  const root = repository("https://gitlab.example.test:9443/team/project.git");
  const selected = click(
    root,
    "https://gitlab.example.test:8443/team/project/-/issues/17",
  );
  const failure = await selectIssueWork(selected, answers("2", root)).catch(
    (error: unknown) => error,
  );
  expect(failure).toMatchObject({
    message: "Selected repository does not match the clicked issue",
  });
});

test.each([
  "git@evil.invalid:fixture/project.git\ngit@github.com:fixture/project.git",
  "https://evil.invalid/fixture/project.git\nhttps://github.com/fixture/project.git",
  "https://git\t@github.com/fixture/project.git",
])("refuses a control-bearing configured remote URL: %s", async (remote) => {
  const root = repository(remote);
  const failure = await selectIssueWork(click(root), answers("2", root)).catch(
    (error: unknown) => error,
  );
  if (!(failure instanceof Error)) {
    throw new TypeError("Expected repository inspection to fail");
  }
  expect(failure.message).toContain("control characters");
});

test("supports genuinely separate configured remote URL entries", async () => {
  const root = repository("git@mirror.invalid:fixture/project.git");
  const configured = Bun.spawnSync([
    "git",
    "-C",
    root,
    "config",
    "--add",
    "remote.origin.url",
    "git@github.com:fixture/project.git",
  ]);
  expect(configured.exitCode).toBe(0);
  expect(await selectIssueWork(click(root), answers("2", ""))).toMatchObject({
    repository: { root },
  });
});

test("preserves Git insteadOf expansion when inspecting a remote", async () => {
  const root = repository("herdr-fixture-alias:project.git");
  const configured = Bun.spawnSync([
    "git",
    "-C",
    root,
    "config",
    "url.git@github.com:fixture/.insteadOf",
    "herdr-fixture-alias:",
  ]);
  expect(configured.exitCode).toBe(0);
  expect(await selectIssueWork(click(root), answers("2", ""))).toMatchObject({
    repository: { root },
  });
});
