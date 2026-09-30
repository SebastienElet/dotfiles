import type {
  PullRequest,
  RepositoryConfig,
} from "./bitbucket-linear-schema.ts";
import { z } from "zod";

type Bitbucket = (path: string) => Promise<unknown>;
const bitbucketUuid = z
  .string()
  .regex(
    /^\{[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}\}$/iu,
  );
const repositorySchema = z.object({
  uuid: bitbucketUuid,
  full_name: z.string().regex(/^[A-Za-z0-9_.-]+\/[A-Za-z0-9_.-]+$/u),
});
const branchSchema = z.object({
  branch: z.object({ name: z.string().min(1) }),
  repository: repositorySchema,
});
const prSchema = z.object({
  id: z.number().int().positive(),
  title: z.string().min(1),
  description: z.string(),
  state: z.enum(["OPEN", "MERGED", "DECLINED"]),
  author: z.object({ uuid: bitbucketUuid }),
  source: branchSchema,
  destination: branchSchema,
  links: z.object({ html: z.object({ href: z.url() }) }),
});
const pageSchema = z
  .object({
    values: z.array(prSchema),
    next: z.url().optional(),
    size: z.number().int().nonnegative().optional(),
    page: z.number().int().positive().optional(),
    pagelen: z.number().int().positive().optional(),
  })
  .refine(
    (
      page: Readonly<{
        values: readonly unknown[];
        pagelen?: number | undefined;
      }>,
    ) => page.pagelen === undefined || page.values.length <= page.pagelen,
    "Page exceeds advertised length",
  );

async function readPullRequests(
  repository: RepositoryConfig,
  bitbucket: Bitbucket,
): Promise<PullRequest[]> {
  const { identity, root, uuid } = await readRepository(repository, bitbucket);
  let path: string | null =
    `${root}/pullrequests?state=OPEN&state=MERGED&state=DECLINED&pagelen=100`;
  const visited = new Set<string>();
  const advertisedSizes: number[] = [];
  const results: PullRequest[] = [];
  while (path !== null) {
    if (visited.has(path)) {
      throw new Error("Bitbucket pagination cycle");
    }
    visited.add(path);
    const page = pageSchema.parse(await bitbucket(path));
    if (page.size !== undefined) {
      advertisedSizes.push(page.size);
    }
    results.push(
      ...page.values.map((pr) =>
        normalizePullRequest(pr, { repository, identity, providerId: uuid }),
      ),
    );
    path = nextPath(page.next, root);
  }
  validateInventory(results, advertisedSizes);
  return results;
}

function validateInventory(
  results: readonly PullRequest[],
  advertisedSizes: readonly number[],
): void {
  if (new Set(results.map((pr) => pr.url)).size !== results.length) {
    throw new Error("Duplicate pull request identity");
  }
  if (advertisedSizes.some((size) => size !== results.length)) {
    throw new Error("Bitbucket inventory contradicts advertised total");
  }
}
function normalizePullRequest(
  pr: Readonly<z.infer<typeof prSchema>>,
  settings: Readonly<{
    repository: RepositoryConfig;
    providerId: string;
    identity: string;
  }>,
): PullRequest {
  const { repository, providerId, identity } = settings;
  const url = `https://bitbucket.org/${identity}/pull-requests/${pr.id}`;
  if (
    pr.links.html.href !== url ||
    pr.destination.repository.full_name !== identity ||
    pr.destination.repository.uuid !== providerId
  ) {
    throw new Error("Invalid or duplicate pull request identity");
  }
  return {
    id: pr.id,
    title: pr.title,
    description: pr.description,
    state: pr.state,
    branch: pr.source.branch.name,
    url,
    repository: identity,
    sourceRepository: pr.source.repository.full_name,
    sourceRepositoryId: pr.source.repository.uuid,
    destinationRepositoryId: pr.destination.repository.uuid,
    authorId: pr.author.uuid,
    teamKey: repository.teamKey,
    teamId: repository.teamId,
  };
}
function nextPath(value: string | undefined, root: string): string | null {
  if (value === undefined) {
    return null;
  }
  const next = new URL(value);
  if (
    next.origin !== "https://api.bitbucket.org" ||
    next.username !== "" ||
    next.password !== "" ||
    next.hash !== "" ||
    next.pathname !== `${root}/pullrequests`
  ) {
    throw new Error("Bitbucket pagination escaped configured repository");
  }
  return `${next.pathname}${next.search}`;
}

async function readRepository(
  repository: RepositoryConfig,
  bitbucket: Bitbucket,
): Promise<Readonly<{ identity: string; root: string; uuid: string }>> {
  const identity = `${repository.workspace}/${repository.repository}`;
  const root = `/2.0/repositories/${identity}`;
  const actual = repositorySchema.parse(await bitbucket(root));
  if (actual.full_name !== identity) {
    throw new Error("Bitbucket repository identity mismatch");
  }
  return { identity, root, uuid: actual.uuid };
}

export { bitbucketUuid, readPullRequests };
export type { Bitbucket };
