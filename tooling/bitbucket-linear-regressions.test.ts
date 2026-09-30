import { chmod, mkdir, mkdtemp, rm, writeFile } from "node:fs/promises";
import { expect, test } from "bun:test";
import { configSchema } from "./bitbucket-linear-schema.ts";
import { createProviders } from "./bitbucket-linear-providers.ts";
import { join } from "node:path";
import { tmpdir } from "node:os";

const teamId = "00000000-0000-4000-8000-000000000001";
const secondPullRequestId = 5;
const done = {
  id: "00000000-0000-4000-8000-000000000002",
  name: "Done",
  type: "completed",
};
const repository = {
  uuid: "{00000000-0000-4000-8000-000000000011}",
  full_name: "acme/app",
};
const pullRequest = {
  id: 4,
  title: "ENG-12",
  description: "",
  state: "MERGED",
  author: { uuid: "{00000000-0000-4000-8000-000000000012}" },
  source: { branch: { name: "ENG-12-fix" }, repository },
  destination: { branch: { name: "main" }, repository },
  links: { html: { href: "https://bitbucket.org/acme/app/pull-requests/4" } },
};
const config = configSchema.parse({
  bktContext: "work",
  linearWorkspace: "work",
  repositories: [
    { workspace: "acme", repository: "app", teamKey: "ENG", teamId },
  ],
});
const issue = {
  id: "00000000-0000-4000-8000-000000000003",
  identifier: "ENG-12",
  team: { id: teamId },
  state: {
    id: "00000000-0000-4000-8000-000000000004",
    name: "Todo",
    type: "unstarted",
  },
  archivedAt: null,
  attachments: {
    nodes: [
      {
        id: "link",
        url: pullRequest.links.html.href,
        title: pullRequest.title,
        subtitle: pullRequest.state,
      },
    ],
    pageInfo: { hasNextPage: false, endCursor: null },
  },
};

test("provider rejects a terminal page that contradicts its advertised total", () => {
  const [configuredRepository] = config.repositories;
  if (configuredRepository === undefined) {
    throw new Error("missing fixture repository");
  }
  const providers = createProviders(config, (_name, args) =>
    args[1]?.includes("pullrequests") === true
      ? { size: 2, page: 1, pagelen: 100, values: [pullRequest] }
      : repository,
  );
  expect(providers.pullRequests(configuredRepository)).rejects.toThrow();
});

test.each([
  { size: null },
  { size: "2" },
  { size: -1 },
  { page: 0 },
  { pagelen: 0 },
])(
  "malformed optional pagination metadata does not become a complete inventory %#",
  (metadata: Readonly<Record<string, unknown>>) => {
    const [configuredRepository] = config.repositories;
    if (configuredRepository === undefined) {
      throw new Error("missing fixture repository");
    }
    const providers = createProviders(config, (_name, args) =>
      args[1]?.includes("pullrequests") === true
        ? { values: [pullRequest], ...metadata }
        : repository,
    );
    expect(providers.pullRequests(configuredRepository)).rejects.toThrow();
  },
);

test("advertised total agrees with all collected pages before inventory succeeds", async () => {
  const [configuredRepository] = config.repositories;
  if (configuredRepository === undefined) {
    throw new Error("missing fixture repository");
  }
  const providers = createProviders(config, (_name, args) => {
    if (args[1]?.includes("pullrequests") !== true) {
      return repository;
    }
    if (args[1].includes("cursor=")) {
      return {
        size: 2,
        page: 2,
        pagelen: 100,
        values: [
          {
            ...pullRequest,
            id: secondPullRequestId,
            links: {
              html: { href: "https://bitbucket.org/acme/app/pull-requests/5" },
            },
          },
        ],
      };
    }
    return {
      size: 2,
      page: 1,
      pagelen: 100,
      values: [pullRequest],
      next: "https://api.bitbucket.org/2.0/repositories/acme/app/pullrequests?cursor=opaque",
    };
  });
  const collected = await providers.pullRequests(configuredRepository);
  expect(collected.map((pr) => pr.id)).toEqual([
    pullRequest.id,
    secondPullRequestId,
  ]);
});

const responses = [
  {
    first: "context",
    prefix: "list",
    response: { contexts: [{ name: "work", host: "api.bitbucket.org" }] },
  },
  {
    first: "api",
    prefix: "/2.0/user",
    response: { uuid: pullRequest.author.uuid },
  },
  {
    first: "api",
    prefix: "/2.0/repositories/acme/app/pullrequests",
    response: { size: 2, page: 1, pagelen: 100, values: [pullRequest] },
  },
  {
    first: "api",
    prefix: "/2.0/repositories/acme/app",
    response: repository,
  },
  {
    first: "api",
    prefix: "query Identity",
    response: {
      data: { viewer: { id: teamId }, organization: { id: teamId } },
    },
  },
  {
    first: "api",
    prefix: "query Team",
    response: {
      data: {
        team: {
          id: teamId,
          key: "ENG",
          states: {
            nodes: [done],
            pageInfo: { hasNextPage: false, endCursor: null },
          },
        },
      },
    },
  },
  { first: "api", prefix: "query Issue", response: { data: { issue } } },
  {
    first: "api",
    prefix: "mutation Complete",
    response: {
      data: {
        issueUpdate: { success: true, issue: { id: issue.id, state: done } },
      },
    },
  },
];
const executableMode = 0o755;
async function fakeTools(directory: string): Promise<string> {
  const bin = join(directory, "bin");
  const marker = join(directory, "mutation");
  await mkdir(bin);

  const script = `#!${process.execPath}\nconst responses = ${JSON.stringify(responses)};\nconst [first, second] = process.argv.slice(2);\nconst match = responses.find(row => row.first === first && second?.startsWith(row.prefix));\nif (!match) throw new Error("unexpected fixture request");\nif (second?.startsWith("mutation")) await Bun.write(${JSON.stringify(marker)}, "mutation");\nprocess.stdout.write(JSON.stringify(match.response));\n`;
  for (const name of ["bkt", "linear"]) {
    const path = join(bin, name);
    await writeFile(path, script);
    await chmod(path, executableMode);
  }
  return bin;
}

test.each([false, true])(
  "real CLI refuses truncated inventory before mutations, apply=%s",
  async (apply) => {
    const directory = await mkdtemp(
      join(tmpdir(), "bitbucket-linear-regression-"),
    );
    try {
      const bin = await fakeTools(directory);
      const path = join(directory, "config.json");
      await writeFile(path, JSON.stringify(config));
      const result = Bun.spawnSync(
        [
          process.execPath,
          join(import.meta.dir, "bitbucket-linear-sync"),
          "--config",
          path,
          "--json",
          ...(apply ? ["--apply"] : []),
        ],
        {
          env: { ...process.env, PATH: bin },
          stdout: "pipe",
          stderr: "pipe",
        },
      );
      expect(result.exitCode).toBe(1);
      expect(result.stdout.toString()).toContain("inventory-or-team-failed");
      expect(await Bun.file(join(directory, "mutation")).exists()).toBe(false);
    } finally {
      await rm(directory, { recursive: true, force: true });
    }
  },
);
