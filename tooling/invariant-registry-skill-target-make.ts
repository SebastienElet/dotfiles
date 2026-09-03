import { lstatSync, readFileSync } from "node:fs";
import type { ConsumerName } from "./invariant-registry-validation-options.ts";
import { resolve } from "node:path";

const deploymentDestinations: Readonly<Record<ConsumerName, string>> = {
  claude: ".claude",
  codex: ".agents",
  cursor: ".cursor",
};
const makeTimeoutMilliseconds = 2000;
const makeMaxBufferBytes = 64_000;
const makeExecutable = "/usr/bin/make";
const inertCommand = "/usr/bin/true";
const sourceMarker = "__INVARIANT_REGISTRY_SOURCE__";
const rejectedMakefilePatterns = [
  /^[ \t]*(?:-include|include|load|sinclude)\b/mu,
  /^\t[@-]*\+/mu,
  /\$\([ \t]*(?:eval|file|guile)\b/u,
  /\$\{[ \t]*(?:eval|file|guile)\b/u,
  /^[ \t]*override[ \t]+(?:define[ \t]+)?(?:CREATE_SYMLINK|DOTFILES_PATH|HOME|MAKE|SHELL)\b/mu,
] as const;

type MakeDeploymentRequest = Readonly<{
  agent: ConsumerName;
  root: string;
  slug: string;
}>;
type MakeProbeCommandRequest = Readonly<{
  makefile: string;
  probeHome: string;
  root: string;
  target: string;
}>;

const makefileCanBeEvaluated = (path: string): boolean => {
  try {
    const stats = lstatSync(path);
    if (!stats.isFile()) {
      return false;
    }
    const source = new TextDecoder("utf-8", { fatal: true }).decode(
      readFileSync(path),
    );
    return rejectedMakefilePatterns.every(
      (pattern: Readonly<RegExp>) => !pattern.test(source),
    );
  } catch {
    return false;
  }
};

const isRegularFile = (path: string): boolean => {
  try {
    return lstatSync(path).isFile();
  } catch {
    return false;
  }
};

const makeEnvironment = (): Readonly<Record<string, string>> => ({
  GNUMAKEFLAGS: "",
  LANG: "C",
  LC_ALL: "C",
  MAKEFILES: "",
  MAKEFLAGS: "",
  MFLAGS: "",
});

const makeProbeCommand = ({
  makefile,
  probeHome,
  root,
  target,
}: MakeProbeCommandRequest): string[] => [
  makeExecutable,
  "--no-print-directory",
  "--dry-run",
  "--no-builtin-rules",
  "--no-builtin-variables",
  "--file",
  makefile,
  `HOME=${probeHome}`,
  `DOTFILES_PATH=${root}`,
  `SHELL=${inertCommand}`,
  `MAKE=${inertCommand}`,
  `CREATE_SYMLINK=echo ${sourceMarker}$<`,
  target,
];

const runMakeProbe = (
  root: string,
  command: readonly string[],
): string | undefined => {
  try {
    const result = Bun.spawnSync([...command], {
      cwd: root,
      env: makeEnvironment(),
      killSignal: "SIGKILL",
      maxBuffer: makeMaxBufferBytes,
      stderr: "pipe",
      stdout: "pipe",
      timeout: makeTimeoutMilliseconds,
    });
    if (result.exitCode !== 0) {
      return undefined;
    }
    return new TextDecoder("utf-8", { fatal: true }).decode(result.stdout);
  } catch {
    return undefined;
  }
};

const makeResolvesSkillDeployment = ({
  agent,
  root,
  slug,
}: MakeDeploymentRequest): boolean => {
  const makefile = resolve(root, "Makefile");
  if (
    !isRegularFile(makeExecutable) ||
    !isRegularFile(inertCommand) ||
    !makefileCanBeEvaluated(makefile)
  ) {
    return false;
  }
  const probeHome = resolve(root, ".invariant-registry-make-probe");
  const target = resolve(
    probeHome,
    deploymentDestinations[agent],
    "skills",
    slug,
  );
  const source = resolve(root, "harness/skills", slug);
  const output = runMakeProbe(
    root,
    makeProbeCommand({ makefile, probeHome, root, target }),
  );
  if (output === undefined) {
    return false;
  }
  const matches = output
    .split(/\r?\n/u)
    .filter((line) => line.includes(sourceMarker));
  return matches.length === 1 && matches[0] === `echo ${sourceMarker}${source}`;
};

export { makeResolvesSkillDeployment };
