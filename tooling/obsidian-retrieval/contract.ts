import { resolve } from "node:path";
import { validateDefaultCorpus } from "./default-corpus-contract.ts";
import { validateEvaluations } from "./evaluation-contract.ts";
import { z } from "zod";

type ContractSources = Readonly<{
  skill: string;
  reference: string;
  userInstructions: string;
  evaluations: unknown;
}>;

const expectedCommands = new Set([
  "aliases",
  "backlinks",
  "base:query",
  "base:views",
  "bases",
  "file",
  "files",
  "links",
  "properties",
  "property:read",
  "read",
  "search",
  "search:context",
  "tag",
  "tags",
  "tasks",
  "vault",
]);
const unsafeCommands = new Set(
  `append base:create bookmark command create daily:append daily:prepend delete dev:cdp dev:console
  dev:css dev:debug dev:dom dev:errors dev:mobile dev:screenshot devtools eval history:open
  history:restore hotkey move open plugin plugin:disable plugin:enable plugin:install plugin:reload
  plugin:uninstall plugins:restrict prepend property:remove property:set publish:add publish:open
  publish:remove random reload rename restart search:open snippet:disable snippet:enable sync:open
  sync:restore tab:open task template:insert theme theme:install theme:set theme:uninstall unique
  vault:open vaults web workspace:delete workspace:load workspace:save`.split(
    /\s+/u,
  ),
);
const dangerousGuidance = new Set([
  "append",
  "base:create",
  "create",
  "daily:append",
  "daily:prepend",
  "delete",
  "eval",
  "move",
  "plugin:disable",
  "plugin:enable",
  "plugin:install",
  "plugin:uninstall",
  "prepend",
  "property:remove",
  "property:set",
  "publish:add",
  "publish:remove",
  "rename",
  "snippet:disable",
  "snippet:enable",
  "template:insert",
  "theme:install",
  "theme:set",
  "theme:uninstall",
  "workspace:delete",
  "workspace:save",
]);
const unsafeCommandPattern = [...unsafeCommands]
  .toSorted((left, right) => right.length - left.length)
  .map((command) => command.replaceAll(/[.*+?^${}()|[\]\\]/gu, String.raw`\$&`))
  .join("|");
const requiredSkillPhrases = [
  "explicit corpus input",
  "configured default Obsidian corpus",
  "nearest ancestor containing `.obsidian/`",
  "single current workspace root",
  "Never search parent directories, `$HOME`, or Obsidian",
  "Exact anchors",
  "Conceptual questions",
  "Obsidian semantics",
  "Read every source note used for the answer",
  "candidate, not evidence",
  "not prove that the information is absent",
  "Report unavailable tools, inaccessible roots, and incomplete indexes",
  "Do not install QMD",
  "Never replace a missing or inaccessible local corpus with Web retrieval",
  "Never write, create, append, prepend, move, rename, or delete vault content",
  "references/obsidian-cli.md",
];
const allowedReferenceTokens = new Set([
  ...expectedCommands,
  ".base",
  "obsidian vault info=path",
  "obsidian version",
]);
const repositoryRootSchema = z
  .string()
  .min(1, "repository root must not be empty");
const matchesExpectedCommands = (values: readonly string[]): boolean =>
  values.length === expectedCommands.size &&
  new Set(values).size === values.length &&
  values.every((value) => expectedCommands.has(value));
