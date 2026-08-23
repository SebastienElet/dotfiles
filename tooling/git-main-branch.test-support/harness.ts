import { chmod, mkdtemp, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";

export type Fixture = {
  repository?: boolean;
  localBranches?: readonly string[];
  showRefStatus?: number;
  showRefStderr?: string;
  remotes?: Readonly<
    Record<
      string,
      { urls: readonly string[]; head?: string; headOutput?: string }
    >
  >;
  contexts?: unknown;
  repositories?: Readonly<Record<string, unknown>>;
  failures?: Readonly<Record<string, { status: number; stderr: string }>>;
};

const entrypoint = join(import.meta.dir, "..", "git-main-branch");
const fakeCommand = join(import.meta.dir, "fake-command.ts");
const temporaryDirectories: string[] = [];

export async function cleanup(): Promise<void> {
  await Promise.all(
    temporaryDirectories.splice(0).map((path) => rm(path, { recursive: true })),
  );
}

export async function run(fixture: Fixture, ...arguments_: string[]) {
  const root = await mkdtemp(join(tmpdir(), "git-main-branch-"));
  temporaryDirectories.push(root);
  const bin = join(root, "bin");
  await Bun.$`mkdir -p ${bin}`.quiet();
  await Bun.write(join(root, "fixture.json"), JSON.stringify(fixture));
  for (const command of ["git", "bkt"] as const) {
    const executable = join(bin, command);
    await Bun.write(
      executable,
      `#!/bin/sh\nexec "${process.execPath}" "${fakeCommand}" ${command} "$@"\n`,
    );
    await chmod(executable, 0o755);
  }
  return Bun.spawnSync([entrypoint, ...arguments_], {
    cwd: root,
    env: {
      ...process.env,
      FIXTURE_PATH: join(root, "fixture.json"),
      PATH: `${bin}:${dirname(process.execPath)}:/usr/bin:/bin`,
    },
    stdout: "pipe",
    stderr: "pipe",
  });
}
