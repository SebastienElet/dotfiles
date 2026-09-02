import { resolve } from "node:path";

type HarnessReflectionSources = Readonly<{
  evals: unknown;
  reference: string;
  skill: string;
}>;
type EvalQuery = Readonly<{
  query: string;
  shouldActivate: boolean;
}>;
type EvalSources = Readonly<{
  queries: readonly EvalQuery[];
  slug?: unknown;
  valid: boolean;
  version?: unknown;
}>;

const expectedSets = {
  Decisions: ["skip", "link", "propose"],
  "Registry classes": [
    "not-applied",
    "not-loaded",
    "unknown",
    "blind-spot",
    "judgment",
  ],
  "Control kinds": ["probabilistic", "enforceable"],
  "Probabilistic surfaces": [
    "always-loaded-instruction",
    "conditional-skill",
    "project-local-contract",
  ],
  "Enforceable surfaces": [
    "hook",
    "permission",
    "lint",
    "type",
    "architectural-test",
  ],
} as const;

const linkQuery =
  "Relie ces constats pr-feedback récurrents à l'invariant existant sans le dupliquer";
const stylisticQuery =
  "J'ai une préférence stylistique isolée pour ce nommage, change-le";

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

const normalize = (source: string): string => source.replaceAll(/\s+/gu, " ");

const codeValues = (source: string): readonly string[] => {
  const pattern = /`(?<value>[^`]+)`/gu;
  const values: string[] = [];
  for (
    let match = pattern.exec(source);
    match !== null;
    match = pattern.exec(source)
  ) {
    values.push(match.groups?.value ?? "");
  }
  return values;
};

const sameSet = (
  actual: readonly string[],
  expected: readonly string[],
): boolean =>
  actual.length === expected.length &&
  actual.every((value) => expected.includes(value)) &&
  new Set(actual).size === actual.length;

const tableValue = (source: string, name: string): string | undefined =>
  source
    .split("\n")
    .find((line) => line.startsWith(`| ${name} |`))
    ?.split("|")[2]
    ?.trim();

const setFinding = (name: string): string => {
  if (name === "Decisions") {
    return "guidance closes the decision set";
  }
  if (name === "Registry classes") {
    return "guidance closes the registry-class set";
  }
  return "guidance closes compatible control surfaces";
};

const setFindings = (reference: string): readonly string[] =>
  Object.entries(expectedSets).flatMap(
    ([name, expected]: readonly [string, readonly string[]]) =>
      sameSet(codeValues(tableValue(reference, name) ?? ""), expected)
        ? []
        : [setFinding(name)],
  );

const flowFindings = (skill: string, reference: string): readonly string[] => {
  const section =
    /3\. Classify the cause as[\s\S]*?`harness-gap`/u.exec(skill)?.[0] ?? "";
  const diagnostics = codeValues(section);
  const skillFlow = [
    "3. Classify the cause as",
    "5. If the result is not `harness-gap`",
    "6. For `harness-gap`, read",
    "inspect the named registry",
    "return exactly `skip`, `link`, or",
  ];
  const ordered = skillFlow.every(
    (marker, index) =>
      index === 0 ||
      skill.indexOf(marker) > skill.indexOf(skillFlow[index - 1] ?? ""),
  );
  return [
    ...(sameSet(diagnostics, [
      "task-specific",
      "owned-defect",
      "external-transient",
      "missing-capability",
      "harness-gap",
    ])
      ? []
      : ["skill flow preserves the diagnostic class and harness-gap gate"]),
    ...(ordered &&
    reference.indexOf("Search the registry") <
      reference.indexOf("Classify the registry cause") &&
    reference.indexOf("Classify the registry cause") <
      reference.indexOf("Return exactly one decision:")
      ? []
      : ["skill flow is ordered from diagnosis to registry decision"]),
    ...(normalize(skill).includes("reason and next diagnostic action") &&
    normalize(skill).includes("do not read the reference or registry")
      ? []
      : ["non-harness-gap stops with the diagnostic skip"]),
  ];
};

const semanticFindings = (
  reference: string,
  skill: string,
): readonly string[] => [
  ...(tableValue(reference, "Evidence policy") ===
  "concrete PR URLs required; missing evidence returns `skip`"
    ? []
    : ["reference requires concrete evidence before propose"]),
  ...(tableValue(reference, "CLI report claim") ===
  "CLI accepted snapshot read in execution environment"
    ? []
    : ["reference limits the CLI claim to its execution snapshot"]),
  ...(normalize(skill).includes(
    "accepted the snapshot read in that execution environment",
  ) && !normalize(skill).includes("presenting the change as valid")
    ? []
    : ["skill limits the CLI claim to its execution snapshot"]),
  ...(normalize(reference).includes(
    "a candidate with measured or verified verification",
  ) &&
  normalize(reference).includes("a retired record without retirement") &&
  normalize(reference).includes("unknown replacement")
    ? []
    : ["reference names invalid lifecycle state combinations"]),
  ...(normalize(reference).includes("duplicate record") &&
  normalize(reference).includes("explicit approval") &&
  normalize(reference).includes("factual `pr-feedback`")
    ? []
    : ["reference preserves deduplication, approval, and factual feedback"]),
];

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
    slug: value.skill,
    valid: queries.length === value.queries.length,
    version: value.version,
  };
};

const evalFindings = (evals: EvalSources): readonly string[] => [
  ...(evals.valid ? [] : ["evals have valid structured queries"]),
  ...(evals.slug === "harness-reflection"
    ? []
    : ["evals use the harness-reflection slug"]),
  ...(evals.version === "1.1" ? [] : ["evals use version 1.1"]),
  ...(evals.queries.some(
    ({ query, shouldActivate }) => shouldActivate && query === linkQuery,
  )
    ? []
    : ["evals reject an incorrect registry-link query"]),
  ...(evals.queries.some(
    ({ query, shouldActivate }) => !shouldActivate && query === stylisticQuery,
  )
    ? []
    : ["evals reject an isolated stylistic preference"]),
];

const validateHarnessReflectionContract = (
  sources: HarnessReflectionSources,
): readonly string[] => [
  ...flowFindings(sources.skill, sources.reference),
  ...setFindings(sources.reference),
  ...semanticFindings(sources.reference, sources.skill),
  ...evalFindings(parseEvals(sources.evals)),
];

export {
  loadHarnessReflectionSources,
  validateHarnessReflectionContract,
  type HarnessReflectionSources,
};
