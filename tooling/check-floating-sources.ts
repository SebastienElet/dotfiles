import { join } from "node:path";
import { statSync } from "node:fs";
import { z } from "zod";

type Rule = Readonly<{ name: string; pattern: Readonly<RegExp> }>;
type Contents = Readonly<Record<string, readonly string[]>>;
type Entry = readonly [string, readonly string[]];
type Exception = Readonly<{ path: string; line: string; reason: string }>;
type Finding = Readonly<{
  path: string;
  number: number;
  text: string;
  rule: string;
}>;

const argumentOffset = 2;
const homebrewName = /^[a-z0-9][\w@.+-]*(?:\/[\w@.+-]+){0,2}$/u;
const ignoredPaths = [
  /\.md$/u,
  /(?:^|\/)bun\.lock$/u,
  /lazy-lock\.json$/u,
  /^docs\//u,
  /^tooling\/arnes\//u,
  /^tooling\/check-floating-sources\.ts$/u,
  /\.(?:test|integration)\.ts$/u,
  /test-(?:support|command|provider)/u,
  /(?:^|\/)tests\//u,
];
const rules: readonly Rule[] = [
  {
    name: "branch reference in a download URL",
    pattern:
      /(?:\/(?:raw|blob|archive|tree)\/|githubusercontent\.com\/[^/\s]+\/[^/\s]+\/)(?:main|master|HEAD)\//u,
  },
  {
    name: "latest release",
    pattern: /@latest\b|\/releases\/latest\b|\/latest\/download\b|-s latest\b/u,
  },
  {
    name: "installer piped into a shell",
    pattern:
      /\b(?:curl|wget)\b[^|]*\|\s*(?:sudo\s+)?(?:\w+=\S+\s+)*(?:ba|z)?sh\b/u,
  },
  {
    name: "unversioned Volta package",
    pattern: /\bvolta install\s+(?!["$])(?![^\s]*@\d)\S+/u,
  },
  {
    name: "unversioned npm global package",
    pattern: /\bnpm (?:install|i|add) (?:-g|--global)\s+(?![^\s]*@\d)\S+/u,
  },
  {
    name: "unversioned package runner",
    pattern: /\b(?:npx|bunx|uvx|pnpx)\s+(?:-\S+\s+)*(?![^\s]*@\d)[^\s-]\S*/u,
  },
  {
    name: "unpinned Cargo installation",
    pattern: /\bcargo install\b(?!.*--(?:rev|tag|version|locked\b.*--rev))/u,
  },
  {
    name: "unversioned Go installation",
    pattern: /\bgo install\s+(?![^\s]*@v\d)\S+/u,
  },
  {
    name: "unversioned Fisher plugin",
    pattern: /\bfisher install\s+(?![^\s"]*@)[\w.-]+\/[\w.-]+/u,
  },
  {
    name: "floating CI runner label",
    pattern: /\b(?:ubuntu|macos|windows)-latest\b/u,
  },
  {
    name: "Git clone without a revision",
    pattern: /\bgit clone\b(?!.*\b[0-9a-f]{40}\b)/u,
  },
];
const exceptions: readonly Exception[] = [
  {
    path: ".moon/tasks/harness-claude.yml",
    line: "https://claude.ai/install.sh | bash -s latest",
    reason: "Claude Code stays on its latest release by policy",
  },
  {
    path: ".moon/tasks/harness-codex.yml",
    line: "volta install @openai/codex",
    reason: "Codex stays on its latest release by policy",
  },
  {
    path: "install.sh",
    line: "git clone --depth 1 https://github.com/SebastienElet/dotfiles.git .dotfiles",
    reason: "bootstrap of this repository itself",
  },
];
const homebrewLockSchema = z.object({
  formulae: z.record(z.string(), z.string()),
  casks: z.record(z.string(), z.string()),
});

function trackedFiles(root: string): string[] {
  const result = Bun.spawnSync(["git", "-C", root, "ls-files", "-z"], {
    stderr: "inherit",
  });
  if (result.exitCode !== 0) {
    throw new Error("git ls-files failed");
  }
  return result.stdout
    .toString()
    .split("\0")
    .filter((path) => path !== "")
    .filter(
      (path) =>
        !ignoredPaths.some((pattern: Readonly<RegExp>) => pattern.test(path)),
    );
}

function isExcepted(path: string, text: string): boolean {
  return exceptions.some(
    (exception) => exception.path === path && text.includes(exception.line),
  );
}

function withContinuation(lines: readonly string[], index: number): string {
  const text = lines[index] ?? "";
  return /[|\\]\s*$/u.test(text) ? `${text} ${lines[index + 1] ?? ""}` : text;
}

function ruleFindings(path: string, lines: readonly string[]): Finding[] {
  return lines.flatMap((_line, index) => {
    const text = withContinuation(lines, index);
    return isExcepted(path, text)
      ? []
      : rules
          .filter((rule) => rule.pattern.test(text))
          .map((rule) => ({ path, number: index + 1, text, rule: rule.name }));
  });
}

function homebrewFindings(
  path: string,
  lines: readonly string[],
  locked: readonly string[],
): Finding[] {
  return lines.flatMap((text, index) =>
    (/\bbrew install\s+(?<names>[^;&|)\n]+)/u.exec(text)?.groups?.names ?? "")
      .trim()
      .split(/\s+/u)
      .filter((name) => homebrewName.test(name) && !locked.includes(name))
      .map((name) => ({
        path,
        number: index + 1,
        text,
        rule: `Homebrew package ${name} missing from homebrew.lock.json`,
      })),
  );
}

async function lockedHomebrewPackages(root: string): Promise<string[]> {
  const lock = homebrewLockSchema.parse(
    await Bun.file(join(root, "homebrew.lock.json")).json(),
  );
  return [...Object.keys(lock.formulae), ...Object.keys(lock.casks)];
}

function staleExceptions(contents: Contents): string[] {
  return exceptions
    .filter(
      (exception) =>
        !(contents[exception.path] ?? []).some((text) =>
          text.includes(exception.line),
        ),
    )
    .map(
      (exception) => `stale exception in ${exception.path}: ${exception.line}`,
    );
}

async function readTextFiles(
  root: string,
  paths: readonly string[],
): Promise<Contents> {
  const files = paths.filter(
    (path) =>
      statSync(join(root, path), { throwIfNoEntry: false })?.isFile() === true,
  );
  const texts = await Promise.all(
    files.map((path) => Bun.file(join(root, path)).text()),
  );
  return Object.fromEntries(
    files.flatMap((path, position): Entry[] => {
      const text = texts[position] ?? "";
      return text.includes("\0") ? [] : [[path, text.split("\n")]];
    }),
  );
}

async function main(): Promise<number> {
  const root = z.string().default(".").parse(Bun.argv[argumentOffset]);
  const locked = await lockedHomebrewPackages(root);
  const contents = await readTextFiles(root, trackedFiles(root));
  const entries = Object.entries(contents);
  const findings = [
    ...entries.flatMap(([path, lines]: Entry) => ruleFindings(path, lines)),
    ...entries.flatMap(([path, lines]: Entry) =>
      homebrewFindings(path, lines, locked),
    ),
  ];
  const failures = [
    ...findings.map(
      (finding) =>
        `${finding.path}:${finding.number}: ${finding.rule}: ${finding.text.trim()}`,
    ),
    ...staleExceptions(contents),
  ];
  for (const failure of failures) {
    process.stderr.write(`floating source: ${failure}\n`);
  }
  return failures.length === 0 ? 0 : 1;
}

try {
  process.exitCode = await main();
} catch (error) {
  process.stderr.write(
    `floating sources: ${error instanceof Error ? error.message : String(error)}\n`,
  );
  process.exitCode = 1;
}
