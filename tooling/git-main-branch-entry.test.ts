import { afterEach, describe, expect, test } from "bun:test";
import { cleanup, run } from "./git-main-branch.test-support/harness.ts";

afterEach(cleanup);

const repository = {
  uuid: "{11111111-1111-1111-1111-111111111111}",
  full_name: "acme/service",
  mainbranch: { name: "develop" },
};

describe("generic entrypoint compatibility", () => {
  test.each([["main"], ["master"], ["trunk"]])(
    "selects local %s",
    async (branch) => {
      const result = await run(
        { localBranches: [branch] },
        "-C",
        "/repository",
      );
      expect(result.exitCode).toBe(0);
      expect(result.stdout.toString()).toBe(`${branch}\n`);
    },
  );

  test("keeps permissive fallback outside a repository", async () => {
    const result = await run({ repository: false }, "-C", "/missing");
    expect([
      result.exitCode,
      result.stdout.toString(),
      result.stderr.toString(),
    ]).toEqual([0, "main\n", ""]);
  });

  test("strict mode exposes repository and branch-probe failures", async () => {
    const outside = await run(
      {
        repository: false,
        failures: {
          "git rev-parse --git-dir": { status: 128, stderr: "fatal: absent\n" },
        },
      },
      "--strict",
    );
    expect([
      outside.exitCode,
      outside.stdout.toString(),
      outside.stderr.toString(),
    ]).toEqual([1, "", "fatal: absent\n"]);
    const probe = await run({ showRefStatus: 42 }, "--strict");
    expect([probe.exitCode, probe.stdout.toString()]).toEqual([42, ""]);
  });

  test("permissive fallback preserves branch-probe diagnostics", async () => {
    const result = await run({
      showRefStatus: 42,
      showRefStderr: "fatal: probe\n",
    });
    expect([
      result.exitCode,
      result.stdout.toString(),
      result.stderr.toString(),
    ]).toEqual([0, "main\n", "fatal: probe\nfatal: probe\nfatal: probe\n"]);
  });
});

describe("Bitbucket Cloud entrypoint", () => {
  test("resolves a nonstandard branch despite stale local metadata", async () => {
    const result = await run(
      {
        localBranches: ["main"],
        remotes: {
          origin: {
            urls: ["git@bitbucket.org:acme/service.git"],
            head: "develop",
          },
        },
        repositories: { "acme/service": repository },
      },
      "--bitbucket-cloud",
      "-C",
      "/repository",
    );
    expect([
      result.exitCode,
      result.stdout.toString(),
      result.stderr.toString(),
    ]).toEqual([0, "develop\n", ""]);
  });

  test("accepts equivalent remotes and URL forms", async () => {
    const result = await run(
      {
        remotes: {
          origin: {
            urls: ["https://user@bitbucket.org/acme/service.git"],
            head: "develop",
          },
          mirror: {
            urls: ["ssh://git@bitbucket.org/acme/service.git"],
            head: "develop",
          },
        },
        repositories: { "acme/service": repository },
      },
      "--strict",
      "--bitbucket-cloud",
    );
    expect([result.exitCode, result.stdout.toString()]).toEqual([
      0,
      "develop\n",
    ]);
  });

  test.each([
    ["no remote", { remotes: {} }],
    [
      "non-Bitbucket URL",
      {
        remotes: {
          origin: {
            urls: ["git@github.com:acme/service.git"],
            head: "develop",
          },
        },
      },
    ],
    [
      "multiple contexts",
      {
        remotes: {
          origin: {
            urls: ["git@bitbucket.org:acme/service.git"],
            head: "develop",
          },
        },
        contexts: {
          contexts: [
            { name: "a", host: "api.bitbucket.org" },
            { name: "b", host: "api.bitbucket.org" },
          ],
        },
      },
    ],
    [
      "missing remote HEAD",
      { remotes: { origin: { urls: ["git@bitbucket.org:acme/service.git"] } } },
    ],
    [
      "conflicting remotes",
      {
        remotes: {
          origin: {
            urls: ["git@bitbucket.org:acme/service.git"],
            head: "develop",
          },
          mirror: {
            urls: ["git@bitbucket.org:acme/other.git"],
            head: "develop",
          },
        },
        repositories: {
          "acme/service": repository,
          "acme/other": {
            ...repository,
            uuid: "{22222222-2222-2222-2222-222222222222}",
            full_name: "acme/other",
          },
        },
      },
    ],
    [
      "provider disagreement",
      {
        remotes: {
          origin: {
            urls: ["git@bitbucket.org:acme/service.git"],
            head: "develop",
          },
        },
        repositories: {
          "acme/service": { ...repository, mainbranch: { name: "main" } },
        },
      },
    ],
    [
      "remote branch disagreement",
      {
        remotes: {
          origin: {
            urls: ["git@bitbucket.org:acme/service.git"],
            head: "develop",
          },
          mirror: {
            urls: ["git@bitbucket.org:acme/service.git"],
            head: "main",
          },
        },
        repositories: { "acme/service": repository },
      },
    ],
    [
      "malformed provider response",
      {
        remotes: {
          origin: {
            urls: ["git@bitbucket.org:acme/service.git"],
            head: "develop",
          },
        },
        repositories: { "acme/service": { full_name: "acme/service" } },
      },
    ],
  ] as const)("fails closed on %s", async (_name, fixture) => {
    const result = await run(fixture, "--bitbucket-cloud");
    expect(result.exitCode).not.toBe(0);
    expect(result.stdout.toString()).toBe("");
    expect(result.stderr.toString().trim().length).toBeGreaterThan(0);
  });

  test("reports unavailable dependencies", async () => {
    const gitMissing = await run(
      {
        failures: {
          "git rev-parse --git-dir": { status: 127, stderr: "git missing\n" },
        },
      },
      "--bitbucket-cloud",
    );
    expect([gitMissing.exitCode, gitMissing.stdout.toString()]).toEqual([
      1,
      "",
    ]);
    expect(gitMissing.stderr.toString()).toContain("git missing");

    const result = await run(
      {
        remotes: {
          origin: {
            urls: ["git@bitbucket.org:acme/service.git"],
            head: "develop",
          },
        },
        failures: {
          "bkt context list --json": { status: 127, stderr: "bkt missing\n" },
        },
      },
      "--bitbucket-cloud",
    );
    expect(result.exitCode).toBe(1);
    expect(result.stdout.toString()).toBe("");
    expect(result.stderr.toString()).toContain("bkt missing");
  });
});
