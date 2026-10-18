import { afterEach, expect, test } from "bun:test";
import { mkdir, mkdtemp, rm, symlink } from "node:fs/promises";
import { join } from "node:path";
import { tmpdir } from "node:os";

const entrypoint = join(import.meta.dir, "git-main-branch");
const roots: string[] = [];
const environment = {
  ...process.env,
  GIT_CONFIG_GLOBAL: "/dev/null",
  GIT_CONFIG_NOSYSTEM: "1",
  GIT_OPTIONAL_LOCKS: "0",
  GIT_TERMINAL_PROMPT: "0",
};

afterEach(async () => {
  await Promise.all(
    roots.splice(0).map((root) => rm(root, { recursive: true })),
  );
});

async function temporaryDirectory(): Promise<string> {
  const root = await mkdtemp(join(tmpdir(), "main branch "));
  roots.push(root);
  return root;
}

function git(directory: string, ...arguments_: readonly string[]): string {
  const result = Bun.spawnSync(["git", "-C", directory, ...arguments_], {
    env: environment,
    stderr: "pipe",
    stdout: "pipe",
  });
  if (result.exitCode !== 0) {
    throw new Error(result.stderr.toString());
  }
  return result.stdout.toString();
}

async function repository(branch: string): Promise<string> {
  const directory = await temporaryDirectory();
  git(directory, "init", "--quiet", `--initial-branch=${branch}`);
  git(
    directory,
    "-c",
    "user.name=Test",
    "-c",
    "user.email=test@example.test",
    "-c",
    "commit.gpgsign=false",
    "-c",
    "core.hooksPath=/dev/null",
    "commit",
    "--quiet",
    "--allow-empty",
    "--message=base",
  );
  return directory;
}

function resolve(
  directory: string,
  ...arguments_: readonly string[]
): readonly [number, string, string] {
  const result = Bun.spawnSync([entrypoint, ...arguments_, "-C", directory], {
    cwd: tmpdir(),
    env: environment,
    stderr: "pipe",
    stdout: "pipe",
  });
  return [
    result.exitCode,
    result.stdout.toString(),
    result.stderr.toString(),
  ] as const;
}

test("resolves a branch with Git and Fish but without Bun on PATH", async () => {
  const directory = await repository("master");
  const bin = join(await temporaryDirectory(), "bin");
  await mkdir(bin);
  for (const command of ["bash", "dirname", "fish", "git"]) {
    const executable = Bun.which(command);
    if (executable === null) {
      throw new Error(`${command} missing from test environment`);
    }
    await symlink(executable, join(bin, command));
  }
  const result = Bun.spawnSync([entrypoint, "--strict", "-C", directory], {
    env: { ...environment, PATH: bin },
    stderr: "pipe",
    stdout: "pipe",
  });
  expect([
    result.exitCode,
    result.stdout.toString(),
    result.stderr.toString(),
  ]).toEqual([0, "master\n", ""]);
});

test.each(["main", "master", "trunk"])(
  "resolves real Git %s without changing refs, index, configuration or files",
  async (branch) => {
    const directory = await repository(branch);
    await Bun.write(join(directory, "untracked"), "keep\n");
    const before = git(directory, "status", "--porcelain=v1");
    const refs = git(directory, "show-ref");
    const head = await Bun.file(join(directory, ".git/HEAD")).text();
    const config = await Bun.file(join(directory, ".git/config")).text();
    const index = await Bun.file(join(directory, ".git/index")).arrayBuffer();
    expect(resolve(directory)).toEqual([0, `${branch}\n`, ""]);
    expect(resolve(directory, "--strict")).toEqual([0, `${branch}\n`, ""]);
    expect(git(directory, "status", "--porcelain=v1")).toBe(before);
    expect(git(directory, "show-ref")).toBe(refs);
    expect(await Bun.file(join(directory, ".git/HEAD")).text()).toBe(head);
    expect(await Bun.file(join(directory, ".git/config")).text()).toBe(config);
    expect(await Bun.file(join(directory, "untracked")).text()).toBe("keep\n");
    expect(await Bun.file(join(directory, ".git/index")).arrayBuffer()).toEqual(
      index,
    );
  },
);

test("prefers main over master and trunk even at detached HEAD", async () => {
  const directory = await repository("trunk");
  git(directory, "branch", "master");
  git(directory, "branch", "main");
  git(directory, "checkout", "--quiet", "--detach");
  git(
    directory,
    "symbolic-ref",
    "refs/remotes/origin/HEAD",
    "refs/remotes/origin/trunk",
  );
  expect(resolve(directory, "--strict")).toEqual([0, "main\n", ""]);
  git(directory, "branch", "-D", "main");
  expect(resolve(directory)).toEqual([0, "master\n", ""]);
});

test("falls back to main without a candidate, including strict mode", async () => {
  const directory = await repository("develop");
  expect(resolve(directory)).toEqual([0, "main\n", ""]);
  expect(resolve(directory, "--strict")).toEqual([0, "main\n", ""]);
});

test("falls back outside Git while strict mode reports the Git error", async () => {
  const directory = await temporaryDirectory();
  expect(resolve(directory)).toEqual([0, "main\n", ""]);
  const [status, stdout, stderr] = resolve(directory, "--strict");
  expect([status, stdout]).toEqual([1, ""]);
  expect(stderr).toContain("not a git repository");
});
