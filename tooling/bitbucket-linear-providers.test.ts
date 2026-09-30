import { expect, test } from "bun:test";
import { configSchema } from "./bitbucket-linear-schema.ts";
import { createProviders } from "./bitbucket-linear-providers.ts";
import { z } from "zod";

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
const [configuredRepository] = config.repositories;
if (configuredRepository === undefined) {
  throw new Error("missing fixture repository");
}
const repository = {
  uuid: "{00000000-0000-4000-8000-000000000011}",
  full_name: "acme/app",
};
const authorId = "{00000000-0000-4000-8000-000000000012}";
const otherAuthorId = "{00000000-0000-4000-8000-000000000013}";
const rawPr = {
  id: 4,
  title: "ENG-12",
  description: "",
  state: "MERGED",
  author: { uuid: authorId },
  source: { branch: { name: "ENG-12-fix" }, repository },
  destination: { branch: { name: "main" }, repository },
  links: { html: { href: "https://bitbucket.org/acme/app/pull-requests/4" } },
};

test("provider collects every opaque Bitbucket page and every author", async () => {
  const paths: string[] = [];
  const providers = createProviders(config, (_name, args) => {
    const path = args[1] ?? "";
    paths.push(path);
    if (!path.includes("pullrequests")) {
      return repository;
    }
    if (path.includes("cursor=")) {
      return {
        values: [
          {
            ...rawPr,
            id: 5,
            author: { uuid: otherAuthorId },
            links: {
              html: { href: "https://bitbucket.org/acme/app/pull-requests/5" },
            },
          },
        ],
      };
    }
    return {
      values: [rawPr],
      next: "https://api.bitbucket.org/2.0/repositories/acme/app/pullrequests?cursor=opaque",
    };
  });
  const collected = await providers.pullRequests(configuredRepository);
  expect(collected.map((pr) => pr.authorId)).toEqual([authorId, otherAuthorId]);
  expect(paths.at(-1)).toContain("cursor=opaque");
});

test.each([
  { values: [rawPr], next: null },
  { values: [rawPr], next: "https://evil.test/collect" },
  { values: [{ ...rawPr, state: "UNKNOWN" }] },
  { values: [{ ...rawPr, description: null }] },
  {
    values: [
      {
        ...rawPr,
        destination: {
          ...rawPr.destination,
          repository: { ...repository, full_name: "acme/other" },
        },
      },
    ],
  },
  {
    values: [
      {
        ...rawPr,
        links: {
          html: { href: "https://bitbucket.org/other/app/pull-requests/4" },
        },
      },
    ],
  },
])(
  "invalid external PR/page refuses inventory %#",
  (page: Readonly<Record<string, unknown>>) => {
    const providers = createProviders(config, (_name, args) =>
      (args[1] ?? "").includes("pullrequests") ? page : repository,
    );
    expect(providers.pullRequests(configuredRepository)).rejects.toThrow();
  },
);

test("a malformed intermediate page cannot masquerade as completed inventory", () => {
  const providers = createProviders(config, (_name, args) => {
    if (!(args[1] ?? "").includes("pullrequests")) {
      return repository;
    }
    if ((args[1] ?? "").includes("cursor=")) {
      return { values: null };
    }
    return {
      values: [rawPr],
      next: "https://api.bitbucket.org/2.0/repositories/acme/app/pullrequests?cursor=two",
    };
  });
  expect(providers.pullRequests(configuredRepository)).rejects.toThrow();
});

test("GraphQL data alongside errors is a failure", () => {
  const providers = createProviders(config, () => ({
    data: { viewer: { id: "user" } },
    errors: [{ message: "partial" }],
  }));
  expect(providers.issue("ENG-12")).rejects.toThrow("GraphQL errors");
});

test("malformed provider UUIDs cannot authorize an attachment", () => {
  const providers = createProviders(config, (_name, args) =>
    (args[1] ?? "").includes("pullrequests")
      ? { values: [{ ...rawPr, author: { uuid: "unknown" } }] }
      : repository,
  );
  expect(providers.pullRequests(configuredRepository)).rejects.toThrow();
});

