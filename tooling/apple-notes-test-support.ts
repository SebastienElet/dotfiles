import { chmod, mkdtemp, readFile, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";

export type Response = Readonly<{
  status?: number;
  stdout?: string;
  stdoutBytes?: number[];
  stderr?: string;
  exportFile?: string;
  appearDirectory?: string;
}>;
const roots: string[] = [];

export async function cleanupNotesFixtures(): Promise<void> {
  await Promise.all(
    roots.splice(0).map((root) => rm(root, { recursive: true, force: true })),
  );
}

export async function runNotes(
  arguments_: readonly string[],
  responses: readonly Response[],
  stdin = "",
  prepare?: (root: string) => void,
) {
  const root = await mkdtemp(join(tmpdir(), "apple-notes-test-"));
  roots.push(root);
  const bin = join(root, "bin");
  await Bun.$`mkdir -p ${bin}`.quiet();
  const fake = join(root, "osascript.ts");
  await Bun.write(
    fake,
    `#!/usr/bin/env bun
const responses = JSON.parse(process.env.NOTES_RESPONSES);
const state = process.env.NOTES_STATE;
const index = Number((await Bun.file(state).exists()) ? await Bun.file(state).text() : "0");
await Bun.write(state, String(index + 1));
const script = await Bun.stdin.text();
const { appendFile } = await import("node:fs/promises");
await appendFile(process.env.NOTES_LOG, script + "\\u0000");
const response = responses[index] ?? { status: 1, stderr: "unexpected osascript call\\n" };
if (response.appearDirectory) {
  const { mkdir, writeFile } = await import("node:fs/promises");
  await mkdir(response.appearDirectory);
  await writeFile(response.appearDirectory + "/foreign.txt", "foreign");
}
if (response.exportFile) {
  const match = script.match(/set exportPath to "([^"]+)\\/"/);
  if (match) await Bun.write(match[1] + "/" + response.exportFile, "attachment");
}
if (response.stdoutBytes) process.stdout.write(new Uint8Array(response.stdoutBytes));
else process.stdout.write(response.stdout ?? "");
process.stderr.write(response.stderr ?? "");
process.exit(response.status ?? 0);
`,
  );
  const osascript = join(bin, "osascript");
  await Bun.write(
    osascript,
    `#!/bin/sh\nexec "${process.execPath}" "${fake}"\n`,
  );
  await chmod(osascript, 0o755);
  const entrypoint = join(
    import.meta.dir,
    "..",
    ".agents",
    "skills",
    "apple-notes",
    "scripts",
    "notes.sh",
  );
  prepare?.(root);
  const result = Bun.spawnSync([entrypoint, ...arguments_], {
    cwd: root,
    stdin: Buffer.from(stdin),
    stdout: "pipe",
    stderr: "pipe",
    env: {
      ...process.env,
      PATH: `${bin}:${dirname(process.execPath)}:/usr/bin:/bin`,
      NOTES_RESPONSES: JSON.stringify(responses),
      NOTES_STATE: join(root, "state"),
      NOTES_LOG: join(root, "log"),
    },
  });
  const log = (await Bun.file(join(root, "log")).exists())
    ? await readFile(join(root, "log"), "utf8")
    : "";
  return {
    exitCode: result.exitCode,
    stdout: result.stdout.toString(),
    stdoutBytes: [...result.stdout],
    stderr: result.stderr.toString(),
    calls: log.split("\0").filter(Boolean),
    root,
  };
}
