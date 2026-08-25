import { chmod, mkdtemp, readFile, rm } from "node:fs/promises";
import { dirname, join } from "node:path";
import { tmpdir } from "node:os";

type Response = Readonly<{
  status?: number;
  stdout?: string;
  stdoutBytes?: readonly number[];
  stderr?: string;
  exportFile?: string;
  appearDirectory?: string;
}>;
type RunNotesResult = Readonly<{
  calls: readonly string[];
  exitCode: number;
  root: string;
  stderr: string;
  stdout: string;
  stdoutBytes: readonly number[];
}>;
type RunNotesOptions = Readonly<{
  prepare?: (root: string) => void;
  stdin?: string;
}>;
const executableMode = 0o755;
const entrypoint = join(
  import.meta.dir,
  "..",
  ".agents",
  "skills",
  "apple-notes",
  "scripts",
  "notes.sh",
);
const roots: string[] = [];

async function cleanupNotesFixtures(): Promise<void> {
  await Promise.all(
    roots.splice(0).map(async (root) => {
      await rm(root, { force: true, recursive: true });
    }),
  );
}

async function prepareOsaScript(root: string): Promise<string> {
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
  await chmod(osascript, executableMode);
  return bin;
}

async function readCalls(root: string): Promise<readonly string[]> {
  const logPath = join(root, "log");
  const log = (await Bun.file(logPath).exists())
    ? await readFile(logPath, "utf8")
    : "";
  return log.split("\0").filter(Boolean);
}

async function runNotes(
  arguments_: readonly string[],
  responses: readonly Response[],
  options: RunNotesOptions = {},
): Promise<RunNotesResult> {
  const root = await mkdtemp(join(tmpdir(), "apple-notes-test-"));
  roots.push(root);
  const bin = await prepareOsaScript(root);
  options.prepare?.(root);
  const result = Bun.spawnSync([entrypoint, ...arguments_], {
    cwd: root,
    env: {
      ...process.env,
      NOTES_LOG: join(root, "log"),
      NOTES_RESPONSES: JSON.stringify(responses),
      NOTES_STATE: join(root, "state"),
      PATH: `${bin}:${dirname(process.execPath)}:/usr/bin:/bin`,
    },
    stderr: "pipe",
    stdin: Buffer.from(options.stdin ?? ""),
    stdout: "pipe",
  });
  return {
    calls: await readCalls(root),
    exitCode: result.exitCode,
    root,
    stderr: result.stderr.toString(),
    stdout: result.stdout.toString(),
    stdoutBytes: [...result.stdout],
  };
}

export {
  cleanupNotesFixtures,
  runNotes,
  type Response,
  type RunNotesOptions,
  type RunNotesResult,
};
