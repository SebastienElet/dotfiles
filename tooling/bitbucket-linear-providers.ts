import {
  type Bitbucket,
  bitbucketUuid,
  readPullRequests,
} from "./bitbucket-linear-bitbucket.ts";
import type {
  Config,
  Issue,
  PullRequest,
  RepositoryConfig,
  Team,
} from "./bitbucket-linear-schema.ts";
import {
  type Graphql,
  readIssue,
  readTeam,
} from "./bitbucket-linear-linear.ts";
import type { AttachmentAction } from "./bitbucket-linear-core.ts";
import { createWrites } from "./bitbucket-linear-writes.ts";
import { z } from "zod";

type Command = (name: string, arguments_: readonly string[]) => unknown;
type Identity = Readonly<{
  bitbucketUserId: string;
  linearUserId: string;
  linearOrganizationId: string;
}>;
type MaybePromise<Value> = Value | Promise<Value>;
type Providers = Readonly<{
  identity: () => MaybePromise<Identity>;
  pullRequests: (
    repository: RepositoryConfig,
  ) => MaybePromise<readonly PullRequest[]>;
  team: (repository: RepositoryConfig) => MaybePromise<Team>;
  issue: (identifier: string) => MaybePromise<Issue>;
  attach: (issueId: string, action: AttachmentAction) => MaybePromise<void>;
  complete: (issueId: string, stateId: string) => MaybePromise<void>;
}>;
function createProviders(config: Config, command: Command): Providers {
  const bitbucket: Bitbucket = async (path) =>
    await command("bkt", [
      "api",
      path,
      "--context",
      config.bktContext,
      "--json",
    ]);
  const graphql: Graphql = async (query, variables = {}) => {
    const response = z
      .object({ data: z.unknown(), errors: z.array(z.unknown()).optional() })
      .parse(
        await command("linear", [
          "api",
          query,
          "--workspace",
          config.linearWorkspace,
          "--variables-json",
          JSON.stringify(variables),
        ]),
      );
    if (response.errors !== undefined && response.errors.length > 0) {
      throw new Error("Linear returned GraphQL errors");
    }
    if (response.data === undefined || response.data === null) {
      throw new Error("Linear returned no data");
    }
    return response.data;
  };
  return {
    identity: () => readIdentity(config, { command, bitbucket, graphql }),
    pullRequests: (repository) => readPullRequests(repository, bitbucket),
    team: (repository) => readTeam(repository, graphql),
    issue: (id) => readIssue(id, graphql),
    ...createWrites(graphql),
  };
}
async function readIdentity(
  config: Config,
  transports: Readonly<{
    command: Command;
    bitbucket: Bitbucket;
    graphql: Graphql;
  }>,
): Promise<Identity> {
  const { command, bitbucket, graphql } = transports;
  const contexts = z
    .object({
      contexts: z.array(z.object({ name: z.string(), host: z.string() })),
    })
    .parse(await command("bkt", ["context", "list", "--json"]))
    .contexts.filter((context) => context.name === config.bktContext);
  if (contexts.length !== 1 || contexts[0]?.host !== "api.bitbucket.org") {
    throw new Error("Configured Bitbucket context is not uniquely Cloud");
  }
  const user = z
    .object({ uuid: bitbucketUuid })
    .parse(await bitbucket("/2.0/user"));
  const identity = z
    .object({
      viewer: z.object({ id: z.uuid() }),
      organization: z.object({ id: z.uuid() }),
    })
    .parse(
      await graphql("query Identity { viewer { id } organization { id } }"),
    );
  return {
    bitbucketUserId: user.uuid,
    linearUserId: identity.viewer.id,
    linearOrganizationId: identity.organization.id,
  };
}

export { createProviders };
export type { Command, Identity, Providers };
