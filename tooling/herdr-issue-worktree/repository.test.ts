import { afterEach, expect, test } from "bun:test";
import {
  mkdirSync,
  mkdtempSync,
  readFileSync,
  realpathSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { inspectRepository } from "./repository.ts";
import { join } from "node:path";
import { tmpdir } from "node:os";

const temporaryRoots: string[] = [];
const stalledGitTestTimeoutMilliseconds = 10_000;

afterEach(() => {
  for (const root of temporaryRoots.splice(0)) {
    rmSync(root, { force: true, recursive: true });
  }
});

function git(directory: string, ...arguments_: readonly string[]): string {
  const result = Bun.spawnSync(
    [
      "git",
      "-c",
      "user.email=test@example.com",
      "-c",
      "user.name=Test",
      "-C",
      directory,
      ...arguments_,
    ],
    { stderr: "pipe", stdout: "pipe" },
  );
  expect(result.exitCode).toBe(0);
  return result.stdout.toString().trim();
}

function repository(): string {
  const root = mkdtempSync(join(tmpdir(), "herdr-issue-repository-"));
  temporaryRoots.push(root);
  git(root, "init", "-q", "-b", "main");
  git(root, "commit", "-q", "--allow-empty", "-m", "initial");
  return realpathSync(root);
}

test("reads Git identity and branch without changing the source checkout", () => {
  const root = repository();
  git(root, "remote", "add", "origin", "git@github.com:fixture/project.git");
  const head = git(root, "rev-parse", "HEAD");

  expect(inspectRepository(root)).toEqual({
    branch: "main",
    commonDirectory: join(root, ".git"),
    gitDirectory: join(root, ".git"),
    head,
    remotes: [
      {
        identity: { host: "github.com", path: "fixture/project" },
        name: "origin",
      },
    ],
    root,
  });
  expect(git(root, "status", "--porcelain")).toBe("");
  expect(git(root, "branch", "--show-current")).toBe("main");
  expect(git(root, "rev-parse", "HEAD")).toBe(head);
});

test("identifies a linked checkout through the source Git common directory", () => {
  const root = repository();
  const linked = join(root, "linked-checkout");
  git(root, "worktree", "add", "-q", "-b", "issue-work", linked);

  const context = inspectRepository(linked);

  expect(context.branch).toBe("issue-work");
  expect(context.commonDirectory).toBe(join(root, ".git"));
  expect(context.root).toBe(linked);
  expect(git(root, "branch", "--show-current")).toBe("main");
});

test("reads nested GitLab and SSH remotes without retaining embedded credentials", () => {
  const root = repository();
  git(
    root,
    "remote",
    "add",
    "origin",
    "https://fixture-secret@gitlab.example.test/team/group/project.git",
  );
  git(
    root,
    "remote",
    "add",
    "upstream",
    "ssh://git@gitlab.example.test/team/group/project.git",
  );

  expect(inspectRepository(root).remotes).toEqual([
    {
      identity: { host: "gitlab.example.test", path: "team/group/project" },
      name: "origin",
    },
    {
      identity: { host: "gitlab.example.test", path: "team/group/project" },
      name: "upstream",
    },
  ]);
});

test("keeps unknown local remotes unresolved", () => {
  const root = repository();
  git(root, "remote", "add", "origin", "/fixture/another-repository");

  expect(inspectRepository(root).remotes).toEqual([
    { identity: null, name: "origin" },
  ]);
});

test.each([
  "file://git@github.com:fixture/project.git",
  "https://user@github.com:fixture/project.git",
])("keeps misleading URI-like remotes unresolved: %s", (remote) => {
  const root = repository();
  git(root, "remote", "add", "origin", remote);

  expect(inspectRepository(root).remotes).toEqual([
    { identity: null, name: "origin" },
  ]);
});

test("retains a remote port rather than equating a different hosting endpoint", () => {
  const root = repository();
  git(
    root,
    "remote",
    "add",
    "origin",
    "https://gitlab.example.test:8443/team/project.git",
  );

  expect(inspectRepository(root).remotes).toEqual([
    {
      identity: { host: "gitlab.example.test:8443", path: "team/project" },
      name: "origin",
    },
  ]);
});

test("ignores inherited Git overrides instead of inspecting another repository", () => {
  const root = repository();
  const anotherRepository = repository();
  git(root, "remote", "add", "origin", "git@github.com:fixture/project.git");
  const result = Bun.spawnSync(
    [
      process.execPath,
      "--config=/dev/null",
      "--no-env-file",
      "--eval",
      'import { inspectRepository } from "./repository.ts"; process.stdout.write(JSON.stringify(inspectRepository(process.argv[1])));',
      root,
    ],
    {
      cwd: import.meta.dir,
      env: {
        ...process.env,
        GIT_DIR: join(anotherRepository, ".git"),
        GIT_WORK_TREE: anotherRepository,
      },
      stderr: "pipe",
      stdout: "pipe",
    },
  );

  expect(result.stderr.toString()).toBe("");
  expect(result.exitCode).toBe(0);
  expect(result.stdout.toString()).toContain(`"root":${JSON.stringify(root)}`);
  expect(result.stdout.toString()).toContain('"path":"fixture/project"');
});

test("preserves source changes and treats paths and remote URLs as data", () => {
  const root = repository();
  const nested = join(root, "$(touch marker) ' repository");
  mkdirSync(nested);
  writeFileSync(join(root, "pending.txt"), "keep this work\n");
  git(
    root,
    "remote",
    "add",
    "origin",
    "git@github.com:fixture/$(touch marker).git",
  );

  expect(inspectRepository(nested).root).toBe(root);
  expect(git(root, "status", "--porcelain")).toBe("?? pending.txt");
});

test("refuses a config that redirects the chosen working directory outside its root", () => {
  const root = repository();
  const anotherRepository = repository();
  git(root, "config", "core.worktree", anotherRepository);

  expect(() => inspectRepository(root)).toThrow(
    "Git inspection resolved outside the selected repository path",
  );
});

test("refuses a detached checkout instead of inventing a branch", () => {
  const root = repository();
  git(root, "checkout", "-q", "--detach");

  expect(() => inspectRepository(root)).toThrow("Git inspection failed");
});

test("refuses a non-repository and a relative repository path", () => {
  const root = mkdtempSync(join(tmpdir(), "herdr-issue-non-repository-"));
  temporaryRoots.push(root);

  expect(() => inspectRepository(root)).toThrow("Git inspection failed");
  expect(() => inspectRepository("relative/path")).toThrow();
});

test("refuses an unborn repository instead of inventing a base commit", () => {
  const root = mkdtempSync(join(tmpdir(), "herdr-issue-unborn-"));
  temporaryRoots.push(root);
  git(root, "init", "-q", "-b", "main");

  expect(() => inspectRepository(root)).toThrow("Git inspection failed");
});

test(
  "reports a timeout when a repository config read stalls",
  () => {
    const root = repository();
    const blockedConfig = join(root, "blocked-config");
    const fifo = Bun.spawnSync(["mkfifo", blockedConfig]);
    expect(fifo.exitCode).toBe(0);
    const configPath = join(root, ".git", "config");
    const originalConfig = readFileSync(configPath, "utf8");
    writeFileSync(
      configPath,
      `${originalConfig}\n[include]\npath = ${blockedConfig}\n`,
    );

    expect(() => inspectRepository(root)).toThrow(
      "Git inspection timed out or received SIGTERM",
    );
    expect(readFileSync(configPath, "utf8")).toContain("[include]");
  },
  stalledGitTestTimeoutMilliseconds,
);
