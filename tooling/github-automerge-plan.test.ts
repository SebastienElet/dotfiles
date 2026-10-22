import type { GithubApi, GithubRequest } from "./github-automerge-plan.ts";
import { expect, test } from "bun:test";
import { planGithubAutomerge } from "./github-automerge-plan.ts";
import { rejects } from "node:assert/strict";

function emptyRepositories(
  overrides: Readonly<Record<string, unknown>> = {},
): GithubApi {
  return (request: GithubRequest): Promise<unknown> => {
    if (request.method !== "GET") {
      throw new Error("Planning attempted a write");
    }
    if (Object.hasOwn(overrides, request.endpoint)) {
      return Promise.resolve(overrides[request.endpoint]);
    }
    return Promise.resolve(
      request.endpoint.includes("/rulesets")
        ? [[]]
        : { default_branch: "trunk", allow_auto_merge: false },
    );
  };
}

test("plans required CI before auto-merge for only the eleven requested repositories", async () => {
  const plans = await planGithubAutomerge(emptyRepositories());
  expect(plans.map((plan) => plan.repository)).toEqual([
    "SebastienElet/dotfiles",
    "SebastienElet/dev-metrics",
    "SebastienElet/arnes",
    "SebastienElet/agent-experiments",
    "SebastienElet/agent-measure",
    "SebastienElet/agent-handoff",
    "SebastienElet/herdr-worktree-cleanup",
    "SebastienElet/agent-reach-adapter",
    "SebastienElet/herdr-issue-worktree",
    "SebastienElet/proof-integrity-review",
    "SebastienElet/mst-cli",
  ]);
});

test("adds CI to mst-cli without replacing its existing protection", async () => {
  const plans = await planGithubAutomerge(emptyRepositories());
  const mst = plans.find((plan) => plan.repository.endsWith("/mst-cli"));
  expect(mst).toMatchObject({
    branch: "trunk",
    contexts: ["ci"],
    changes: [
      {
        method: "POST",
        endpoint: "repos/SebastienElet/mst-cli/rulesets",
        body: {
          target: "branch",
          enforcement: "active",
          conditions: {
            ref_name: { include: ["~DEFAULT_BRANCH"], exclude: [] },
          },
          rules: [
            {
              type: "required_status_checks",
              parameters: {
                required_status_checks: [
                  { context: "ci", integration_id: 15_368 },
                ],
                strict_required_status_checks_policy: false,
              },
            },
          ],
        },
      },
      {
        method: "PATCH",
        endpoint: "repos/SebastienElet/mst-cli",
        body: { allow_auto_merge: true },
      },
    ],
  });
  expect(
    plans
      .flatMap((plan) => plan.contexts)
      .some((context) => context.includes("publish")),
  ).toBe(false);
});

const managedEndpoint = "repos/SebastienElet/mst-cli/rulesets";
const managedSummary = { id: 42, name: "Required CI for auto-merge" };
const rulesetListEndpoint = `${managedEndpoint}?includes_parents=false&per_page=100`;
const existingRuleset = {
  ...managedSummary,
  target: "branch",
  enforcement: "disabled",
  bypass_actors: [],
  conditions: { ref_name: { include: ["refs/heads/trunk"], exclude: [] } },
  rules: [
    { type: "non_fast_forward" },
    {
      type: "required_status_checks",
      parameters: {
        strict_required_status_checks_policy: true,
        do_not_enforce_on_create: false,
        required_status_checks: [{ context: "security", integration_id: 7 }],
      },
    },
  ],
};

test("updates the named ruleset from a later page and preserves other rules and checks", async () => {
  const plans = await planGithubAutomerge(
    emptyRepositories({
      [rulesetListEndpoint]: [[{ id: 1, name: "Unrelated" }], [managedSummary]],
      [`${managedEndpoint}/42`]: existingRuleset,
    }),
  );
  expect(plans.at(-1)?.changes[0]).toMatchObject({
    method: "PUT",
    endpoint: `${managedEndpoint}/42`,
    body: {
      enforcement: "active",
      bypass_actors: [],
      conditions: { ref_name: { include: ["~DEFAULT_BRANCH"], exclude: [] } },
      rules: [
        { type: "non_fast_forward" },
        {
          type: "required_status_checks",
          parameters: {
            strict_required_status_checks_policy: true,
            do_not_enforce_on_create: false,
            required_status_checks: [
              { context: "security", integration_id: 7 },
              { context: "ci", integration_id: 15_368 },
            ],
          },
        },
      ],
    },
  });
});

