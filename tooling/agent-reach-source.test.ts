import { afterEach, expect, test } from "bun:test";
import {
  mkdir,
  mkdtemp,
  readFile,
  readdir,
  realpath,
  rm,
  writeFile,
} from "node:fs/promises";
import { fetchAgentReachSource } from "./agent-reach-source.ts";
import { join } from "node:path";
import { rejects } from "node:assert/strict";
import { tmpdir } from "node:os";
import { verifyGitSourceBytes } from "./agent-reach-integrity.ts";

const directories: string[] = [];
afterEach(async () => {
  await Promise.all(
    directories
      .splice(0)
      .map((directory) => rm(directory, { recursive: true, force: true })),
  );
});

async function git(...args: readonly string[]): Promise<string> {
  const child = Bun.spawn(["git", ...args], { stdout: "pipe", stderr: "pipe" });
  const [output, errors, status] = await Promise.all([
    new Response(child.stdout).text(),
    new Response(child.stderr).text(),
    child.exited,
  ]);
  if (status !== 0) {
    throw new Error(errors);
  }
  return output.trim();
}

async function fixture(): Promise<
  Readonly<{ home: string; repository: string; revision: string }>
> {
  const directory = await realpath(
    await mkdtemp(join(tmpdir(), "agent-reach-source-")),
  );
  directories.push(directory);
  const repository = join(directory, "repository");
  const home = join(directory, "home");
  await mkdir(repository);
  await mkdir(home);
  await git("init", "--quiet", repository);
  await writeFile(join(repository, "recipe.md"), "original");
  await git("-C", repository, "add", "recipe.md");
  await git(
    "-C",
    repository,
    "-c",
    "user.name=Fixture",
    "-c",
    "user.email=fixture@example.test",
    "commit",
    "--quiet",
    "-m",
    "fixture",
  );
  const revision = await git("-C", repository, "rev-parse", "HEAD");
  return { home, repository, revision };
}

test("fetches exactly the pinned Git source and preserves an offline replay", async () => {
  const options = await fixture();
  const path = await fetchAgentReachSource(options);
  expect(path).toStartWith(options.home);
  expect(await readFile(join(path, "recipe.md"), "utf8")).toBe("original");
  await rm(options.repository, { recursive: true });
  expect(await fetchAgentReachSource(options)).toBe(path);
});

test("refuses dirty cached sources instead of packaging their edits", async () => {
  const options = await fixture();
  const path = await fetchAgentReachSource(options);
  expect(path).toStartWith(options.home);
  await writeFile(join(path, "recipe.md"), "edited");
  await rejects(fetchAgentReachSource(options), /divergent/u);
  expect(await readFile(join(path, "recipe.md"), "utf8")).toBe("edited");
});

test("a missing revision fails without leaving an incomplete source checkout", async () => {
  const options = await fixture();
  await rejects(
    fetchAgentReachSource({
      ...options,
      revision: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
    }),
  );
  expect(
    await readdir(join(options.home, ".local/share/agent-reach/upstream")),
  ).toEqual([]);
});

test.each(["--assume-unchanged", "--skip-worktree"])(
  "refuses edited bytes hidden by %s",
  async (flag) => {
    const options = await fixture();
    const path = await fetchAgentReachSource(options);
    await git("-C", path, "update-index", flag, "recipe.md");
    await writeFile(join(path, "recipe.md"), "foreign");
    await rejects(fetchAgentReachSource(options), /divergent/u);
  },
);

test("refuses ignored extra resources and redirected Git worktrees", async () => {
  const options = await fixture();
  const path = await fetchAgentReachSource(options);
  await writeFile(join(path, ".git/info/exclude"), "foreign.md\n");
  await writeFile(join(path, "foreign.md"), "foreign");
  await rejects(fetchAgentReachSource(options), /divergent/u);
  await rm(join(path, "foreign.md"));
  await git("-C", path, "config", "core.worktree", options.repository);
  await writeFile(join(path, "recipe.md"), "foreign");
  await rejects(fetchAgentReachSource(options), /divergent/u);
});

test.each([
  "",
  "malformed\0",
  "100644 blob invalid\tfile\0",
  "100644 blob aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\t../file\0",
])("rejects an incomplete or malformed Git tree", async (tree) => {
  const options = await fixture();
  await rejects(verifyGitSourceBytes(options.repository, tree));
});
