import { resolve } from "node:path";

type HarnessReflectionSources = Readonly<{
  evals: unknown;
  reference: string;
  skill: string;
}>;
type EvalQuery = Readonly<{ query: string; shouldActivate: boolean }>;
type EvalSources = Readonly<{
  queries: readonly EvalQuery[];
  valid: boolean;
  version?: unknown;
}>;

const isRecord = (value: unknown): value is Readonly<Record<string, unknown>> =>
  typeof value === "object" && value !== null;

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

const normalizeWhitespace = (source: string): string =>
  source.replaceAll(/\s+/gu, " ");

const ordered = (source: string, terms: readonly string[]): boolean =>
  terms.reduce((lastIndex, term) => {
    const index = source.indexOf(term, lastIndex + 1);
    return index === -1 ? Number.POSITIVE_INFINITY : index;
  }, -1) !== Number.POSITIVE_INFINITY;

const skillFindings = (skill: string): readonly string[] => {
  const normalizedSkill = normalizeWhitespace(skill);
  const diagnostics = [
    "task-specific",
    "owned-defect",
    "external-transient",
    "missing-capability",
    "harness-gap",
  ];
  const flow = [
    "3. Classify the cause as",
    "5. If the result is not `harness-gap`",
    "6. For `harness-gap`, read",
    "inspect the named registry",
    "return exactly `skip`, `link`, or\n   `propose`",
  ];
  return [
    ...(containsAll(skill, diagnostics)
      ? []
      : ["skill flow preserves the diagnostic class and harness-gap gate"]),
    ...(ordered(skill, flow)
      ? []
      : ["skill flow is ordered from diagnosis to registry decision"]),
    ...(normalizedSkill.includes("reason and next diagnostic action") &&
    normalizedSkill.includes("do not read the reference or registry")
      ? []
      : ["non-harness-gap stops with the diagnostic skip"]),
    ...(containsAll(skill, [
      "pr-feedback",
      "skill-manager",
      "agent-instructions",
    ])
      ? []
      : ["skill preserves factual PR feedback and downstream routing"]),
  ];
};

const referenceFindings = (reference: string): readonly string[] => {
  const normalizedReference = normalizeWhitespace(reference);
  const classes = [
    "not-applied",
    "not-loaded",
    "unknown",
    "blind-spot",
    "judgment",
  ];
  const lifecycleStates = [
    "a candidate with measured or verified verification",
    "a retired record without retirement",
    "unknown replacement",
  ];
  return [
    ...(containsAll(reference, classes)
      ? []
      : ["reference names every registry cause class"]),
    ...(ordered(reference, [
      "Search the registry",
      "Classify the registry cause",
      "Return exactly one decision:",
    ])
      ? []
      : ["reference orders lookup, registry classification, and decision"]),
    ...(reference.includes("Return exactly one decision:")
      ? []
      : ["reference returns exactly skip, link, or propose"]),
    ...(containsAll(normalizedReference, [
      "duplicate record",
      "two distinct PR URLs",
      "valid PR evidence",
      "Missing concrete PR evidence requires `skip`",
      "retiredAt",
      "replacedBy",
      "explicit approval",
      "factual `pr-feedback`",
      "harness/invariants/registry.json",
      "bun tooling/invariant-registry-cli.ts",
      ...lifecycleStates,
    ])
      ? []
      : [
          "reference governs evidence, deduplication, approval, and lifecycle states",
        ]),
  ];
};

const reportFindings = (reference: string): readonly string[] =>
  containsAll(reference, [
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
  ])
    ? []
    : ["reference requires the complete harness-gap decision report"];

const parseEvals = (value: unknown): EvalSources => {
  if (!isRecord(value) || !Array.isArray(value.queries)) {
    return { queries: [], valid: false };
  }
  const queries = value.queries.flatMap((query) =>
    isRecord(query) &&
    typeof query.query === "string" &&
    typeof query.should_activate === "boolean" &&
    typeof query.reason === "string"
      ? [{ query: query.query, shouldActivate: query.should_activate }]
      : [],
  );
  return {
    queries,
    valid:
      typeof value.version === "string" &&
      queries.length === value.queries.length,
    version: value.version,
  };
};

const evalFindings = (evals: EvalSources): readonly string[] => [
  ...(evals.valid ? [] : ["evals have valid structured queries"]),
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
): readonly string[] => [
  ...skillFindings(sources.skill),
  ...referenceFindings(sources.reference),
  ...reportFindings(sources.reference),
  ...evalFindings(parseEvals(sources.evals)),
];

export {
  loadHarnessReflectionSources,
  validateHarnessReflectionContract,
  type HarnessReflectionSources,
};
