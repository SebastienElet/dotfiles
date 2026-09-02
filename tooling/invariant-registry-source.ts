import { z } from "zod";

type SourceFields = Readonly<{
  evidenceUrl: string;
  pullRequestUrl: string;
}>;
type ForgePatterns = Readonly<{
  evidence: Readonly<RegExp>;
  pullRequest: Readonly<RegExp>;
}>;

const repositoryPart = String.raw`[A-Za-z0-9_.-]+`;
const githubPatterns: ForgePatterns = {
  pullRequest: new RegExp(
    String.raw`^https:\/\/github\.com\/(?<owner>${repositoryPart})\/(?<repository>${repositoryPart})\/pull\/(?<number>[1-9][0-9]*)\/?$`,
    "iu",
  ),
  evidence: new RegExp(
    String.raw`^https:\/\/github\.com\/(?<owner>${repositoryPart})\/(?<repository>${repositoryPart})\/pull\/(?<number>[1-9][0-9]*)#(?:(?:issuecomment|pullrequestreview)-[1-9][0-9]*|discussion_r[1-9][0-9]*)$`,
    "iu",
  ),
};
const bitbucketPatterns: ForgePatterns = {
  pullRequest: new RegExp(
    String.raw`^https:\/\/bitbucket\.org\/(?<owner>${repositoryPart})\/(?<repository>${repositoryPart})\/pull-requests\/(?<number>[1-9][0-9]*)\/?$`,
    "iu",
  ),
  evidence: new RegExp(
    String.raw`^https:\/\/bitbucket\.org\/(?<owner>${repositoryPart})\/(?<repository>${repositoryPart})\/pull-requests\/(?<number>[1-9][0-9]*)\/comments\/[1-9][0-9]*\/?$`,
    "iu",
  ),
};

const canonicalUrl = (value: string): string =>
  value.endsWith("/") ? value.slice(0, -1) : value;

const coherentSource = (
  source: SourceFields,
  patterns: ForgePatterns,
): boolean => {
  const pullRequest = patterns.pullRequest.exec(source.pullRequestUrl)?.groups;
  const evidence = patterns.evidence.exec(source.evidenceUrl)?.groups;
  return (
    pullRequest !== undefined &&
    evidence !== undefined &&
    pullRequest.owner?.toLowerCase() === evidence.owner?.toLowerCase() &&
    pullRequest.repository?.toLowerCase() ===
      evidence.repository?.toLowerCase() &&
    pullRequest.number === evidence.number
  );
};

const githubSourceSchema = z
  .object({
    provider: z.literal("github"),
    pullRequestUrl: z.string().regex(githubPatterns.pullRequest),
    evidenceUrl: z.string().regex(githubPatterns.evidence),
  })
  .strict()
  .refine((source) => coherentSource(source, githubPatterns))
  .transform((source) => ({
    ...source,
    evidenceUrl: canonicalUrl(source.evidenceUrl),
    pullRequestUrl: canonicalUrl(source.pullRequestUrl),
  }));
const bitbucketSourceSchema = z
  .object({
    provider: z.literal("bitbucket-cloud"),
    pullRequestUrl: z.string().regex(bitbucketPatterns.pullRequest),
    evidenceUrl: z.string().regex(bitbucketPatterns.evidence),
  })
  .strict()
  .refine((source) => coherentSource(source, bitbucketPatterns))
  .transform((source) => ({
    ...source,
    evidenceUrl: canonicalUrl(source.evidenceUrl),
    pullRequestUrl: canonicalUrl(source.pullRequestUrl),
  }));
const sourceSchema = z.discriminatedUnion("provider", [
  githubSourceSchema,
  bitbucketSourceSchema,
]);
type ReviewSource = z.output<typeof sourceSchema>;

const pullRequestIdentity = (source: Readonly<ReviewSource>): string =>
  `${source.provider}:${source.pullRequestUrl.toLowerCase()}`;

const evidenceOccurrenceIdentity = (source: Readonly<ReviewSource>): string =>
  `${source.provider}:${source.evidenceUrl.toLowerCase()}`;

export {
  evidenceOccurrenceIdentity,
  pullRequestIdentity,
  sourceSchema,
  type ReviewSource,
};