const backtickedTokens = (text: string): string[] => {
  const tokens: string[] = [];
  for (const match of text.matchAll(/`(?<token>[^`]+)`/gu)) {
    if (match.groups?.token !== undefined) {
      tokens.push(match.groups.token);
    }
  }
  return tokens;
};
const allowlistSection = (reference: string): string | undefined => {
  const headings = [...reference.matchAll(/^## .+$/gmu)];
  const matches: RegExpExecArray[] = [];
  for (const heading of headings) {
    if (heading[0] === "## Read-only allowlist") {
      matches.push(heading);
    }
  }
  const [allowlistHeading] = matches;
  if (matches.length !== 1 || allowlistHeading?.index === undefined) {
    return undefined;
  }
  const start = allowlistHeading.index + allowlistHeading[0].length;
  let nextIndex: number | undefined = undefined;
  for (const heading of headings) {
    if (heading.index !== undefined && heading.index > start) {
      nextIndex = heading.index;
      break;
    }
  }
  return reference.slice(start, nextIndex ?? reference.length);
};
const positiveDangerousGuidance = (skill: string): string[] => {
  const sentences = skill.replaceAll(/\s+/gu, " ").split(/(?<=[.!?])\s+/u);
  const findings: string[] = [];
  for (const sentence of sentences) {
    const normalizedSentence = sentence.toLowerCase();
    const tokens = normalizedSentence.match(/[a-z]+(?::[a-z]+)*/gu) ?? [];
    const dangerous = tokens.filter((token) => dangerousGuidance.has(token));
    const positiveDangerous = dangerous.filter((token) => {
      const index = normalizedSentence.indexOf(token);
      const prefix = sentence.slice(0, index);
      return !/\b(?:do not|does not|must not|never|refuse to|without)\b/iu.test(
        prefix,
      );
    });
    const imperativePattern = new RegExp(
      `\\b(?:allow(?:ed)?|permit(?:ted)?|run|use|invoke|expose)\\s+(?:the\\s+)?(?:command\\s+)?(${unsafeCommandPattern})(?![\\w:])`,
      "giu",
    );
    findings.push(...positiveDangerous);
    for (const match of sentence.matchAll(imperativePattern)) {
      const prefix = sentence.slice(0, match.index);
      const negated =
        /\b(?:do not|does not|must not|never|refuse to|without)\b/iu.test(
          prefix,
        );
      if (!negated && match[1] !== undefined) {
        findings.push(match[1]);
      }
    }
  }
  return findings;
};

function validateSkill(skill: string): string[] {
  const errors: string[] = [];
  const normalizedSkill = skill.replaceAll(/\s+/gu, " ");
  for (const phrase of requiredSkillPhrases) {
    if (!normalizedSkill.includes(phrase.replaceAll(/\s+/gu, " "))) {
      errors.push(`SKILL.md is missing: ${phrase}`);
    }
  }
  return errors;
}

function validateAllowlist(reference: string): string[] {
  const errors: string[] = [];
  const section = allowlistSection(reference);
  if (section === undefined) {
    errors.push("the read-only allowlist must occur exactly once");
  } else {
    const commands = backtickedTokens(section);
    if (!matchesExpectedCommands(commands)) {
      errors.push("the read-only allowlist does not match the canonical set");
    }
    if (commands.some((command) => unsafeCommands.has(command))) {
      errors.push("the read-only allowlist contains an unsafe command");
    }
  }
  return errors;
}

function validateReference(skill: string, reference: string): string[] {
  const errors = validateAllowlist(reference);
  const unexpectedTokens = backtickedTokens(reference).filter(
    (token) => !allowedReferenceTokens.has(token),
  );
  if (unexpectedTokens.length > 0) {
    errors.push(`unexpected reference tokens: ${unexpectedTokens.join(", ")}`);
  }
  const invocations: string[] = [];
  for (const match of `${skill}\n${reference}`.matchAll(
    /\bobsidian\s+(?<command>[a-z][a-z:-]*)/gu,
  )) {
    const command = match.groups?.command;
    if (command !== undefined && command !== "version" && command !== "vault") {
      invocations.push(command);
    }
  }
  if (invocations.length > 0) {
    errors.push(`unsafe Obsidian invocations: ${invocations.join(", ")}`);
  }
  const dangerous = positiveDangerousGuidance(`${skill}\n${reference}`);
  if (dangerous.length > 0) {
    errors.push(`positive dangerous guidance: ${dangerous.join(", ")}`);
  }
  return errors;
}

const validateContract = ({
  skill,
  reference,
  userInstructions,
  evaluations,
}: ContractSources): string[] => [
  ...validateSkill(skill),
  ...validateReference(skill, reference),
  ...validateDefaultCorpus(userInstructions),
  ...validateEvaluations(evaluations),
];

const loadContractSources = async (
  repositoryRoot: string,
): Promise<ContractSources> => {
  const skillRoot = resolve(
    repositoryRoot,
    "harness/skills/obsidian-retrieval",
  );
  try {
    const [skill, reference, userInstructions, evaluationsText] =
      await Promise.all([
        Bun.file(resolve(skillRoot, "SKILL.md")).text(),
        Bun.file(resolve(skillRoot, "references/obsidian-cli.md")).text(),
        Bun.file(resolve(repositoryRoot, "harness/USER.md")).text(),
        Bun.file(resolve(skillRoot, "evals/trigger-queries.json")).text(),
      ]);
    return {
      evaluations: JSON.parse(evaluationsText) as unknown,
      reference,
      skill,
      userInstructions,
    };
  } catch (error) {
    throw new Error(
      `unable to read contract inputs: ${error instanceof Error ? error.message : String(error)}`,
      { cause: error },
    );
  }
};

if (import.meta.main) {
  try {
    const repositoryRoot =
      process.argv[2] === undefined
        ? process.cwd()
        : repositoryRootSchema.parse(process.argv[2]);
    const errors = validateContract(await loadContractSources(repositoryRoot));
    if (errors.length > 0) {
      throw new Error(errors.join("\n"));
    }
    process.stdout.write("Obsidian retrieval contract passed\n");
  } catch (error) {
    process.stderr.write(
      `${error instanceof Error ? error.message : String(error)}\n`,
    );
    process.exitCode = 1;
  }
}

export { loadContractSources, validateContract };
export type { ContractSources };
