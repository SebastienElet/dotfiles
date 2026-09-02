import { harnessReflectionContractSchema } from "./harness-reflection-contract-schema.ts";
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
type ContractBlock = Readonly<{
  json?: string;
  wrapper?: string;
}>;

const expectedReferenceWrapper = `# Invariant Registry

Use this reference only after the diagnostic class is \`harness-gap\`. The canonical registry remains
the source of truth for named invariant records. The JSON block below is the sole authority for the
reflection workflow; execute its \`workflowOrder\` literally and reject prose or data that contradicts
any closed value.

## Authoritative workflow contract

\`\`\`json
{{contract}}
\`\`\`
`;
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

const codeValues = (source: string): readonly string[] => {
  const values: string[] = [];
  const pattern = /`(?<value>[^`]+)`/gu;
  for (
    let match = pattern.exec(source);
    match !== null;
    match = pattern.exec(source)
  ) {
    values.push(match.groups?.value ?? "");
  }
  return values;
};

const sameValues = (
  actual: readonly string[],
  expected: readonly string[],
): boolean =>
  actual.length === expected.length &&
  actual.every((value, index) => value === expected[index]);

const normalizeWhitespace = (source: string): string =>
  source.replaceAll(/\s+/gu, " ");

const extractContractBlock = (reference: string): ContractBlock => {
  const matches = [...reference.matchAll(/```json\n(?<json>[\s\S]*?)\n```/gu)];
  if (matches.length !== 1) {
    return {};
  }
  const [match] = matches;
  const json = match?.groups?.json;
  if (match === undefined || json === undefined) {
    return {};
  }
  return {
    json,
    wrapper: reference.replace(match[0], "```json\n{{contract}}\n```"),
  };
};

const authoritativeContractFindings = (
  reference: string,
): readonly string[] => {
  const block = extractContractBlock(reference);
  if (block.json === undefined) {
    return ["reference has one authoritative JSON contract"];
  }
  let parsed: unknown = undefined;
  try {
    parsed = JSON.parse(block.json);
  } catch {
    return ["authoritative contract is valid JSON"];
  }
  return [
    ...(block.wrapper === expectedReferenceWrapper
      ? []
      : ["reference contains no parallel or contradictory authority"]),
    ...(harnessReflectionContractSchema.safeParse(parsed).success
      ? []
      : ["authoritative contract preserves exact workflow invariants"]),
  ];
};

const skillFlowFindings = (skill: string): readonly string[] => {
  const diagnosticSection =
    /3\. Classify the cause as[\s\S]*?or `harness-gap`\./u.exec(skill)?.[0] ??
    "";
  const expectedDiagnostics = [
    "task-specific",
    "owned-defect",
    "external-transient",
    "missing-capability",
    "harness-gap",
  ];
  const markers = [
    "2. Preserve the smallest useful evidence:",
    "3. Classify the cause as",
    "5. If the result is not `harness-gap`",
    "6. For `harness-gap`, read",
    "execute its single authoritative JSON contract in the declared order",
  ];
  const ordered = markers.every(
    (marker, index) =>
      index === 0 ||
      skill.indexOf(marker) > skill.indexOf(markers[index - 1] ?? ""),
  );
  return [
    ...(sameValues(codeValues(diagnosticSection), expectedDiagnostics)
      ? []
      : ["skill preserves the diagnostic classes before the harness-gap gate"]),
    ...(ordered ? [] : ["skill consumes the authoritative contract in order"]),
    ...(normalizeWhitespace(skill).includes(
      "reason and next diagnostic action",
    ) && skill.includes("do not read the reference or registry")
      ? []
      : ["non-harness-gap stops before registry access"]),
    ...(skill.includes(
      "[references/invariant-registry.md](references/invariant-registry.md)",
    )
      ? []
      : ["skill routes harness-gap to the registry reference"]),
  ];
};

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

const exactQuery = (
  evals: EvalSources,
  query: string,
  shouldActivate: boolean,
): boolean =>
  evals.queries.filter(
    (candidate) =>
      candidate.query === query && candidate.shouldActivate === shouldActivate,
  ).length === 1;

const evalFindings = (evals: EvalSources): readonly string[] => [
  ...(evals.valid ? [] : ["evals have valid structured queries"]),
  ...(evals.slug === "harness-reflection"
    ? []
    : ["evals use the harness-reflection slug"]),
  ...(evals.version === "1.1" ? [] : ["evals use version 1.1"]),
  ...(exactQuery(evals, linkQuery, true)
    ? []
    : ["evals preserve the exact positive registry-link query"]),
  ...(exactQuery(evals, stylisticQuery, false)
    ? []
    : ["evals preserve the exact negative stylistic query"]),
];

const validateHarnessReflectionContract = (
  sources: HarnessReflectionSources,
): readonly string[] => [
  ...skillFlowFindings(sources.skill),
  ...authoritativeContractFindings(sources.reference),
  ...evalFindings(parseEvals(sources.evals)),
];

export {
  loadHarnessReflectionSources,
  validateHarnessReflectionContract,
  type HarnessReflectionSources,
};
