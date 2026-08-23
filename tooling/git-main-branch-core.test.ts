import { describe, expect, test } from "bun:test";
import {
  parseBitbucketIdentity,
  parseCloudContexts,
  parseProviderRepository,
  parseRemoteHead,
  reconcileProviderEvidence,
} from "./git-main-branch-core.ts";

describe("parseBitbucketIdentity", () => {
  test.each([
    ["git@bitbucket.org:acme/service.git", "acme/service"],
    ["ssh://git@bitbucket.org/acme/service.git", "acme/service"],
    ["https://bitbucket.org/acme/service.git", "acme/service"],
    ["https://token@bitbucket.org/acme/service", "acme/service"],
  ])("parses %s", (url, identity) => {
    expect(parseBitbucketIdentity(url)).toEqual({
      identity,
      workspace: "acme",
      slug: "service",
    });
  });

  test.each([
    "git@github.com:acme/service.git",
    "git@bitbucket.org:acme",
    "https://bitbucket.org/acme/service/extra",
    "https://bitbucket.org/acme/%2Fservice",
    "ssh://someone@bitbucket.org/acme/service.git",
  ])("rejects %s", (url) =>
    expect(() => parseBitbucketIdentity(url)).toThrow(url),
  );
});

test("parseCloudContexts requires exactly one named Cloud context", () => {
  expect(
    parseCloudContexts(
      '{"contexts":[{"name":"cloud","host":"api.bitbucket.org"}]}',
    ),
  ).toBe("cloud");
  expect(() => parseCloudContexts('{"contexts":[]}')).toThrow("exactly one");
  expect(() =>
    parseCloudContexts(
      '{"contexts":[{"name":"a","host":"api.bitbucket.org"},{"name":"b","host":"api.bitbucket.org"}]}',
    ),
  ).toThrow("exactly one");
  expect(() => parseCloudContexts("not json")).toThrow("valid JSON");
});

test("provider repositories require validated identity, UUID and branch", () => {
  expect(
    parseProviderRepository(
      '{"uuid":"{11111111-1111-1111-1111-111111111111}","full_name":"acme/service","mainbranch":{"name":"develop"}}',
      "acme/service",
    ),
  ).toEqual({
    uuid: "{11111111-1111-1111-1111-111111111111}",
    branch: "develop",
  });
  expect(() => parseProviderRepository("{}", "acme/service")).toThrow(
    "repository response",
  );
  expect(() =>
    parseProviderRepository(
      '{"uuid":"{11111111-1111-1111-1111-111111111111}","full_name":"acme/other","mainbranch":{"name":"develop"}}',
      "acme/service",
    ),
  ).toThrow("repository response");
});

test("remote HEAD parsing requires one heads symref", () => {
  expect(parseRemoteHead("ref: refs/heads/develop\tHEAD\nabc\tHEAD\n")).toBe(
    "develop",
  );
  expect(() => parseRemoteHead("abc\tHEAD\n")).toThrow("symbolic HEAD");
  expect(() => parseRemoteHead("ref: refs/tags/v1\tHEAD\n")).toThrow(
    "symbolic HEAD",
  );
  expect(() =>
    parseRemoteHead(
      "ref: refs/heads/main\tHEAD\nref: refs/heads/trunk\tHEAD\n",
    ),
  ).toThrow("exactly one");
});

test("reconciliation accepts equivalent evidence and refuses conflicts", () => {
  expect(
    reconcileProviderEvidence([
      {
        identity: "acme/service",
        remoteBranch: "develop",
        uuid: "{same}",
        providerBranch: "develop",
      },
      {
        identity: "acme/service",
        remoteBranch: "develop",
        uuid: "{same}",
        providerBranch: "develop",
      },
    ]),
  ).toBe("develop");
  expect(() =>
    reconcileProviderEvidence([
      {
        identity: "acme/service",
        remoteBranch: "develop",
        uuid: "{a}",
        providerBranch: "develop",
      },
      {
        identity: "acme/other",
        remoteBranch: "develop",
        uuid: "{b}",
        providerBranch: "develop",
      },
    ]),
  ).toThrow("same repository");
  expect(() =>
    reconcileProviderEvidence([
      {
        identity: "acme/service",
        remoteBranch: "develop",
        uuid: "{a}",
        providerBranch: "main",
      },
    ]),
  ).toThrow("disagrees");
});