test("Bitbucket pagination cycle is rejected rather than looping", () => {
  const next =
    "https://api.bitbucket.org/2.0/repositories/acme/app/pullrequests?cursor=one";
  const providers = createProviders(config, (_name, args) =>
    (args[1] ?? "").includes("pullrequests")
      ? { values: [], next }
      : repository,
  );
  expect(providers.pullRequests(configuredRepository)).rejects.toThrow(
    "pagination cycle",
  );
});

const rawIssue = {
  id: "00000000-0000-4000-8000-000000000003",
  identifier: "ENG-12",
  team: { id: "00000000-0000-4000-8000-000000000001" },
  state: {
    id: "00000000-0000-4000-8000-000000000004",
    name: "Todo",
    type: "unstarted",
  },
  archivedAt: null,
};
test("Linear issue attachments exhaust pages and reject a missing cursor", async () => {
  const base = rawIssue;
  const providers = createProviders(config, (_name, args) => {
    const variables = z
      .object({ after: z.string().nullable() })
      .parse(JSON.parse(args.at(-1) ?? "{}"));
    return {
      data: {
        issue: {
          ...base,
          attachments: {
            nodes:
              variables.after === null
                ? []
                : [
                    {
                      id: "link",
                      url: rawPr.links.html.href,
                      title: rawPr.title,
                      subtitle: "MERGED",
                    },
                  ],
            pageInfo: {
              hasNextPage: variables.after === null,
              endCursor: variables.after === null ? "opaque" : null,
            },
          },
        },
      },
    };
  });
  const collected = await providers.issue("ENG-12");
  expect(collected.attachments).toHaveLength(1);
  const missing = createProviders(config, () => ({
    data: {
      issue: {
        ...base,
        attachments: {
          nodes: [],
          pageInfo: { hasNextPage: true, endCursor: null },
        },
      },
    },
  }));
  expect(missing.issue("ENG-12")).rejects.toThrow("pagination");
});

test("attachment mutation success is checked against independent persisted fields", () => {
  const base = {
    id: "00000000-0000-4000-8000-000000000003",
    identifier: "ENG-12",
    team: { id: configuredRepository.teamId },
    state: {
      id: "00000000-0000-4000-8000-000000000004",
      name: "Todo",
      type: "unstarted",
    },
    archivedAt: null,
  };
  const attachment = {
    id: "link",
    url: rawPr.links.html.href,
    title: rawPr.title,
    subtitle: "MERGED",
  };
  const providers = createProviders(config, (_name, args) =>
    (args[1] ?? "").startsWith("mutation")
      ? { data: { attachmentCreate: { success: true, attachment } } }
      : {
          data: {
            issue: {
              ...base,
              attachments: {
                nodes: [],
                pageInfo: { hasNextPage: false, endCursor: null },
              },
            },
          },
        },
  );
  expect(
    providers.attach(base.id, { ...attachment, operation: "link" }),
  ).rejects.toThrow("read-back");
});

test("false mutation success and null required results refuse completion", () => {
  for (const payload of [
    { success: false, issue: null },
    { success: true, issue: null },
  ]) {
    const providers = createProviders(config, () => ({
      data: { issueUpdate: payload },
    }));
    expect(
      providers.complete(
        "00000000-0000-4000-8000-000000000003",
        "00000000-0000-4000-8000-000000000002",
      ),
    ).rejects.toThrow();
  }
});

test("config rejects implicit context, malformed IDs and conflicting repository/team mappings", () => {
  expect(
    configSchema.safeParse({ ...config, bktContext: undefined }).success,
  ).toBe(false);
  expect(
    configSchema.safeParse({
      ...config,
      repositories: [...config.repositories, config.repositories[0]],
    }).success,
  ).toBe(false);
  expect(
    configSchema.safeParse({
      ...config,
      repositories: [{ ...config.repositories[0], teamId: "unknown" }],
    }).success,
  ).toBe(false);
});
