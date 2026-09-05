import {
  type LoadedCase,
  fingerprint,
  readSource,
  section,
} from "./sources.ts";
import { type Observation, observationSchema } from "./contracts.ts";
import { dirname, join } from "node:path";
import {
  mkdirSync,
  mkdtempSync,
  readFileSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";

type PreparedFixture = Readonly<{
  root: string;
  workspace: string;
  home: string;
  observations: string;
  env: Readonly<{
    HOME: string;
    CODEX_HOME: string;
    PATH: string;
    [key: string]: string;
  }>;
  instructionFingerprint: string;
  skillFingerprint: string;
  fixtureRevision: string;
}>;

function installFiles(
  workspace: string,
  files: Readonly<Record<string, string>>,
): void {
  for (const [path, content] of Object.entries(files)) {
    const destination = join(workspace, path);
    mkdirSync(dirname(destination), { recursive: true });
    writeFileSync(destination, content, { flag: "wx" });
  }
}

function installShims(workspace: string, shim: string): string {
  const bin = join(workspace, ".eval-bin");
  mkdirSync(bin);
  writeFileSync(join(bin, "shim.ts"), shim, { flag: "wx" });
  for (const tool of ["cat", "rg", "fd", "colgrep-search"]) {
    writeFileSync(
      join(bin, tool),
      `#!${process.execPath}\nimport { invokeShim } from "./shim.ts";\ninvokeShim("${tool}");\n`,
      { mode: 0o755, flag: "wx" },
    );
  }
  return bin;
}

function fixtureEnvironment(
  root: string,
  shim: string,
): Pick<PreparedFixture, "env" | "home" | "workspace" | "observations"> {
  const workspace = join(root, "workspace");
  const home = join(root, "home");
  mkdirSync(join(home, ".codex"), { recursive: true });
  const bin = installShims(workspace, shim);
  const observations = join(workspace, ".observations.jsonl");
  writeFileSync(observations, "", { flag: "wx" });
  return {
    workspace,
    home,
    observations,
    env: {
      HOME: home,
      CODEX_HOME: join(home, ".codex"),
      XDG_CONFIG_HOME: join(home, ".config"),
      PATH: `${bin}:${dirname(process.execPath)}:/usr/bin:/bin`,
      HARNESS_EVAL_WORKSPACE: workspace,
      HARNESS_EVAL_OBSERVATIONS: observations,
      LANG: "C.UTF-8",
      GIT_CONFIG_NOSYSTEM: "1",
      GIT_CONFIG_GLOBAL: "/dev/null",
    },
  };
}

function prepareFixture(
  repository: string,
  entry: LoadedCase,
  variant?: string,
): PreparedFixture {
  const root = mkdtempSync(join(tmpdir(), "harness-eval-"));
  try {
    const instructions =
      variant === undefined
        ? section(
            readSource(repository, "harness/AGENTS.md"),
            "Context Management",
          )
        : readSource(repository, variant);
    const skill = readSource(repository, "harness/skills/code-search/SKILL.md");
    const shim = readSource(repository, "tooling/harness-eval/shim.ts");
    installFiles(join(root, "workspace"), {
      ...entry.fixture.files,
      "AGENTS.md": instructions,
      ".agents/skills/code-search/SKILL.md": skill,
    });
    return {
      root,
      ...fixtureEnvironment(root, shim),
      instructionFingerprint: fingerprint(instructions),
      skillFingerprint: fingerprint(skill),
      fixtureRevision: fingerprint(`${JSON.stringify(entry.fixture)}${shim}`),
    };
  } catch (error) {
    rmSync(root, { recursive: true, force: true });
    throw error;
  }
}

function collectObservations(path: string): readonly Observation[] {
  return readFileSync(path, "utf8")
    .split("\n")
    .filter(Boolean)
    .map((line) => observationSchema.parse(JSON.parse(line)));
}

export { collectObservations, prepareFixture };
export type { PreparedFixture };
