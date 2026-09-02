import { expect, test } from "bun:test";
import { resolve } from "node:path";

type HarnessReflectionSources = Readonly<{
  evals: unknown;
  reference: string;
  skill: string;
}>;
type EvalQuery = Readonly<{ query: string; shouldActivate: boolean }>;
type EvalSources = Readonly<{
  queries: readonly EvalQuery[];
  version?: unknown;
}>;
type ContractRequirement = readonly [string, string, readonly string[]];

const repositoryRoot = resolve(import.meta.dir, "..");

const loadHarnessReflectionSources = async (
  root: string,
): Promise<HarnessReflectionSources> => {
  const skillRoot = resolve(root, "harness/skills/harness-reflection");
  return {
    evals: await Bun.file(
      resolve(skillRoot, "evals/trigger-queries.json"),
    ).json(),
    reference: await Bun.file(
      resolve(skillRoot, "references/invariant-registry.md"),
    ).text(),
    skill: await Bun.file(resolve(skillRoot, "SKILL.md")).text(),
  };
};

const containsAll = (source: string, terms: readonly string[]): boolean =>
  terms.every((term) => source.includes(term));

const sourceRequirements = (
  sources: HarnessReflectionSources,
): readonly ContractRequirement[] =>
  [
    [
      "skill routes factual PR evidence through the registry",
      sources.skill,
      [
        "pr-feedback",
        "harness-gap",
        "references/invariant-registry.md",
        "skip",
        "link",
        "propose",
        "skill-manager",
        "agent-instructions",
      ],
    ],
    [
      "reference names the canonical registry and CLI",
      sources.reference,
      [
        "harness/invariants/registry.json",
        "bun tooling/invariant-registry-cli.ts",
        "not-applied",
        "not-loaded",
        "unknown",
        "blind-spot",
        "judgment",
        "explicit approval",
        "surface",
        "Claude",
        "Codex",
        "Cursor",
        "oracle",
        "## Required report",
        "Registry lookup",
        "Decision and reason",
        "`controlKind` and surface",
        "Sources and evidence",
        "Executable oracle",
        "probabilistic behavioral trial",
        "Approval",
        "Claude: `supported` or `unsupported`",
        "Codex: `supported` or `unsupported`",
        "Cursor: `supported` or `unsupported`",
      ],
    ],
  ] as const;

const isRecord = (value: unknown): value is Readonly<Record<string, unknown>> =>
  typeof value === "object" && value !== null;

const parseEvals = (value: unknown): EvalSources => {
  if (!isRecord(value)) {
    return { queries: [] };
  }
  const queryList = value.queries;
  const queries = Array.isArray(queryList)
    ? queryList.flatMap((query) => {
        if (
          !isRecord(query) ||
          typeof query.query !== "string" ||
          typeof query.should_activate !== "boolean"
        ) {
          return [];
        }
        return [{ query: query.query, shouldActivate: query.should_activate }];
      })
    : [];
  return { queries, version: value.version };
};

const evalFindings = (evals: EvalSources): readonly string[] => [
  ...(evals.version === "1.1" ? [] : ["evals use version 1.1"]),
  ...(evals.queries.some(
    ({ query, shouldActivate }) =>
      shouldActivate && query.includes("invariant existant"),
  )
    ? []
    : ["evals contain a registry-link case"]),
  ...(evals.queries.some(
    ({ query, shouldActivate }) =>
      !shouldActivate && query.includes("stylistique"),
  )
    ? []
    : ["evals reject an isolated stylistic preference"]),
];

const validateHarnessReflectionContract = (
  sources: HarnessReflectionSources,
): readonly string[] => {
  const requirements = sourceRequirements(sources);
  return [
    ...requirements.flatMap(([message, source, terms]) =>
      containsAll(source, terms) ? [] : [message],
    ),
    ...evalFindings(parseEvals(sources.evals)),
  ];
};

test("routes factual PR evidence through the named registry", async () => {
  const sources = await loadHarnessReflectionSources(repositoryRoot);
  expect(validateHarnessReflectionContract(sources)).toEqual([]);
});
