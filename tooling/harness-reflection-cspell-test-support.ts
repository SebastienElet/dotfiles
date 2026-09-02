const cspellStepMarker =
  "      - name: Check configuration and user dictionary\n        run: |\n";
const jobHeader =
  /^ {2}(?<key>[A-Za-z_][\w-]*|"[^"]+"|'[^']+')[ \t]*:[^\n]*$/gmu;
const runBlockIndentWidth = 10;
const commandNameAndVerbLength = 2;

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
    throw new Error("CSpell step is missing its run block");
  }
  const block = job.slice(markerIndex + cspellStepMarker.length);
  const lines = block.split("\n");
  if (lines.some((line) => line.length > 0 && !line.startsWith("          "))) {
    throw new Error("CSpell run block has invalid indentation");
  }
  return lines.map((line) => line.slice(runBlockIndentWidth)).join("\n");
};

const commandLines = (job: string): readonly string[] =>
  extractRunBlock(job)
    .replaceAll(/\\\n\s*/gu, " ")
    .split("\n")
    .map((line) => line.trim())
    .filter((line) => line !== "");

const matchingArgv = (
  job: string,
  label: string,
  matchesCommand: (argv: readonly string[]) => boolean,
): readonly string[] => {
  const matches = commandLines(job)
    .map((line) => line.split(/\s+/u))
    .filter((argv: readonly string[]) => matchesCommand(argv));
  if (matches.length !== 1) {
    throw new Error(`CSpell job must contain one ${label} command`);
  }
  return matches[0] ?? [];
};

const extractCspellInstallArgv = (job: string): readonly string[] =>
  matchingArgv(
    job,
    "npm install",
    (argv) => argv[0] === "npm" && argv[1] === "install",
  );

const extractCspellLintArgv = (job: string): readonly string[] => {
  const argv = matchingArgv(job, "cspell lint", (candidate) => {
    const index = candidate.indexOf("cspell");
    return index !== -1 && candidate[index + 1] === "lint";
  });
  const cspellIndex = argv.indexOf("cspell");
  if (cspellIndex === -1 || argv[cspellIndex + 1] !== "lint") {
    throw new Error("CSpell lint command is missing");
  }
  return argv.slice(cspellIndex);
};

const cspellGateIsFailClosed = (job: string): boolean => {
  const lines = commandLines(job);
  const lintArgv = extractCspellLintArgv(job);
  return (
    job.includes("runs-on: ubuntu-latest") &&
    lines[0] === "set -euo pipefail" &&
    lintArgv.slice(0, commandNameAndVerbLength).join(" ") === "cspell lint" &&
    !lintArgv.some((argument) => ["||", "&&", ";"].includes(argument))
  );
};

export {
  cspellGateIsFailClosed,
  extractCspellInstallArgv,
  extractCspellJob,
  extractCspellLintArgv,
};
