import { resolve } from "node:path";
import { z } from "zod";
import { validateDefaultCorpus } from "./default-corpus-contract.ts";
import { validateEvaluations } from "./evaluation-contract.ts";

export type ContractSources = Readonly<{
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
    /\s+/,
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
  .sort((left, right) => right.length - left.length)
  .map((command) => command.replace(/[.*+?^${}()|[\]\\]/g, "\\$&"))
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

const sameSet = (values: string[], expected: Set<string>): boolean =>
  values.length === expected.size &&
  new Set(values).size === values.length &&
  values.every((value) => expected.has(value));

const backtickedTokens = (text: string): string[] =>
  [...text.matchAll(/`([^`]+)`/g)].flatMap((match) =>
    match[1] === undefined ? [] : [match[1]],
  );

const allowlistSection = (reference: string): string | undefined => {
  const headings = [...reference.matchAll(/^## .+$/gm)];
  const matches = headings.filter(
    (match) => match[0] === "## Read-only allowlist",
  );
  if (matches.length !== 1 || matches[0]?.index === undefined) return undefined;
  const start = matches[0].index + matches[0][0].length;
  const next = headings.find(
    (match) => match.index !== undefined && match.index > start,
  );
  return reference.slice(start, next?.index ?? reference.length);
};

const positiveDangerousGuidance = (skill: string): string[] => {
  const sentences = skill.replace(/\s+/g, " ").split(/(?<=[.!?])\s+/);
  return sentences.flatMap((sentence) => {
    const normalizedSentence = sentence.toLowerCase();
    const tokens = normalizedSentence.match(/[a-z]+(?::[a-z]+)*/g) ?? [];
    const dangerous = tokens.filter((token) => dangerousGuidance.has(token));
    const positiveDangerous = dangerous.filter((token) => {
      const index = normalizedSentence.indexOf(token);
      const prefix = sentence.slice(0, index);
      return !/\b(?:do not|does not|must not|never|refuse to|without)\b/i.test(
        prefix,
      );
    });
    const imperativePattern = new RegExp(
      `\\b(?:allow(?:ed)?|permit(?:ted)?|run|use|invoke|expose)\\s+(?:the\\s+)?(?:command\\s+)?(${unsafeCommandPattern})(?![\\w:])`,
      "gi",
    );
    const imperativeDangerous = [
      ...sentence.matchAll(imperativePattern),
    ].flatMap((match) => {
      const prefix = sentence.slice(0, match.index);
      return /\b(?:do not|does not|must not|never|refuse to|without)\b/i.test(
        prefix,
      ) || match[1] === undefined
        ? []
        : [match[1]];
    });
    return [...positiveDangerous, ...imperativeDangerous];
  });
};

export const validateContract = ({
  skill,
  reference,
  userInstructions,
  evaluations,
}: ContractSources): string[] => {
  const errors: string[] = [];
  const normalizedSkill = skill.replace(/\s+/g, " ");
  for (const phrase of requiredSkillPhrases) {
    if (!normalizedSkill.includes(phrase.replace(/\s+/g, " ")))
      errors.push(`SKILL.md is missing: ${phrase}`);
  }
  const section = allowlistSection(reference);
  if (section === undefined) {
    errors.push("the read-only allowlist must occur exactly once");
  } else {
    const commands = backtickedTokens(section);
    if (!sameSet(commands, expectedCommands))
      errors.push("the read-only allowlist does not match the canonical set");
    if (commands.some((command) => unsafeCommands.has(command)))
      errors.push("the read-only allowlist contains an unsafe command");
  }
  const unexpectedTokens = backtickedTokens(reference).filter(
    (token) => !allowedReferenceTokens.has(token),
  );
  if (unexpectedTokens.length > 0)
    errors.push(`unexpected reference tokens: ${unexpectedTokens.join(", ")}`);
  const invocations = [
    ...`${skill}\n${reference}`.matchAll(/\bobsidian\s+([a-z][a-z:-]*)/g),
  ]
    .flatMap((match) => (match[1] === undefined ? [] : [match[1]]))
    .filter((command) => command !== "version" && command !== "vault");
  if (invocations.length > 0)
    errors.push(`unsafe Obsidian invocations: ${invocations.join(", ")}`);
  const dangerous = positiveDangerousGuidance(`${skill}\n${reference}`);
  if (dangerous.length > 0)
    errors.push(`positive dangerous guidance: ${dangerous.join(", ")}`);
  errors.push(...validateDefaultCorpus(userInstructions));
  errors.push(...validateEvaluations(evaluations));
  return errors;
};

export const loadContractSources = async (
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
      skill,
      reference,
      userInstructions,
      evaluations: JSON.parse(evaluationsText) as unknown,
    };
  } catch (error) {
    throw new Error(
      `unable to read contract inputs: ${error instanceof Error ? error.message : String(error)}`,
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
    if (errors.length > 0) throw new Error(errors.join("\n"));
    console.log("Obsidian retrieval contract passed");
  } catch (error) {
    console.error(error instanceof Error ? error.message : String(error));
    process.exitCode = 1;
  }
}
