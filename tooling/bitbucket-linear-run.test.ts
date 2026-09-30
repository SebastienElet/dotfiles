import {
  type Issue,
  type PullRequest,
  type Team,
  configSchema,
} from "./bitbucket-linear-schema.ts";
import { expect, test } from "bun:test";
import type { Providers } from "./bitbucket-linear-providers.ts";
import { synchronize } from "./bitbucket-linear-run.ts";

const config = configSchema.parse({
  bktContext: "work",
  linearWorkspace: "work",
  repositories: [
    {
      workspace: "acme",
      repository: "app",
      teamKey: "ENG",
      teamId: "00000000-0000-4000-8000-000000000001",
    },
  ],
});
const team: Team = {
  id: "00000000-0000-4000-8000-000000000001",
  key: "ENG",
  states: [
    {
      id: "00000000-0000-4000-8000-000000000002",
      name: "Done",
      type: "completed",
    },
  ],
};
const pr: PullRequest = {
  id: 1,
  title: "ENG-1",
  description: "",
  branch: "ENG-1-fix",
  state: "MERGED",
  url: "https://bitbucket.org/acme/app/pull-requests/1",
  repository: "acme/app",
  sourceRepository: "acme/app",
  sourceRepositoryId: "source",
  destinationRepositoryId: "dest",
  authorId: "author",
  teamKey: team.key,
  teamId: team.id,
};
const issue: Issue = {
  id: "00000000-0000-4000-8000-000000000003",
  identifier: "ENG-1",
  teamId: team.id,
  state: {
    id: "00000000-0000-4000-8000-000000000004",
    name: "Todo",
    type: "unstarted",
  },
  archivedAt: null,
  attachments: [],
};

function storedIssue(issues: readonly Issue[], identifier: string): Issue {
  const found = issues.find(
    (value) => value.identifier === identifier || value.id === identifier,
  );
  if (found === undefined) {
    throw new Error("missing");
  }
  return found;
}
const initialIssues: readonly (readonly [string, Issue])[] = [
  ["ENG-1", issue],
  [
    "ENG-2",
    {
      ...issue,
      id: "00000000-0000-4000-8000-000000000005",
      identifier: "ENG-2",
    },
  ],
];
function memory(prs: readonly PullRequest[] = [pr]): Readonly<{
  providers: Providers;
  events: string[];
  issues: Map<string, Issue>;
}> {
  const events: string[] = [];
  const issues = new Map<string, Issue>(initialIssues);
  const providers: Providers = {
    identity: () => ({
      bitbucketUserId: "author",
      linearUserId: "user",
      linearOrganizationId: "org",
    }),
    team: () => team,
    pullRequests: () => prs,
    issue: (id) => storedIssue([...issues.values()], id),
    attach: (id, action) => {
      events.push(`attach:${id}`);
      const current = storedIssue([...issues.values()], id);
      issues.set(current.identifier, {
        ...current,
        attachments: [
          {
            id: "link",
            url: action.url,
            title: action.title,
            subtitle: action.subtitle,
          },
        ],
      });
    },
    complete: (id, stateId) => {
      events.push(`complete:${id}`);
      const current = storedIssue([...issues.values()], id);
      issues.set(current.identifier, {
        ...current,
        state: { id: stateId, name: "Done", type: "completed" },
      });
    },
  };
  return { providers, events, issues };
}

test("inspection exposes the complete plan without provider mutations", async () => {
  const state = memory();
  const result = await synchronize(config, state.providers, {
    apply: false,
    announce: (plan) => state.events.push(`plan:${plan.issues.length}`),
  });
  expect(state.events).toEqual(["plan:1"]);
  expect(result.exitCode).toBe(0);
  expect(result.plan.issues[0]?.completeStateId).toBe(team.states[0]?.id);
});

