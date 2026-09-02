import { z } from "zod";

const pullRequestUrlPattern =
  /^https:\/\/github\.com\/(?<owner>[A-Za-z0-9](?:[A-Za-z0-9-]{0,37}[A-Za-z0-9])?)\/(?<repository>(?!\.{1,2}\/)[A-Za-z0-9_.-]{1,100})\/pull\/(?<number>[1-9][0-9]*)\/?$/u;
const pullRequestUrlSchema = z
  .string()
  .refine((value): boolean => pullRequestUrlPattern.test(value), {
    message: "Pull request URL must be a canonical GitHub HTTPS URL.",
  })
  .transform((value): string =>
    value.endsWith("/") ? value.slice(0, -1) : value,
  );
const sourceSchema = z
  .object({ pullRequestUrl: pullRequestUrlSchema, evidenceUrl: z.url() })
  .strict();

const pullRequestIdentity = (pullRequestUrl: string): string => {
  const match = pullRequestUrlPattern.exec(pullRequestUrl);
  if (match === null) {
    throw new Error("Pull request URL is not canonical.");
  }
  const owner = match.groups?.owner ?? "";
  const repository = match.groups?.repository ?? "";
  const number = match.groups?.number ?? "";
  return `${owner.toLowerCase()}/${repository.toLowerCase()}/${number}`;
};

export { pullRequestIdentity, sourceSchema };
