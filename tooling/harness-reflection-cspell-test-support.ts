import {
  chmod,
  mkdir,
  mkdtemp,
  readFile,
  rm,
  writeFile,
} from "node:fs/promises";
import { dirname, join, resolve } from "node:path";
import { createHash } from "node:crypto";
import { spawnSync } from "node:child_process";
import { tmpdir } from "node:os";

const repositoryRoot = resolve(import.meta.dir, "..");
const cspellStepMarker =
  "      - name: Check configuration and user dictionary\n        run: |\n";
const jobHeader =
  /^ {2}(?<key>[A-Za-z_][\w-]*|"[^"]+"|'[^']+')[ \t]*:[^\n]*$/gmu;
const runBlockIndentWidth = 10;
const executableMode = 0o755;

interface CspellGateResult {
  readonly normalizedCallLog: string;
  readonly status: number;
}

interface JobHeaderMatch {
  readonly groups?: Readonly<Record<string, string>>;
  readonly index?: number;
}

const normalizedLines = (workflow: string): string => {
  const normalized = workflow.replaceAll("\r\n", "\n");
  if (normalized.includes("\r")) {
    throw new Error("CSpell workflow has unsupported line endings");
  }
  return normalized;
};

const unquote = (key: string): string => {
  const first = key.at(0);
  return first === '"' || first === "'" ? key.slice(1, -1) : key;
};

const extractCspellJob = (workflow: string): string => {
  const normalized = normalizedLines(workflow);
  const headers = [...normalized.matchAll(jobHeader)];
  const cspellHeaders = headers.filter(
    (header: JobHeaderMatch) => unquote(header.groups?.key ?? "") === "cspell",
  );
  if (cspellHeaders.length !== 1) {
    throw new Error("workflow must contain exactly one CSpell job");
  }
  const [selected] = cspellHeaders;
  if (selected === undefined || selected.index === undefined) {
    throw new Error("workflow must contain exactly one CSpell job");
  }
  const next = headers.find(
    (header: JobHeaderMatch) =>
      header.index !== undefined && header.index > selected.index,
  );
  return normalized.slice(selected.index, next?.index ?? normalized.length);
};

const extractRunBlock = (job: string): string => {
  const markerIndex = job.indexOf(cspellStepMarker);
  if (markerIndex === -1) {
    throw new Error("fingerprinted CSpell step is missing its run block");
  }
  const block = job.slice(markerIndex + cspellStepMarker.length);
  const lines = block.split("\n");
  if (lines.some((line) => line.length > 0 && !line.startsWith("          "))) {
    throw new Error("fingerprinted CSpell run block has invalid indentation");
  }
  return lines.map((line) => line.slice(runBlockIndentWidth)).join("\n");
};

const fakeCommand = (command: string, behavior = "exit 0"): string => `#!/bin/sh
{
  printf '${command}'
  printf '\\t%s' "$@"
  printf '\\n'
} >> "$CALL_LOG"
${behavior}
`;

const installFakeCommands = async (binaryDirectory: string): Promise<void> => {
  const commands: readonly (readonly [string, string])[] = [
    ["npm", fakeCommand("npm")],
    ["make", fakeCommand("make")],
    [
      "cspell",
      fakeCommand(
        "cspell",
        `case "$1" in
  link) exit 0 ;;
  trace)
    printf '%s\\n' '@cspell/dict-fr-fr' "$HOME/.config/cspell/user.txt"
    exit 0
    ;;
  lint) exit "$CSPELL_LINT_STATUS" ;;
  *) exit 64 ;;
esac`,
      ),
    ],
  ];
  await mkdir(binaryDirectory);
  await Promise.all(
    commands.map(async ([name, contents]: readonly [string, string]) => {
      const path = join(binaryDirectory, name);
      await writeFile(path, contents, "utf8");
      await chmod(path, executableMode);
    }),
  );
};

const runCspellGate = async (
  job: string,
  lintStatus = 0,
): Promise<CspellGateResult> => {
  const fixtureRoot = await mkdtemp(join(tmpdir(), "cspell-contract-"));
  const binaryDirectory = join(fixtureRoot, "bin");
  const temporaryDirectory = join(fixtureRoot, "tmp");
  const callLog = join(fixtureRoot, "calls.log");
  try {
    await installFakeCommands(binaryDirectory);
    await mkdir(temporaryDirectory);
    await writeFile(callLog, "", "utf8");
    const result = spawnSync(
      "/bin/bash",
      ["--noprofile", "--norc", "-c", extractRunBlock(job)],
      {
        cwd: repositoryRoot,
        encoding: "utf8",
        env: {
          CALL_LOG: callLog,
          CSPELL_LINT_STATUS: String(lintStatus),
          GITHUB_WORKSPACE: repositoryRoot,
          PATH: `${binaryDirectory}:/usr/bin:/bin`,
          TMPDIR: temporaryDirectory,
        },
      },
    );
    const rawCallLog = await readFile(callLog, "utf8");
    const makeCall = rawCallLog
      .split("\n")
      .find((call) => call.startsWith("make\t"));
    const testHome = dirname(makeCall?.split("\t")[2] ?? "");
    if (testHome === ".") {
      throw new Error("CSpell make call did not expose its temporary home");
    }
    const normalizedCallLog = rawCallLog
      .replaceAll(repositoryRoot, "$REPOSITORY_ROOT")
      .replaceAll(testHome, "$TEST_HOME");
    return { normalizedCallLog, status: result.status ?? 1 };
  } finally {
    await rm(fixtureRoot, { force: true, recursive: true });
  }
};

const sha256 = (value: string): string =>
  createHash("sha256").update(value).digest("hex");

export { extractCspellJob, runCspellGate, sha256 };
