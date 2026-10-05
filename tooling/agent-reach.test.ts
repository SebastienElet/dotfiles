import { afterEach, expect, test } from "bun:test";
import { mkdir, mkdtemp, realpath, rm, writeFile } from "node:fs/promises";
import { join } from "node:path";
import { tmpdir } from "node:os";

const directories: string[] = [];
const entry = join(import.meta.dir, "agent-reach.ts");
afterEach(async () => {
  await Promise.all(
    directories
      .splice(0)
      .map((path) => rm(path, { recursive: true, force: true })),
  );
});

async function invoke(
  ...args: readonly string[]
): Promise<Readonly<{ stdout: string; stderr: string; status: number }>> {
  const child = Bun.spawn([process.execPath, entry, ...args], {
    stdout: "pipe",
    stderr: "pipe",
  });
  const [stdout, stderr, status] = await Promise.all([
    new Response(child.stdout).text(),
    new Response(child.stderr).text(),
    child.exited,
  ]);
  return { stdout, stderr, status };
}

test("reports the pinned source and backend coordinates without installing anything", async () => {
  const root = await realpath(
    await mkdtemp(join(tmpdir(), "agent-reach-plan-")),
  );
  directories.push(root);
  await mkdir(join(root, "harness/plugins/agent-reach"), { recursive: true });
  await writeFile(
    join(root, "harness/plugins/agent-reach/source.json"),
    JSON.stringify({
      version: "1.2.3",
      revision: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
      python: ["example==1.2.3"],
      node: ["example@1.2.3"],
    }),
  );
  const result = await invoke("plan", root, tmpdir());
  expect(result.status).toBe(0);
  expect(result.stdout).toContain("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa");
  expect(result.stdout).toContain("example@1.2.3");
});

test("rejects malformed metadata before running any installer", async () => {
  const root = await realpath(
    await mkdtemp(join(tmpdir(), "agent-reach-cli-")),
  );
  directories.push(root);
  await mkdir(join(root, "harness/plugins/agent-reach"), { recursive: true });
  await writeFile(
    join(root, "harness/plugins/agent-reach/source.json"),
    '{"revision":"main"}',
  );
  const result = await invoke("runtime", root, root);
  expect(result.status).not.toBe(0);
  expect(result.stderr).toContain("Agent-Reach:");
});

test("rejects unsupported operations and missing arguments", async () => {
  const unknown = await invoke("unknown", "/tmp", "/tmp");
  const missing = await invoke();
  expect(unknown.status).not.toBe(0);
  expect(missing.status).not.toBe(0);
});
