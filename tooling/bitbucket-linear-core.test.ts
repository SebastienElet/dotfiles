import type { Issue, PullRequest, Team } from "./bitbucket-linear-schema.ts";
import { correlate, planIssue } from "./bitbucket-linear-core.ts";
import { expect, test } from "bun:test";

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
const issue: Issue = {
  id: "00000000-0000-4000-8000-000000000003",
  identifier: "ENG-12",
  teamId: team.id,
  state: {
    id: "00000000-0000-4000-8000-000000000004",
    name: "Todo",
    type: "unstarted",
  },
  archivedAt: null,
  attachments: [],
};
const pr: PullRequest = {
  id: 12,
  title: "Fix eng-12",
  description: "",
  state: "MERGED",
  branch: "eng-12-fix",
  url: "https://bitbucket.org/acme/app/pull-requests/12",
  repository: "acme/app",
  sourceRepository: "acme/app",
  sourceRepositoryId: "source",
  destinationRepositoryId: "dest",
  authorId: "author",
  teamKey: "ENG",
  teamId: team.id,
};

test("correlation normalizes and deduplicates all three signals with lexical boundaries", () => {
  expect(
    correlate(
      { ...pr, title: "xENG-9 ENG-12x eng-12", description: "ENG-12" },
      ["ENG"],
    ),
  ).toEqual(["ENG-12"]);
  expect(
    correlate({ ...pr, title: "ENG-9", description: "ENG-13" }, ["ENG"]),
  ).toEqual(["ENG-12", "ENG-13", "ENG-9"]);
});

test("merged work plans attachment before completion using the team's completed state", () => {
  const plan = planIssue(issue, {
    team,
    pullRequests: [pr],
    complete: true,
  });
  expect(plan.attachments).toHaveLength(1);
  expect(plan.completeStateId).toBe("00000000-0000-4000-8000-000000000002");
});

test.each(["OPEN", "DECLINED"] as const)(
  "%s alone never completes",
  (state) => {
    expect(
      planIssue(issue, {
        team,
        pullRequests: [{ ...pr, state }],
        complete: true,
      }).completeStateId,
    ).toBeNull();
  },
);

test("any open PR across configured repositories prevents closure", () => {
  expect(
    planIssue(issue, {
      team,
      pullRequests: [pr, { ...pr, id: 13, state: "OPEN", url: `${pr.url}3` }],
      complete: true,
    }).completeStateId,
  ).toBeNull();
});

test("incomplete inventory cannot close even a merged issue", () => {
  expect(
    planIssue(issue, { team, pullRequests: [pr], complete: false })
      .completeStateId,
  ).toBeNull();
});

test.each(["completed", "canceled"] as const)(
  "%s issues never move",
  (type) => {
    expect(
      planIssue(
        { ...issue, state: { ...issue.state, type } },
        { team, pullRequests: [pr], complete: true },
      ).completeStateId,
    ).toBeNull();
  },
);

test("missing and ambiguous completed states block closure without blocking attachment repair", () => {
  for (const states of [
    [],
    [
      {
        id: "00000000-0000-4000-8000-000000000002",
        name: "Done",
        type: "completed" as const,
      },
      { id: issue.id, name: "Other", type: "completed" as const },
    ],
  ]) {
    const plan = planIssue(issue, {
      team: { ...team, states },
      pullRequests: [pr],
      complete: true,
    });
    expect(plan.completeStateId).toBeNull();
    expect(plan.problems).toContain("completed-state-unresolved");
    expect(plan.attachments).toHaveLength(1);
  }
});

test("attachment title/state satisfied already does not create another action", () => {
  const current = {
    ...issue,
    attachments: [
      { id: "link", url: pr.url, title: pr.title, subtitle: pr.state },
    ],
  };
  expect(
    planIssue(current, { team, pullRequests: [pr], complete: true })
      .attachments,
  ).toEqual([]);
});

test("duplicate existing attachments block all actions for their issue", () => {
  const attachment = {
    id: "link",
    url: pr.url,
    title: pr.title,
    subtitle: pr.state,
  };
  const plan = planIssue(
    { ...issue, attachments: [attachment, { ...attachment, id: "other" }] },
    { team, pullRequests: [pr], complete: true },
  );
  expect(plan.problems).toContain("duplicate-attachment");
  expect(plan.attachments).toEqual([]);
  expect(plan.completeStateId).toBeNull();
});

export { issue, pr, team };
