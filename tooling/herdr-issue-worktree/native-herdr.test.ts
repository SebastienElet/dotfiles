import { afterEach, expect, test } from "bun:test";
import type { IssueFixture } from "./dispatch-test-support.ts";
import { createNativeHerdr } from "./native-herdr.ts";
import { inspectRepository } from "./repository.ts";
import { issueFixture } from "./dispatch-test-support.ts";

const fixtures: IssueFixture[] = [];
afterEach(() => {
  for (const fixture of fixtures.splice(0)) {
    fixture.dispose();
  }
});

test("validates the native session snapshot rather than accepting malformed state", async () => {
  const herdr = createNativeHerdr(() =>
    Promise.resolve({
      type: "session_snapshot",
      snapshot: { version: "0.9.3" },
    }),
  );

  const result = await Promise.allSettled([herdr.snapshot()]);
  expect(result[0]?.status).toBe("rejected");
});

test("accepts the native primary source when the selected checkout is a linked worktree of that repository", async () => {
  const fixture = issueFixture();
  fixtures.push(fixture);
  const selection = {
    ...fixture.selection,
    repository: inspectRepository(fixture.checkout),
  };
  const calls: (readonly string[])[] = [];
  const herdr = createNativeHerdr((arguments_: readonly string[]) => {
    calls.push(arguments_);
    return Promise.resolve(
      arguments_[0] === "worktree"
        ? {
            source: {
              repo_key: selection.repository.commonDirectory,
              source_checkout_path: fixture.selection.repository.root,
              source_workspace_id: "source",
            },
          }
        : {
            root_pane: {
              pane_id: "source:prep",
              workspace_id: "source",
              cwd: fixture.checkout,
            },
          },
    );
  });
  expect(await herdr.createPreparation(selection, "Preparation fixture")).toBe(
    "source:prep",
  );
  expect(calls.at(-1)).toContain(fixture.checkout);
});

test("refuses a different native repository before any topology mutation", async () => {
  const fixture = issueFixture();
  const other = issueFixture();
  fixtures.push(fixture, other);
  const calls: (readonly string[])[] = [];
  const herdr = createNativeHerdr((arguments_: readonly string[]) => {
    calls.push(arguments_);
    return Promise.resolve({
      source: {
        repo_key: other.selection.repository.commonDirectory,
        source_checkout_path: other.selection.repository.root,
        source_workspace_id: "other",
      },
    });
  });
  const result = await herdr
    .createPreparation(fixture.selection, "Preparation fixture")
    .catch((error: unknown) => error);
  expect(result).toMatchObject({
    message: "Native Herdr resolved another source repository",
  });
  expect(calls).toHaveLength(1);
});

test("rejects a malformed pane after creation without cleanup or another creation", async () => {
  const fixture = issueFixture();
  fixtures.push(fixture);
  const calls: (readonly string[])[] = [];
  const herdr = createNativeHerdr((arguments_: readonly string[]) => {
    calls.push(arguments_);
    return Promise.resolve(
      arguments_[0] === "worktree"
        ? {
            source: {
              repo_key: fixture.selection.repository.commonDirectory,
              source_checkout_path: fixture.selection.repository.root,
              source_workspace_id: null,
            },
          }
        : { root_pane: { pane_id: "created-without-workspace-id" } },
    );
  });
  const results = await Promise.allSettled([
    herdr.createPreparation(fixture.selection, "Preparation fixture"),
  ]);
  expect(results[0]?.status).toBe("rejected");
  const queryAndCreationCount = 2;
  expect(calls).toHaveLength(queryAndCreationCount);
});

test("passes a lossless single-line initial prompt as data after native launch options", async () => {
  const prompt =
    "Apostrophe ' and $(touch /tmp/must-not-exist), `code`,\nnew line\u007F\u0085";
  const calls: (readonly string[])[] = [];
  const herdr = createNativeHerdr((arguments_: readonly string[]) => {
    calls.push(arguments_);
    return Promise.resolve({ type: "agent_started" });
  });
  await herdr.start({
    paneId: "fixture:p1",
    name: "fixture",
    kind: "codex",
    prompt,
  });
  const [call] = calls;
  const encoded = call?.at(-1);
  expect(call?.includes("--")).toBe(true);
  expect(typeof encoded).toBe("string");
  if (encoded === undefined) {
    throw new Error("Initial prompt missing");
  }
  expect(encoded).not.toContain("\n");
  expect(encoded).not.toContain("\u007F");
  expect(encoded).not.toContain("\u0085");
  const data = encoded.slice(encoded.indexOf(": ") + ": ".length);
  expect(JSON.parse(data)).toBe(prompt);
  expect(calls).toHaveLength(1);
});
