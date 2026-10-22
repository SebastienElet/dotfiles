import { isDeepStrictEqual } from "node:util";
import { z } from "zod";

const githubActionsIntegration = 15_368;
const rulesetName = "Required CI for auto-merge";
const platforms = ["ubuntu-24.04", "macos-26"];
const nativePlatforms = [
  "ubuntu-24.04, linux-x64-gnu",
  "macos-26, darwin-arm64",
];
const matrix = (job: string, targets: readonly string[]): string[] =>
  targets.map((target) => `${job} (${target})`);

const repositories = [
  {
    name: "dotfiles",
    contexts: [
      "Lua required",
      "Shell required",
      "Text required",
      "Static required",
      "Arnes required",
      "Deployment required",
      "Fish required",
      "Harness required",
      "TypeScript required",
      "Smoke required",
      "Secret scan",
    ],
  },
  { name: "dev-metrics", contexts: ["check"] },
  { name: "arnes", contexts: matrix("test", platforms) },
  { name: "agent-experiments", contexts: matrix("test", platforms) },
  { name: "agent-measure", contexts: matrix("test", platforms) },
  { name: "agent-handoff", contexts: matrix("test", nativePlatforms) },
  { name: "herdr-worktree-cleanup", contexts: matrix("typescript", platforms) },
  {
    name: "agent-reach-adapter",
    contexts: [...matrix("typescript", platforms), "gitleaks"],
  },
  {
    name: "herdr-issue-worktree",
    contexts: [...matrix("rust", platforms), "typescript"],
  },
  {
    name: "proof-integrity-review",
    contexts: matrix("proof-integrity", nativePlatforms),
  },
  { name: "mst-cli", contexts: ["ci"] },
];

const repositorySchema = z.object({
  default_branch: z.string().min(1),
  allow_auto_merge: z.boolean(),
});
const rulesetSummarySchema = z.object({
  id: z.number().int().positive(),
  name: z.string().min(1),
});
const rulesetPagesSchema = z.array(z.array(rulesetSummarySchema));
const statusParametersSchema = z
  .looseObject({
    strict_required_status_checks_policy: z.boolean(),
    required_status_checks: z
      .array(
        z
          .looseObject({
            context: z.string().min(1),
            integration_id: z.number().int().nullable().optional(),
          })
          .readonly(),
      )
      .readonly(),
  })
  .readonly();
const rulesetSchema = z
  .object({
    name: z.string().min(1),
    target: z.literal("branch"),
    enforcement: z.enum(["active", "disabled", "evaluate"]),
    bypass_actors: z
      .array(z.record(z.string(), z.unknown()).readonly())
      .readonly(),
    conditions: z
      .object({
        ref_name: z
          .object({
            include: z.array(z.string()).readonly(),
            exclude: z.array(z.string()).readonly(),
          })
          .readonly(),
      })
      .readonly(),
    rules: z
      .array(
        z
          .looseObject({
            type: z.string().min(1),
            parameters: z.unknown().optional(),
          })
          .readonly(),
      )
      .readonly(),
  })
  .readonly();
type Ruleset = z.infer<typeof rulesetSchema>;
type StatusCheck = z.infer<
  typeof statusParametersSchema
>["required_status_checks"][number];

type GithubRequest = Readonly<{
  method: "GET" | "POST" | "PUT" | "PATCH";
  endpoint: string;
  body?: unknown;
  paginate?: boolean;
}>;

type GithubApi = (request: GithubRequest) => Promise<unknown>;

type RepositoryPlan = Readonly<{
  repository: string;
  branch: string;
  contexts: readonly string[];
  changes: readonly GithubRequest[];
}>;

function newRuleset(contexts: readonly string[]): Ruleset {
  return {
    name: rulesetName,
    target: "branch",
    enforcement: "active",
    bypass_actors: [],
    conditions: { ref_name: { include: ["~DEFAULT_BRANCH"], exclude: [] } },
    rules: [
      {
        type: "required_status_checks",
        parameters: {
          strict_required_status_checks_policy: false,
          required_status_checks: contexts.map((context) => ({
            context,
            integration_id: githubActionsIntegration,
          })),
        },
      },
    ],
  };
}

