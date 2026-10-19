import { afterEach, expect, test } from "bun:test";
import {
  mkdir,
  mkdtemp,
  readFile,
  realpath,
  rm,
  writeFile,
} from "node:fs/promises";
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

async function fixture(): Promise<
  Readonly<{
    home: string;
    repository: string;
    adapter: string;
    marketplace: string;
  }>
> {
  const home = await realpath(
    await mkdtemp(join(tmpdir(), "agent-reach-installation-")),
  );
  directories.push(home);
  const repository = join(home, "repository");
  const adapter = join(
    repository,
    "node_modules/@sebastienelet/agent-reach-adapter",
  );
  const marketplace = join(
    home,
    ".local/share/agent-reach/adapter/0.0.1/marketplace",
  );
  await mkdir(join(adapter, "resources"), { recursive: true });
  await mkdir(join(adapter, "src"));
  await writeFile(
    join(adapter, "resources/source.json"),
    '{"version":"0.2.0","revision":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","python":["example==1.2.3"],"node":["example@1.2.3"]}',
  );
  return { home, repository, adapter, marketplace };
}

async function invoke(
  operation: string,
  repository: string,
  home: string,
): Promise<Readonly<{ status: number; stdout: string; stderr: string }>> {
  const child = Bun.spawn(
    [process.execPath, entry, operation, repository, home],
    { stdout: "pipe", stderr: "pipe" },
  );
  const [status, stdout, stderr] = await Promise.all([
    child.exited,
    new Response(child.stdout).text(),
    new Response(child.stderr).text(),
  ]);
  return { status, stdout, stderr };
}

test("uses the packaged prepare entry point and copies its podcast resource", async () => {
  const { home, repository, adapter, marketplace } = await fixture();
  await mkdir(join(marketplace, "plugins/agent-reach/scripts"), {
    recursive: true,
  });
  await writeFile(
    join(marketplace, "plugins/agent-reach/scripts/transcribe-xiaoyuzhou.sh"),
    "podcast",
  );
  await writeFile(
    join(adapter, "src/agent-reach.ts"),
    `if (process.argv[2] !== "prepare" || process.argv[3] !== ${JSON.stringify(home)}) process.exit(2); process.stdout.write(${JSON.stringify(marketplace)});`,
  );
  const result = await invoke("package", repository, home);
  expect(result.status).toBe(0);
  expect(result.stdout.trim()).toBe(marketplace);
  expect(
    await readFile(
      join(home, ".agent-reach/tools/xiaoyuzhou/transcribe.sh"),
      "utf8",
    ),
  ).toBe("podcast");
});

test("refuses a failed adapter and an unexpected preparation path", async () => {
  const { home, repository, adapter } = await fixture();
  await writeFile(
    join(adapter, "src/agent-reach.ts"),
    'process.stderr.write("prepare refused"); process.exit(1);',
  );
  const failed = await invoke("package", repository, home);
  expect(failed.status).not.toBe(0);
  expect(failed.stderr).toContain("prepare refused");
  await writeFile(
    join(adapter, "src/agent-reach.ts"),
    'process.stdout.write("/unexpected/marketplace");',
  );
  const unexpected = await invoke("package", repository, home);
  expect(unexpected.status).not.toBe(0);
  expect(unexpected.stderr).toContain(
    "Unexpected Agent-Reach marketplace path",
  );
});

test("passes the published marketplace identity to both native plugin managers", async () => {
  const { home, repository, marketplace } = await fixture();
  for (const [host, binary] of [
    ["claude", ".local/bin/claude"],
    ["codex", ".volta/bin/codex"],
  ] as const) {
    const executable = join(home, binary);
    await mkdir(join(executable, ".."), { recursive: true });
    await writeFile(
      executable,
      `#!${process.execPath}\nprocess.stdout.write(JSON.stringify(process.argv.slice(2)) + "\\n");\n`,
      { mode: 0o700 },
    );
    const result = await invoke(host, repository, home);
    if (process.platform === "darwin") {
      expect(result.status).toBe(0);
      expect(result.stdout).toContain(
        JSON.stringify(["plugin", "marketplace", "add", marketplace]),
      );
      expect(result.stdout).toContain("agent-reach@agent-reach-adapter");
    } else {
      expect(result.status).not.toBe(0);
      expect(result.stderr).toContain("targets macOS");
    }
  }
});
