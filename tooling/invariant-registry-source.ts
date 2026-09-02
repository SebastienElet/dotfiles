import { z } from "zod";

type Provider = "github" | "bitbucket-cloud";
type SourceFields = Readonly<{
  evidenceUrl: string;
  pullRequestUrl: string;
}>;
type ParsedPullRequest = Readonly<{
  canonicalUrl: string;
  number: string;
  owner: string;
  repository: string;
}>;
type ParsedEvidence = ParsedPullRequest & Readonly<{ occurrence: string }>;
type ParsedUrl = Readonly<{ hash: string; pathname: string }>;

const repositoryPart = /^[A-Za-z0-9_.-]+$/u;
const positiveInteger = /^[1-9][0-9]*$/u;
const bitbucketEvidencePathLength = 2;

const providerDomain = (provider: Provider): string =>
  provider === "github" ? "github.com" : "bitbucket.org";

const pathSegments = (url: ParsedUrl): readonly string[] => {
  const segments = url.pathname.split("/").slice(1);
  return segments.at(-1) === "" ? segments.slice(0, -1) : segments;
};

const parseUrl = (value: string, provider: Provider): ParsedUrl | undefined => {
  const domain = providerDomain(provider);
  const authority = /^https:\/\/(?<authority>[^/?#]+)/u.exec(value)?.groups
    ?.authority;
  if (authority !== domain || value.includes("%")) {
    return undefined;
  }
  try {
    const url = new URL(value);
    return url.protocol === "https:" &&
      url.hostname === domain &&
      url.username === "" &&
      url.password === "" &&
      url.port === "" &&
      url.search === ""
      ? { hash: url.hash, pathname: url.pathname }
      : undefined;
  } catch {
    return undefined;
  }
};

const validRepositoryPart = (value: string): boolean =>
  repositoryPart.test(value) && value !== "." && value !== "..";

const parsePullRequest = (
  value: string,
  provider: Provider,
): ParsedPullRequest | undefined => {
  const url = parseUrl(value, provider);
  if (url === undefined || url.hash !== "") {
    return undefined;
  }
  const [owner, repository, keyword, number, ...extra] = pathSegments(url);
  const expectedKeyword = provider === "github" ? "pull" : "pull-requests";
  if (
    owner === undefined ||
    repository === undefined ||
    !validRepositoryPart(owner) ||
    !validRepositoryPart(repository) ||
    keyword !== expectedKeyword ||
    number === undefined ||
    !positiveInteger.test(number) ||
    extra.length > 0
  ) {
    return undefined;
  }
  return {
    canonicalUrl: `https://${providerDomain(provider)}/${owner}/${repository}/${keyword}/${number}`,
    number,
    owner,
    repository,
  };
};

const githubEvidence = (url: ParsedUrl): string | undefined =>
  /^#(?<occurrence>(?:issuecomment|pullrequestreview)-[1-9][0-9]*|discussion_r[1-9][0-9]*)$/u.exec(
    url.hash,
  )?.groups?.occurrence;

const bitbucketEvidence = (
  url: ParsedUrl,
  extra: readonly string[],
): string | undefined => {
  if (
    extra[0] !== "_" ||
    extra[1] !== "diff" ||
    extra.length !== bitbucketEvidencePathLength
  ) {
    return undefined;
  }
  return /^#(?<occurrence>comment-[1-9][0-9]*)$/u.exec(url.hash)?.groups
    ?.occurrence;
};

const evidenceOccurrence = (
  url: ParsedUrl,
  provider: Provider,
  extra: readonly string[],
): string | undefined => {
  if (provider === "bitbucket-cloud") {
    return bitbucketEvidence(url, extra);
  }
  return extra.length > 0 ? undefined : githubEvidence(url);
};

const parseEvidence = (
  value: string,
  provider: Provider,
): ParsedEvidence | undefined => {
  const url = parseUrl(value, provider);
  if (url === undefined) {
    return undefined;
  }
  const [owner, repository, keyword, number, ...extra] = pathSegments(url);
  const expectedKeyword = provider === "github" ? "pull" : "pull-requests";
  const occurrence = evidenceOccurrence(url, provider, extra);
  if (
    owner === undefined ||
    repository === undefined ||
    !validRepositoryPart(owner) ||
    !validRepositoryPart(repository) ||
    keyword !== expectedKeyword ||
    number === undefined ||
    !positiveInteger.test(number) ||
    occurrence === undefined
  ) {
    return undefined;
  }
  return {
    canonicalUrl: value.endsWith("/") ? value.slice(0, -1) : value,
    number,
    occurrence,
    owner,
    repository,
  };
};

const parsedSource = (
  source: SourceFields,
  provider: Provider,
):
  | Readonly<{
      evidence: ParsedEvidence;
      pullRequest: ParsedPullRequest;
    }>
  | undefined => {
  const pullRequest = parsePullRequest(source.pullRequestUrl, provider);
  const evidence = parseEvidence(source.evidenceUrl, provider);
  return pullRequest !== undefined &&
    evidence !== undefined &&
    pullRequest.owner.toLowerCase() === evidence.owner.toLowerCase() &&
    pullRequest.repository.toLowerCase() ===
      evidence.repository.toLowerCase() &&
    pullRequest.number === evidence.number
    ? { evidence, pullRequest }
    : undefined;
};

const canonicalSource = <ProviderName extends Provider>(
  source: SourceFields & Readonly<{ provider: ProviderName }>,
  provider: ProviderName,
): SourceFields & Readonly<{ provider: ProviderName }> => {
  const parsed = parsedSource(source, provider);
  if (parsed === undefined) {
    throw new TypeError("invalid review source");
  }
  return {
    ...source,
    evidenceUrl: parsed.evidence.canonicalUrl,
    pullRequestUrl: parsed.pullRequest.canonicalUrl,
  };
};

const githubSourceSchema = z
  .object({
    provider: z.literal("github"),
    pullRequestUrl: z.string(),
    evidenceUrl: z.string(),
  })
  .strict()
  .refine((source) => parsedSource(source, "github") !== undefined)
  .transform((source) => canonicalSource(source, "github"));
const bitbucketSourceSchema = z
  .object({
    provider: z.literal("bitbucket-cloud"),
    pullRequestUrl: z.string(),
    evidenceUrl: z.string(),
  })
  .strict()
  .refine((source) => parsedSource(source, "bitbucket-cloud") !== undefined)
  .transform((source) => canonicalSource(source, "bitbucket-cloud"));

const sourceSchema = z.discriminatedUnion("provider", [
  githubSourceSchema,
  bitbucketSourceSchema,
]);
type ReviewSource = z.output<typeof sourceSchema>;

const pullRequestIdentity = (source: Readonly<ReviewSource>): string => {
  const parsed = parsePullRequest(source.pullRequestUrl, source.provider);
  if (parsed === undefined) {
    throw new TypeError("invalid pull request source");
  }
  return `${source.provider}:${parsed.owner.toLowerCase()}/${parsed.repository.toLowerCase()}/${parsed.number}`;
};

const evidenceOccurrenceIdentity = (source: Readonly<ReviewSource>): string => {
  const parsed = parseEvidence(source.evidenceUrl, source.provider);
  if (parsed === undefined) {
    throw new TypeError("invalid review evidence");
  }
  return `${source.provider}:${parsed.owner.toLowerCase()}/${parsed.repository.toLowerCase()}/${parsed.number}/${parsed.occurrence}`;
};

export {
  evidenceOccurrenceIdentity,
  pullRequestIdentity,
  sourceSchema,
  type ReviewSource,
};