function updateRuleset(
  current: Ruleset,
  contexts: readonly string[],
  branch: string,
): Ruleset {
  const scope = current.conditions.ref_name;
  if (
    scope.exclude.length > 0 ||
    scope.include.length !== 1 ||
    !["~DEFAULT_BRANCH", `refs/heads/${branch}`].includes(
      scope.include[0] ?? "",
    )
  ) {
    throw new Error("Managed ruleset must target only the default branch");
  }
  const statusRules = current.rules.filter(
    (rule) => rule.type === "required_status_checks",
  );
  if (statusRules.length > 1) {
    throw new Error(
      "Managed ruleset has multiple required_status_checks rules",
    );
  }
  const [previous] = statusRules;
  const requiredRule = mergeStatusChecks(previous, contexts);
  return {
    ...current,
    enforcement: "active",
    conditions: { ref_name: { include: ["~DEFAULT_BRANCH"], exclude: [] } },
    rules:
      previous === undefined
        ? [...current.rules, requiredRule]
        : current.rules.map((rule) =>
            rule === previous ? requiredRule : rule,
          ),
  };
}

function pinCheckIntegration(check: StatusCheck): StatusCheck {
  return { ...check, integration_id: githubActionsIntegration };
}

function mergeStatusChecks(
  previous: Ruleset["rules"][number] | undefined,
  contexts: readonly string[],
): Ruleset["rules"][number] {
  const parameters =
    previous === undefined
      ? {
          strict_required_status_checks_policy: false,
          required_status_checks: [],
        }
      : statusParametersSchema.parse(previous.parameters);
  const existingChecks = parameters.required_status_checks;
  if (existingChecks.some((check) => /\bpublish\b/iu.test(check.context))) {
    throw new Error("Refusing managed ruleset with a required publish job");
  }
  const checks = existingChecks.map((check) =>
    contexts.includes(check.context) ? pinCheckIntegration(check) : check,
  );
  for (const context of contexts) {
    if (!checks.some((check) => check.context === context)) {
      checks.push({ context, integration_id: githubActionsIntegration });
    }
  }
  return {
    ...previous,
    type: "required_status_checks",
    parameters: { ...parameters, required_status_checks: checks },
  };
}

async function planRuleset(
  api: GithubApi,
  options: Readonly<{
    endpoint: string;
    contexts: readonly string[];
    branch: string;
  }>,
): Promise<GithubRequest[]> {
  const { endpoint, contexts, branch } = options;
  const pages = rulesetPagesSchema.parse(
    await api({
      method: "GET",
      endpoint: `${endpoint}/rulesets?includes_parents=false&per_page=100`,
      paginate: true,
    }),
  );
  const matches = pages
    .flat()
    .filter((ruleset) => ruleset.name === rulesetName);
  if (matches.length > 1) {
    throw new Error(`${endpoint}: multiple managed rulesets`);
  }
  const [match] = matches;
  if (match === undefined) {
    return [
      {
        method: "POST",
        endpoint: `${endpoint}/rulesets`,
        body: newRuleset(contexts),
      },
    ];
  }
  const rulesetEndpoint = `${endpoint}/rulesets/${match.id}`;
  const current = rulesetSchema.parse(
    await api({ method: "GET", endpoint: rulesetEndpoint }),
  );
  if (current.name !== rulesetName) {
    throw new Error(`${rulesetEndpoint}: managed ruleset name changed`);
  }
  const desired = updateRuleset(current, contexts, branch);
  return isDeepStrictEqual(current, desired)
    ? []
    : [{ method: "PUT", endpoint: rulesetEndpoint, body: desired }];
}

async function planGithubAutomerge(api: GithubApi): Promise<RepositoryPlan[]> {
  const plans: RepositoryPlan[] = [];
  for (const { name, contexts } of repositories) {
    const repository = `SebastienElet/${name}`;
    const endpoint = `repos/${repository}`;
    const state = repositorySchema.parse(
      await api({ method: "GET", endpoint }),
    );
    const changes = await planRuleset(api, {
      endpoint,
      contexts,
      branch: state.default_branch,
    });
    if (!state.allow_auto_merge) {
      changes.push({
        method: "PATCH",
        endpoint,
        body: { allow_auto_merge: true },
      });
    }
    plans.push({ repository, branch: state.default_branch, contexts, changes });
  }
  return plans;
}

export { planGithubAutomerge };
export type { GithubApi, GithubRequest, RepositoryPlan };
