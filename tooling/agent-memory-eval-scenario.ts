import { readFile } from "node:fs/promises";

type EvaluationScenario = Readonly<{
  capabilities: readonly string[];
  id: string;
  prompt: string;
  followUpPrompt?: string;
  unrelatedPrompt?: string;
}>;

type EvaluationScenarios = Readonly<{
  scenarios: readonly EvaluationScenario[];
  version: 1;
}>;

async function loadEvaluationScenarios(path: string): Promise<EvaluationScenarios> {
  const parsed: unknown = JSON.parse(await readFile(path, "utf8"));
  if (!isRecord(parsed) || parsed.version !== 1 || !Array.isArray(parsed.scenarios)) {
    throw new Error("invalid scenario contract version");
  }
  const scenarios = parsed.scenarios.map(parseScenario);
  if (scenarios.length === 0 || new Set(scenarios.map(({ id }) => id)).size !== scenarios.length) {
    throw new Error("scenario contract requires unique scenarios");
  }
  return { scenarios, version: 1 };
}

function scenarioById(contract: EvaluationScenarios, id: string): EvaluationScenario {
  const scenario = contract.scenarios.find((candidate) => candidate.id === id);
  if (scenario === undefined) throw new Error(`missing scenario: ${id}`);
  return scenario;
}

function parseScenario(value: unknown, index: number): EvaluationScenario {
  if (!isRecord(value)) throw new Error(`invalid scenario ${index + 1}`);
  const {
    capabilities,
    follow_up_prompt: followUpPrompt,
    id,
    prompt,
    unrelated_prompt: unrelatedPrompt,
  } = value;
  if (
    typeof id !== "string" ||
    id.length === 0 ||
    typeof prompt !== "string" ||
    prompt.length === 0 ||
    !Array.isArray(capabilities) ||
    capabilities.length === 0 ||
    !capabilities.every((capability) => typeof capability === "string" && capability.length > 0) ||
    (unrelatedPrompt !== undefined &&
      (typeof unrelatedPrompt !== "string" || unrelatedPrompt.length === 0)) ||
    (followUpPrompt !== undefined &&
      (typeof followUpPrompt !== "string" || followUpPrompt.length === 0))
  ) {
    throw new Error(`invalid scenario ${index + 1}`);
  }
  return {
    capabilities,
    ...(followUpPrompt === undefined ? {} : { followUpPrompt }),
    id,
    prompt,
    ...(unrelatedPrompt === undefined ? {} : { unrelatedPrompt }),
  };
}

function isRecord(value: unknown): value is Readonly<Record<string, unknown>> {
  return value !== null && typeof value === "object" && !Array.isArray(value);
}

export { loadEvaluationScenarios, scenarioById };
export type { EvaluationScenario, EvaluationScenarios };