test("a second plan is empty after the first plan has been applied", async () => {
  const state: Record<string, unknown> = {};
  const first = await planGithubAutomerge(emptyRepositories(state));
  for (const plan of first) {
    state[`repos/${plan.repository}`] = {
      default_branch: "trunk",
      allow_auto_merge: true,
    };
    state[
      `repos/${plan.repository}/rulesets?includes_parents=false&per_page=100`
    ] = [[managedSummary]];
    state[`repos/${plan.repository}/rulesets/42`] = plan.changes[0]?.body;
  }
  const second = await planGithubAutomerge(emptyRepositories(state));
  expect(second.flatMap((plan) => plan.changes)).toEqual([]);
});

const conflictingRulesets: readonly Readonly<{
  label: string;
  value: unknown;
}>[] = [
  { label: "wrong target", value: { ...existingRuleset, target: "tag" } },
  { label: "missing rules", value: { ...existingRuleset, rules: null } },
  { label: "wrong name", value: { ...existingRuleset, name: "Renamed" } },
  {
    label: "broader scope",
    value: {
      ...existingRuleset,
      conditions: { ref_name: { include: ["~ALL"], exclude: [] } },
    },
  },
  {
    label: "excluded branch",
    value: {
      ...existingRuleset,
      conditions: {
        ref_name: {
          include: ["~DEFAULT_BRANCH"],
          exclude: ["refs/heads/trunk"],
        },
      },
    },
  },
  {
    label: "duplicate status rules",
    value: {
      ...existingRuleset,
      rules: [existingRuleset.rules[1], existingRuleset.rules[1]],
    },
  },
  {
    label: "invalid parameters",
    value: {
      ...existingRuleset,
      rules: [{ type: "required_status_checks", parameters: {} }],
    },
  },
];
test.each(conflictingRulesets)(
  "rejects a conflicting or malformed managed ruleset: $label",
  async ({ value }) => {
    await rejects(
      planGithubAutomerge(
        emptyRepositories({
          [rulesetListEndpoint]: [[managedSummary]],
          [`${managedEndpoint}/42`]: value,
        }),
      ),
    );
  },
);

test("rejects a malformed ruleset page", async () => {
  await rejects(
    planGithubAutomerge(
      emptyRepositories({
        [rulesetListEndpoint]: [null],
      }),
    ),
  );
});

test("rejects malformed repository metadata", async () => {
  await rejects(
    planGithubAutomerge(
      emptyRepositories({
        "repos/SebastienElet/dotfiles": { default_branch: "" },
      }),
    ),
  );
});

test("preserves HTTP failures rather than interpreting them as an absent ruleset", async () => {
  const unavailable: GithubApi = () =>
    Promise.reject(new Error("gh failed: HTTP 403"));
  await rejects(planGithubAutomerge(unavailable), /HTTP 403/u);
});

test("rejects duplicate managed rulesets", async () => {
  await rejects(
    planGithubAutomerge(
      emptyRepositories({
        [rulesetListEndpoint]: [
          [managedSummary, { ...managedSummary, id: 43 }],
        ],
      }),
    ),
    /multiple/u,
  );
});

test("refuses an existing managed ruleset that requires a publish job", async () => {
  await rejects(
    planGithubAutomerge(
      emptyRepositories({
        [rulesetListEndpoint]: [[managedSummary]],
        [`${managedEndpoint}/42`]: {
          ...existingRuleset,
          rules: [
            {
              type: "required_status_checks",
              parameters: {
                strict_required_status_checks_policy: false,
                required_status_checks: [{ context: "build / publish" }],
              },
            },
          ],
        },
      }),
    ),
    /publish/u,
  );
});

test("adds required checks when a managed ruleset has only unrelated rules", async () => {
  const plans = await planGithubAutomerge(
    emptyRepositories({
      [rulesetListEndpoint]: [[managedSummary]],
      [`${managedEndpoint}/42`]: {
        ...existingRuleset,
        rules: [{ type: "non_fast_forward" }],
      },
    }),
  );
  expect(plans.at(-1)?.changes[0]?.body).toMatchObject({
    rules: [
      { type: "non_fast_forward" },
      {
        type: "required_status_checks",
        parameters: {
          required_status_checks: [{ context: "ci", integration_id: 15_368 }],
        },
      },
    ],
  });
});
