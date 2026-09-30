import { expect, test } from "bun:test";
import { mkdtemp, rm, writeFile } from "node:fs/promises";
import { join } from "node:path";
import { tmpdir } from "node:os";

const entry = join(import.meta.dir, "bitbucket-linear-sync");
const usageError = 2;
function invoke(
  args: readonly string[],
  environment: Readonly<NodeJS.ProcessEnv> = process.env,
): Bun.SyncSubprocess<"pipe", "pipe"> {
  return Bun.spawnSync([process.execPath, entry, ...args], {
    env: environment,
    stdout: "pipe",
    stderr: "pipe",
  });
}

test("real entrypoint documents inspection as default", () => {
  const result = invoke(["--help"]);
  expect(result.exitCode).toBe(0);
  expect(result.stdout.toString()).toContain("Inspection is the default");
});

test.each([
  { args: ["--apply"] },
  { args: ["--unknown"] },
  { args: ["--config", "one", "--config", "two"] },
  { args: ["--json", "--json"] },
])(
  "invalid argv exits 2 before any provider %#",
  ({ args }: Readonly<{ args: readonly string[] }>) => {
    const result = invoke(args, { PATH: "" });
    expect(result.exitCode).toBe(usageError);
    expect(result.stderr.toString()).toContain("usage/configuration");
  },
);

test("invalid config exits 2 without printing sensitive input", async () => {
  const directory = await mkdtemp(join(tmpdir(), "bitbucket-linear-"));
  try {
    const path = join(directory, "config.json");
    await writeFile(path, JSON.stringify({ token: "DO-NOT-PRINT" }));
    const result = invoke(["--config", path, "--apply"], { PATH: "" });
    expect(result.exitCode).toBe(usageError);
    expect(result.stderr.toString()).not.toContain("DO-NOT-PRINT");
  } finally {
    await rm(directory, { recursive: true, force: true });
  }
});

test("missing provider command fails visibly with code 1 rather than passing empty inventory", async () => {
  const directory = await mkdtemp(join(tmpdir(), "bitbucket-linear-"));
  try {
    const path = join(directory, "config.json");
    await writeFile(
      path,
      JSON.stringify({
        bktContext: "work",
        linearWorkspace: "work",
        repositories: [
          {
            workspace: "acme",
            repository: "app",
            teamKey: "ENG",
            teamId: "00000000-0000-4000-8000-000000000001",
          },
        ],
      }),
    );
    const result = invoke(["--config", path, "--json"], { PATH: "" });
    expect(result.exitCode).toBe(1);
    expect(result.stderr.toString()).toContain("provider/inventory");
  } finally {
    await rm(directory, { recursive: true, force: true });
  }
});
