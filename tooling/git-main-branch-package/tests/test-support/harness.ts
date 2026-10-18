import { chmod, mkdtemp, rm } from "node:fs/promises";
import { dirname, join } from "node:path";
import { tmpdir } from "node:os";

interface Fixture {
  repository?: boolean;
  localBranches?: readonly string[];
  showRefStatus?: number;
  showRefStderr?: string;
  failures?: Readonly<
    Record<string, Readonly<{ status: number; stderr: string }>>
  >;
}

type CommandResult = Readonly<{
  exitCode: number;
  stderr: Uint8Array;
  stdout: Uint8Array;
}>;

const entrypoint = join(import.meta.dir, "../..", "cli.ts");
const fakeCommand = join(import.meta.dir, "fake-command.ts");
const temporaryDirectories: string[] = [];
const executableFileMode = 0o755;

async function cleanup(): Promise<void> {
  await Promise.all(
    temporaryDirectories.splice(0).map((path) => rm(path, { recursive: true })),
  );
}

async function run(
  fixture: Readonly<Fixture>,
  ...commandArguments: readonly string[]
): Promise<CommandResult> {
  const root = await mkdtemp(join(tmpdir(), "git-main-branch-"));
  temporaryDirectories.push(root);
  const bin = join(root, "bin");
  await Bun.$`mkdir -p ${bin}`.quiet();
  await Bun.write(join(root, "fixture.json"), JSON.stringify(fixture));
  const executable = join(bin, "git");
  await Bun.write(
    executable,
    `#!/bin/sh\nexec "${process.execPath}" "${fakeCommand}" git "$@"\n`,
  );
  await chmod(executable, executableFileMode);
  return Bun.spawnSync([entrypoint, ...commandArguments], {
    cwd: root,
    env: {
      ...process.env,
      FIXTURE_PATH: join(root, "fixture.json"),
      PATH: `${bin}:${dirname(process.execPath)}:/usr/bin:/bin`,
    },
    stderr: "pipe",
    stdout: "pipe",
  });
}

export { cleanup, run };
export type { Fixture };
