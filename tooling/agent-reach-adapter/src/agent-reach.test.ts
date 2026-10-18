import { afterEach, expect, test } from "bun:test";
import {
  mkdir,
  mkdtemp,
  readdir,
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

async function invoke(
  args: readonly string[],
  environment: Readonly<Record<string, string>> = {},
): Promise<
  Readonly<{ stdout: string; stderr: string; status: number; home: string }>
> {
  const home =
    environment.HOME ??
    (await realpath(await mkdtemp(join(tmpdir(), "adapter-cli-"))));
  if (environment.HOME === undefined) {
    directories.push(home);
  }
  const child = Bun.spawn([process.execPath, entry, ...args], {
    cwd: home,
    env: {
      ...process.env,
      HOME: home,
      BUN_RUNTIME_TRANSPILER_CACHE_PATH: "0",
      ...environment,
    },
    stdout: "pipe",
    stderr: "pipe",
  });
  const [stdout, stderr, status] = await Promise.all([
    new Response(child.stdout).text(),
    new Response(child.stderr).text(),
    child.exited,
  ]);
  return { stdout, stderr, status, home };
}

test("plans from its own resources without a checkout argument or home mutation", async () => {
  const result = await invoke(["plan"]);
  expect(result.status).toBe(0);
  expect(result.stdout).toContain("a19a171fa980a0785849596492e0af4db800c82f");
  expect(result.stdout).toContain("mcporter@0.14.2");
  expect(await readdir(result.home)).toEqual([]);
});

test("reports unavailable prerequisites without reading sessions or installing tools", async () => {
  const result = await invoke(["doctor"], { PATH: "" });
  expect(result.status).toBe(1);
  expect(result.stderr).toContain("Missing prerequisite: git");
  expect(await readdir(result.home)).toEqual([]);
});

test.each(
  [
    [],
    ["unknown"],
    ["prepare"],
    ["prepare", "/absent/adapter-home"],
    ["plan", "extra"],
  ].map((args: readonly string[]) => ({ args })),
)(
  "rejects invalid invocation %j",
  async ({ args }: Readonly<{ args: readonly string[] }>) => {
    const result = await invoke(args);
    expect(result.status).toBe(1);
    expect(result.stderr).toContain("Agent-Reach:");
    expect(await readdir(result.home)).toEqual([]);
  },
);

test.each([
  { script: "#!/bin/sh\nexit 7\n", diagnostic: "git failed (7)" },
  {
    script: "#!/bin/sh\nexit 0\n",
    diagnostic: "Empty prerequisite version: git",
  },
])(
  "reports a failed or empty prerequisite probe: $diagnostic",
  async ({ script, diagnostic }) => {
    const bin = await realpath(
      await mkdtemp(join(tmpdir(), "adapter-prerequisite-")),
    );
    directories.push(bin);
    await writeFile(join(bin, "git"), script, { mode: 0o700 });
    const result = await invoke(["doctor"], { PATH: bin });
    expect(result.status).toBe(1);
    expect(result.stderr).toContain(diagnostic);
    expect(await readdir(result.home)).toEqual([]);
  },
);

async function prerequisiteFixture(): Promise<
  Readonly<{ bin: string; home: string }>
> {
  const directory = await realpath(
    await mkdtemp(join(tmpdir(), "adapter-tools-")),
  );
  directories.push(directory);
  const bin = join(directory, "bin");
  const home = join(directory, "home");
  await mkdir(bin);
  await mkdir(home);
  for (const executable of [
    "git",
    "python3",
    "uv",
    "docker",
    "ffmpeg",
    "node",
    "mcporter",
    "agent-reach",
    "claude",
    "codex",
  ]) {
    await writeFile(
      join(bin, executable),
      "#!/bin/sh\nprintf 'fixture 1.0.0\\n'\n",
      { mode: 0o700 },
    );
  }
  for (const executable of [
    "opencli",
    "twitter",
    "bili",
    "boss",
    "gh",
    "curl",
    "yt-dlp",
  ]) {
    await writeFile(join(bin, executable), "#!/bin/sh\nexit 7\n", {
      mode: 0o700,
    });
  }
  return { bin, home };
}

test.each(["opencli", "twitter", "bili", "boss", "gh", "curl", "yt-dlp"])(
  "reports a missing declared platform prerequisite: %s",
  async (executable) => {
    const { bin, home } = await prerequisiteFixture();
    await rm(join(bin, executable));
    const result = await invoke(["doctor"], { PATH: bin, HOME: home });
    expect(result.status).toBe(1);
    expect(result.stderr).toContain(`Missing prerequisite: ${executable}`);
  },
);

test("requires the isolated interpreter even when system Python is available", async () => {
  const { bin, home } = await prerequisiteFixture();
  const result = await invoke(["doctor"], { PATH: bin, HOME: home });
  expect(result.status).toBe(1);
  expect(result.stderr).toContain("Missing prerequisite:");
  expect(result.stderr).toContain("python-tools/agent-reach/bin/python");
});

test("reports available executables without starting platform clients", async () => {
  const { bin, home } = await prerequisiteFixture();
  const pythonDirectory = join(
    home,
    ".local/share/agent-reach/python-tools/agent-reach/bin",
  );
  await mkdir(pythonDirectory, { recursive: true });
  await writeFile(join(pythonDirectory, "python"), "#!/bin/sh\nexit 7\n", {
    mode: 0o700,
  });
  const result = await invoke(["doctor"], { PATH: bin, HOME: home });
  expect(result.status).toBe(0);
  expect(result.stdout).toContain("boss: executable found");
  expect(result.stdout).toContain("python: executable found");
});
