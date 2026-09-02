import {
  type HarnessReflectionContract,
  harnessReflectionContractSchema,
} from "./harness-reflection-contract-schema.ts";
import { duplicateJsonObjectKeys } from "./harness-reflection-contract-json-keys.ts";
import { promotionResultsFindings } from "./harness-reflection-promotion-results.ts";
import { resolve } from "node:path";

type HarnessReflectionSources = Readonly<{
  evals: unknown;
  promotionResults: unknown;
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

## Authoritative workflow contract

\`\`\`json
{{contract}}
\`\`\`
`;
const expectedSkillSurfaceDigest =
  "9d99a33527aa814ef92cbc009d481626306e40a892e8fb1131b4701365350e97";
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
    promotionResults: await Bun.file(
      resolve(skillRoot, "evals/promotion-workflow-results.json"),
    ).json(),
    reference: await Bun.file(
      resolve(skillRoot, "references/invariant-registry.md"),
    ).text(),
    skill: await Bun.file(resolve(skillRoot, "SKILL.md")).text(),
  };
};

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
  if (duplicateJsonObjectKeys(block.json).length > 0) {
    return ["authoritative contract has unique JSON object keys"];
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

const parseHarnessReflectionContract = (
  reference: string,
): HarnessReflectionContract => {
  const block = extractContractBlock(reference);
  if (
    block.json === undefined ||
    duplicateJsonObjectKeys(block.json).length > 0
  ) {
    throw new TypeError("invalid harness reflection contract");
  }
  return harnessReflectionContractSchema.parse(JSON.parse(block.json));
};

const skillSurfaceFindings = (skill: string): readonly string[] =>
  new Bun.CryptoHasher("sha256").update(skill).digest("hex") ===
  expectedSkillSurfaceDigest
    ? []
    : ["skill contains only the closed router surface"];

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

const uniqueEvalQueries = (evals: EvalSources): boolean => {
  const queryTexts = evals.queries.map(({ query }) => query);
  return new Set(queryTexts).size === queryTexts.length;
};

const evalFindings = (evals: EvalSources): readonly string[] => [
  ...(evals.valid ? [] : ["evals have valid structured queries"]),
  ...(evals.slug === "harness-reflection"
    ? []
    : ["evals use the harness-reflection slug"]),
  ...(evals.version === "1.1" ? [] : ["evals use version 1.1"]),
  ...(uniqueEvalQueries(evals)
    ? []
    : ["eval queries have unique text and polarity"]),
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
  ...skillSurfaceFindings(sources.skill),
  ...authoritativeContractFindings(sources.reference),
  ...evalFindings(parseEvals(sources.evals)),
  ...promotionResultsFindings(
    sources.promotionResults,
    sources.skill,
    sources.reference,
  ),
];

export {
  loadHarnessReflectionSources,
  parseHarnessReflectionContract,
  validateHarnessReflectionContract,
  type HarnessReflectionSources,
};
