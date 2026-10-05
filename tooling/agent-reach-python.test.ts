import { afterEach, expect, test } from "bun:test";
import { mkdir, mkdtemp, realpath, rm, symlink } from "node:fs/promises";
import { join } from "node:path";
import { tmpdir } from "node:os";

const directories: string[] = [];
const script = join(
  import.meta.dir,
  "../harness/plugins/agent-reach/agent-reach-python",
);
afterEach(async () => {
  await Promise.all(
    directories
      .splice(0)
      .map((directory) => rm(directory, { recursive: true, force: true })),
  );
});

test("forwards arguments to the isolated interpreter even when HOME contains spaces", async () => {
  const directory = await realpath(
    await mkdtemp(join(tmpdir(), "agent-reach-python-")),
  );
  directories.push(directory);
  const home = join(directory, "home with spaces");
  const environment = join(
    home,
    ".local/share/agent-reach/python-tools/agent-reach/bin",
  );
  await mkdir(environment, { recursive: true });
  await symlink("/usr/bin/printf", join(environment, "python"));
  const child = Bun.spawn(
    ["bash", script, String.raw`%s\n`, "first", "two words"],
    {
      stdout: "pipe",
      stderr: "pipe",
      env: { ...process.env, HOME: home },
    },
  );
  const [output, status] = await Promise.all([
    new Response(child.stdout).text(),
    child.exited,
  ]);
  expect(status).toBe(0);
  expect(output).toBe("first\ntwo words\n");
});

test("a missing isolated interpreter fails instead of using system Python", async () => {
  const home = await realpath(
    await mkdtemp(join(tmpdir(), "agent-reach-python-missing-")),
  );
  directories.push(home);
  const child = Bun.spawn(["bash", script, "--version"], {
    stdout: "pipe",
    stderr: "pipe",
    env: { ...process.env, HOME: home },
  });
  expect(await child.exited).not.toBe(0);
});