test("apply publishes complete plan before linking then closing and replays without duplication", async () => {
  const state = memory();
  const announce = (): number => state.events.push("plan");
  expect(
    await synchronize(config, state.providers, {
      apply: true,
      announce,
    }),
  ).toHaveProperty("exitCode", 0);
  expect(state.events).toEqual([
    "plan",
    `attach:${issue.id}`,
    `complete:${issue.id}`,
  ]);
  state.events.length = 0;
  expect(
    await synchronize(config, state.providers, {
      apply: true,
      announce,
    }),
  ).toHaveProperty("exitCode", 0);
  expect(state.events).toEqual(["plan"]);
});

test("partial attachment failure prevents its closure and preserves independent issue progress", async () => {
  const state = memory([
    pr,
    { ...pr, id: 2, title: "ENG-2", branch: "ENG-2-fix", url: `${pr.url}2` },
  ]);
  const original = state.providers.attach;
  const providers = {
    ...state.providers,
    attach: async (
      id: string,
      action: Parameters<Providers["attach"]>[1],
    ): Promise<void> => {
      if (id === issue.id) {
        throw new Error("failure");
      }
      await original(id, action);
    },
  };
  expect(
    await synchronize(config, providers, {
      apply: true,
      announce: () => null,
    }),
  ).toHaveProperty("exitCode", 1);
  expect(state.events).not.toContain(`complete:${issue.id}`);
  expect(state.events).toContain(
    "complete:00000000-0000-4000-8000-000000000005",
  );
  expect(
    await synchronize(config, state.providers, {
      apply: true,
      announce: () => null,
    }),
  ).toHaveProperty("exitCode", 0);
  expect(state.issues.get("ENG-1")?.state.type).toBe("completed");
});

test("ambiguous PR referencing an issue prevents closure from another merged PR", async () => {
  const state = memory([
    pr,
    { ...pr, id: 2, title: "ENG-1 ENG-2", state: "OPEN", url: `${pr.url}2` },
  ]);
  const result = await synchronize(config, state.providers, {
    apply: true,
    announce: () => null,
  });
  expect(result.exitCode).toBe(1);
  expect(state.events).not.toContain(`complete:${issue.id}`);
});

test("incomplete repository blocks closure for the same team across repositories", async () => {
  const state = memory();
  const expanded = {
    ...config,
    repositories: [
      ...config.repositories,
      {
        workspace: "acme",
        teamKey: "ENG",
        teamId: team.id,
        repository: "other",
      },
    ],
  };
  const providers = {
    ...state.providers,
    pullRequests: (
      repository: Readonly<(typeof expanded.repositories)[number]>,
    ): readonly PullRequest[] => {
      if (repository.repository === "other") {
        throw new Error("intermediate-page");
      }
      return [pr];
    },
  };
  expect(
    await synchronize(expanded, providers, {
      apply: true,
      announce: () => null,
    }),
  ).toHaveProperty("exitCode", 1);
  expect(state.events).not.toContain(`complete:${issue.id}`);
});

test("wrong team is visible and never mutates the issue", async () => {
  const state = memory();
  const providers = {
    ...state.providers,
    issue: (): Issue => ({ ...issue, teamId: "other" }),
  };
  expect(
    await synchronize(config, providers, {
      apply: true,
      announce: () => null,
    }),
  ).toHaveProperty("exitCode", 1);
  expect(state.events).toEqual([]);
});

test("failed pre-write plan announcement prevents every mutation", () => {
  const state = memory();
  expect(
    synchronize(config, state.providers, {
      apply: true,
      announce: () => {
        throw new Error("output unavailable");
      },
    }),
  ).rejects.toThrow();
  expect(state.events).toEqual([]);
});

test("an attachment removed after planning prevents completion on the fresh issue read", async () => {
  const state = memory();
  const attachment = {
    id: "link",
    url: pr.url,
    title: pr.title,
    subtitle: pr.state,
  };
  state.issues.set(issue.identifier, { ...issue, attachments: [attachment] });
  const result = await synchronize(config, state.providers, {
    apply: true,
    announce: () =>
      state.issues.set(issue.identifier, { ...issue, attachments: [] }),
  });
  expect(result.exitCode).toBe(1);
  expect(state.events).not.toContain(`complete:${issue.id}`);
  expect(state.issues.get(issue.identifier)?.state.type).toBe("unstarted");
});
