import type { GithubApi, GithubRequest } from "./github-automerge-plan.ts";
import { expect, test } from "bun:test";
import { join, resolve } from "node:path";
import { mkdtemp, rm, writeFile } from "node:fs/promises";
import { configureGithubAutomerge } from "./configure-github-automerge.ts";
import { rejects } from "node:assert/strict";
import { tmpdir } from "node:os";

const changesPerRepository = 2;
const fixtureExecutableMode = 0o700;
const timeoutTestMilliseconds = 35_000;

function githubInMemory(): { api: GithubApi; writes: GithubRequest[] } {
  const writes: GithubRequest[] = [];
  const api: GithubApi = (request) => {
    if (request.method !== "GET") {
      writes.push(request);
      return Promise.resolve({});
    }
    return Promise.resolve(
      request.endpoint.includes("rulesets")
        ? [[]]
        : { default_branch: "main", allow_auto_merge: false },
    );
  };
  return { api, writes };
}

test("dry-run displays the complete plan without writing to GitHub", async () => {
  const { api, writes } = githubInMemory();
  const output: string[] = [];
  await configureGithubAutomerge(["--dry-run"], api, (line) => {
    output.push(line);
  });
  expect(writes).toEqual([]);
  expect(output.join("\n")).toContain(
    "Dry run: 11 repositories, 22 changes; no writes.",
  );
  expect(output.join("\n")).toContain('"allow_auto_merge":true');
});

test.each([
  { arguments_: [] },
  { arguments_: ["--apply", "--dry-run"] },
  { arguments_: ["--dry-rnu"] },
  { arguments_: ["--repo", "third-party/repo"] },
])(
  "rejects ambiguous or unknown arguments before contacting GitHub: %j",
  async ({ arguments_ }) => {
    const { api, writes } = githubInMemory();
    const output: string[] = [];
    await rejects(
      configureGithubAutomerge(arguments_, api, (message) => {
        output.push(message);
      }),
      /Usage:/u,
    );
    expect(writes).toEqual([]);
  },
);

test("a late read failure prevents all writes", async () => {
  const output: string[] = [];
  const memory = githubInMemory();
  const api: GithubApi = (request) =>
    request.endpoint === "repos/SebastienElet/mst-cli"
      ? Promise.reject(new Error("gh failed: HTTP 403"))
      : memory.api(request);
  await rejects(
    configureGithubAutomerge(["--apply"], api, (message) => {
      output.push(message);
    }),
    /HTTP 403/u,
  );
  expect(memory.writes).toEqual([]);
});

test("with an in-memory GitHub, applies required checks before auto-merge", async () => {
  const { api, writes } = githubInMemory();
  const output: string[] = [];
  await configureGithubAutomerge(["--apply"], api, (message) => {
    output.push(message);
  });
  expect(writes.slice(-changesPerRepository)).toMatchObject([
    { method: "POST", endpoint: "repos/SebastienElet/mst-cli/rulesets" },
    {
      method: "PATCH",
      endpoint: "repos/SebastienElet/mst-cli",
      body: { allow_auto_merge: true },
    },
  ]);
});

test("stops on a failed ruleset write without enabling auto-merge", async () => {
  const output: string[] = [];
  const memory = githubInMemory();
  const api: GithubApi = (request) =>
    request.method === "POST"
      ? Promise.reject(new Error("gh failed: HTTP 422"))
      : memory.api(request);
  await rejects(
    configureGithubAutomerge(["--apply"], api, (message) => {
      output.push(message);
    }),
    /HTTP 422/u,
  );
  expect(memory.writes).toEqual([]);
});

async function runFixtureCli(
  recipe: string | null,
): Promise<Readonly<{ status: number; stdout: string; stderr: string }>> {
  const directory = await mkdtemp(join(tmpdir(), "github-automerge-cli-"));
  try {
    if (recipe !== null) {
      await writeFile(join(directory, "gh"), `#!/bin/sh\n${recipe}\n`, {
        mode: fixtureExecutableMode,
      });
    }
    const child = Bun.spawn(
      [
        process.execPath,
        "--config=/dev/null",
        "--no-env-file",
        resolve(import.meta.dir, "configure-github-automerge.ts"),
        "--dry-run",
      ],
      {
        env: { PATH: directory },
        stdout: "pipe",
        stderr: "pipe",
      },
    );
    const [status, stdout, stderr] = await Promise.all([
      child.exited,
      new Response(child.stdout).text(),
      new Response(child.stderr).text(),
    ]);
    return { status, stdout, stderr };
  } finally {
    await rm(directory, { recursive: true, force: true });
  }
}

test.each([
  { recipe: null, error: "gh" },
  { recipe: "printf '%s\\n' 'HTTP 403' >&2\nexit 1", error: "HTTP 403" },
  { recipe: "printf '%s\\n' 'invalid-json'", error: "JSON" },
])(
  "the real CLI fails visibly with a local gh fixture: $error",
  async ({ recipe, error }) => {
    const result = await runFixtureCli(recipe);
    expect(result.status).toBe(1);
    expect(result.stderr).toContain(error);
    expect(result.stdout).toBe("");
  },
);

test(
  "the real CLI bounds a hung gh process",
  async () => {
    const result = await runFixtureCli("exec /bin/sleep 60");
    expect(result.status).toBe(1);
    expect(result.stderr).toContain("SIGTERM");
    expect(result.stdout).toBe("");
  },
  timeoutTestMilliseconds,
);
