import type {
  Issue,
  RepositoryConfig,
  Team,
} from "./bitbucket-linear-schema.ts";
import { attachmentSchema, stateSchema } from "./bitbucket-linear-schema.ts";
import { z } from "zod";

type Graphql = (
  query: string,
  variables?: Readonly<Record<string, unknown>>,
) => Promise<unknown>;
const pageInfoSchema = z
  .object({
    hasNextPage: z.boolean(),
    endCursor: z.string().min(1).nullable(),
  })
  .readonly();
const issueSchema = z.object({
  id: z.uuid(),
  identifier: z.string().regex(/^[A-Z][A-Z0-9]*-[0-9]+$/u),
  team: z.object({ id: z.uuid() }),
  state: stateSchema,
  archivedAt: z.iso.datetime().nullable(),
  attachments: z.object({
    nodes: z.array(attachmentSchema),
    pageInfo: pageInfoSchema,
  }),
});

async function readTeam(
  repository: RepositoryConfig,
  graphql: Graphql,
): Promise<Team> {
  const states: Team["states"][number][] = [];
  let after: string | null = null;
  const visited = new Set<string>();
  do {
    const data = z
      .object({
        team: z.object({
          id: z.uuid(),
          key: z.string(),
          states: z.object({
            nodes: z.array(stateSchema),
            pageInfo: pageInfoSchema,
          }),
        }),
      })
      .parse(
        await graphql(
          "query Team($id:String!,$after:String) { team(id:$id) { id key states(first:100,after:$after) { nodes { id name type } pageInfo { hasNextPage endCursor } } } }",
          { id: repository.teamId, after },
        ),
      );
    if (
      data.team.id !== repository.teamId ||
      data.team.key !== repository.teamKey
    ) {
      throw new Error("Linear team identity mismatch");
    }
    states.push(...data.team.states.nodes);
    after = advance(data.team.states.pageInfo, visited);
    if (after !== null) {
      visited.add(after);
    }
  } while (after !== null);
  if (new Set(states.map((state) => state.id)).size !== states.length) {
    throw new Error("Duplicate team states");
  }
  return {
    id: repository.teamId,
    key: repository.teamKey,
    states,
    ...(repository.completedStateId === undefined
      ? {}
      : { completedStateId: repository.completedStateId }),
  };
}

async function readIssue(identifier: string, graphql: Graphql): Promise<Issue> {
  const attachments: Issue["attachments"][number][] = [];
  let after: string | null = null;
  const visited = new Set<string>();
  let result: Issue | undefined = undefined;
  do {
    const { issue } = z
      .object({ issue: issueSchema })
      .parse(
        await graphql(
          "query Issue($id:String!,$after:String) { issue(id:$id) { id identifier archivedAt team { id } state { id name type } attachments(first:100,after:$after) { nodes { id url title subtitle } pageInfo { hasNextPage endCursor } } } }",
          { id: identifier, after },
        ),
      );
    if (
      (identifier !== issue.id && identifier !== issue.identifier) ||
      (result !== undefined &&
        (result.id !== issue.id ||
          result.state.id !== issue.state.id ||
          result.teamId !== issue.team.id ||
          result.archivedAt !== issue.archivedAt))
    ) {
      throw new Error("Issue identity or state changed during pagination");
    }
    attachments.push(...issue.attachments.nodes);
    result = {
      id: issue.id,
      identifier: issue.identifier,
      teamId: issue.team.id,
      state: issue.state,
      archivedAt: issue.archivedAt,
      attachments,
    };
    after = advance(issue.attachments.pageInfo, visited);
    if (after !== null) {
      visited.add(after);
    }
  } while (after !== null);
  if (
    result === undefined ||
    new Set(attachments.map((attachment) => attachment.id)).size !==
      attachments.length
  ) {
    throw new Error("Incomplete issue attachments");
  }
  return result;
}

function advance(
  pageInfo: z.infer<typeof pageInfoSchema>,
  visited: Readonly<Pick<ReadonlySet<string>, "has">>,
): string | null {
  if (!pageInfo.hasNextPage) {
    return null;
  }
  if (pageInfo.endCursor === null || visited.has(pageInfo.endCursor)) {
    throw new Error("Linear pagination is incomplete or cyclic");
  }
  return pageInfo.endCursor;
}

export { readIssue, readTeam };
export type { Graphql };
