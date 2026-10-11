import { afterEach, expect, test } from "bun:test";
import { cp, mkdir, mkdtemp, rm, writeFile } from "node:fs/promises";
import { join } from "node:path";
import { tmpdir } from "node:os";

const source = join(import.meta.dir, "..");
const roots: string[] = [];
const tokenWordRepetitions = 4;
const token = `ghp_${"fictitious".repeat(tokenWordRepetitions)}`;
const environment = {
  ...process.env,
  CI: "true",
  GIT_CONFIG_GLOBAL: "/dev/null",
  GIT_CONFIG_NOSYSTEM: "1",
};

function run(
  root: string,
  args: readonly string[],
): Bun.SyncSubprocess<"pipe", "pipe"> {
  return Bun.spawnSync([...args], {
    cwd: root,
    env: { ...environment, MOON_WORKSPACE_ROOT: root },
  });
}

function git(root: string, ...args: readonly string[]): void {
  expect(run(root, ["git", ...args]).exitCode).toBe(0);
}

function moon(root: string, task: string): Bun.SyncSubprocess<"pipe", "pipe"> {
  return run(root, [
    "moon",
    "run",
    `repository:${task}`,
    "--upstream",
    "none",
    "--no-actions",
  ]);
}

async function repository(): Promise<string> {
  const root = await mkdtemp(join(tmpdir(), "gitleaks-fictitious-test-"));
  roots.push(root);
  await mkdir(join(root, ".moon/tasks"), { recursive: true });
  await mkdir(join(root, "tooling"));
  await cp(join(source, ".gitleaks.toml"), join(root, ".gitleaks.toml"));
  await cp(
    join(source, "tooling/pre-commit"),
    join(root, "tooling/pre-commit"),
  );
  await cp(
    join(source, ".moon/tasks/secrets.yml"),
    join(root, ".moon/tasks/secrets.yml"),
  );
  await writeFile(
    join(root, ".moon/workspace.yml"),
    'projects: {repository: "."}\n',
  );
  await writeFile(
    join(root, "moon.yml"),
    "tags: [repository]\ntasks:\n  homebrew: {command: noop}\n  dependencies: {command: noop}\n",
  );
  git(root, "init", "-q");
  git(root, "config", "user.email", "fixture@example.invalid");
  git(root, "config", "user.name", "Fictitious fixture");
  git(root, "add", ".");
  git(root, "commit", "-qm", "clean fixture");
  expect(moon(root, "secret-prepare").exitCode).toBe(0);
  return root;
}

afterEach(async () => {
  await Promise.all(
    roots.splice(0).map((root) => rm(root, { recursive: true, force: true })),
  );
});

test("the installed hook accepts a clean commit and refuses a fictitious indexed token", async () => {
  const root = await repository();
  await writeFile(join(root, "clean.txt"), "ordinary content");
  await writeFile(join(root, ".env.example"), "ordinary content");
  git(root, "add", "clean.txt", ".env.example");
  git(root, "commit", "-qm", "clean change");
  await writeFile(join(root, "token.txt"), token);
  git(root, "add", "token.txt");
  await writeFile(join(root, "token.txt"), "working tree cleaned");
  const result = run(root, ["git", "commit", "-qm", "fictitious token"]);
  expect(result.exitCode).not.toBe(0);
  expect(result.stderr.toString()).not.toContain(token);
});

test("the CI Moon task refuses a fictitious token committed then deleted", async () => {
  const root = await repository();
  expect(moon(root, "secret-scan").exitCode).toBe(0);
  await writeFile(join(root, "token.txt"), token);
  git(root, "add", "token.txt");
  git(
    root,
    "-c",
    "core.hooksPath=/dev/null",
    "commit",
    "-qm",
    "fictitious token",
  );
  git(root, "rm", "-q", "token.txt");
  git(root, "commit", "-qm", "delete fictitious token");
  const result = moon(root, "secret-scan");
  expect(result.exitCode).not.toBe(0);
  expect(result.stdout.toString() + result.stderr.toString()).not.toContain(
    token,
  );
});

test("a known fictitious AWS false positive is allowed but an adjacent token is refused", async () => {
  const root = await repository();
  const path =
    "tooling/arnes/tests/measure_hook/collection_boundaries/redaction.rs";
  await mkdir(
    join(root, "tooling/arnes/tests/measure_hook/collection_boundaries"),
    { recursive: true },
  );
  const prefix = "AKIA";
  const example = `${prefix}ABCDEFGHIJKLMNOP`;
  await writeFile(join(root, path), example);
  git(root, "add", path);
  git(root, "commit", "-qm", "known harmless fixture");
  expect(moon(root, "secret-scan").exitCode).toBe(0);
  await writeFile(join(root, path), `${example}\n${token}\n`);
  git(root, "add", path);
  expect(
    run(root, ["git", "commit", "-qm", "adjacent fictitious token"]).exitCode,
  ).not.toBe(0);
});

test.each([".ENV", "ID_RSA", "backup.P12"])(
  "native rules refuse a private text file named %s",
  async (path) => {
    const root = await repository();
    await writeFile(join(root, path), "fictitious private content");
    git(root, "add", "-f", path);
    expect(
      run(root, ["git", "commit", "-qm", "private filename"]).exitCode,
    ).not.toBe(0);
  },
);

test("preparation preserves a third-party hook", async () => {
  const root = await repository();
  const path = join(root, ".git/hooks/pre-commit");
  const content = "#!/usr/bin/env bash\nexit 0\n";
  await writeFile(path, content);
  expect(moon(root, "secret-prepare").exitCode).not.toBe(0);
  expect(await Bun.file(path).text()).toBe(content);
});
