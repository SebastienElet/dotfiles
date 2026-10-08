import { afterEach, expect, test } from "bun:test";
import {
  lstatSync,
  mkdtempSync,
  readFileSync,
  rmSync,
  symlinkSync,
  writeFileSync,
} from "node:fs";
import { createBindingStore } from "./binding-store.ts";
import { join } from "node:path";
import { tmpdir } from "node:os";

const directories: string[] = [];
const longPathRepeatCount = 30;
const nestedGroupCount = 20;
const longTitleLength = 150;
afterEach(() => {
  for (const directory of directories.splice(0)) {
    rmSync(directory, { force: true, recursive: true });
  }
});
function storeDirectory(): string {
  const directory = mkdtempSync(join(tmpdir(), "herdr-bindings-"));
  directories.push(directory);
  return directory;
}
const identity = {
  issue_agent: "codex",
  issue_name: "fixture",
  issue_phase: "starting",
  issue_role: "work",
  issue_checkout: "/work",
  issue_repo: `/source/${"long".repeat(longPathRepeatCount)}/.git`,
  issue_workspace: "w2",
  issue_url: `https://gitlab.example/${"nested/".repeat(nestedGroupCount)}project/-/issues/17`,
  issue_title: ` ${"é".repeat(longTitleLength)}\n\u007F `,
};

test("another process reads complete issue identity without native token limits", async () => {
  const directory = storeDirectory();
  await createBindingStore(directory).patch("w2:p1", identity);
  await createBindingStore(directory).patch("w2:p1", {
    issue_session: "session-17",
  });
  const restored = await createBindingStore(directory).read();
  expect(restored["w2:p1"]).toEqual({
    ...identity,
    issue_session: "session-17",
  });
});

for (const contents of [
  "malformed{",
  '{"version":2,"bindings":{}}',
  '{"version":1,"bindings":{"w2:p1":{"unknown":"raw"}}}',
]) {
  test(`retains unknown or corrupt state without replacing it: ${contents}`, async () => {
    const directory = storeDirectory();
    const path = join(directory, "bindings.json");
    writeFileSync(path, contents);
    const [result] = await Promise.allSettled([
      createBindingStore(directory).patch("w2:p1", identity),
    ]);
    expect(result?.status).toBe("rejected");
    expect(readFileSync(path, "utf8")).toBe(contents);
  });
}

test("refuses a second different-pane owner even after the first write finished", async () => {
  const directory = storeDirectory();
  await createBindingStore(directory).patch("w2:p1", identity);
  const [result] = await Promise.allSettled([
    createBindingStore(directory).patch("w2:p2", identity),
  ]);
  expect(result?.status).toBe("rejected");
  expect(Object.keys(await createBindingStore(directory).read())).toHaveLength(
    1,
  );
});

for (const contents of [
  `{"version":1,"bindings":{"w2:p1":{"unknown":"raw"},"w2:p1":${JSON.stringify(identity)}}}`,
  `{"version":1,"bindings":{"w2:p1":${JSON.stringify(identity).replace('"issue_url":', '"issue_url":"https://unknown.invalid/raw","issue_url":')}}}`,
]) {
  test("rejects duplicate persisted keys and retains original bytes", async () => {
    const directory = storeDirectory();
    const path = join(directory, "bindings.json");
    writeFileSync(path, contents);
    const results = await Promise.allSettled([
      createBindingStore(directory).read(),
      createBindingStore(directory).patch("w2:p1", identity),
    ]);
    expect(
      results.every(
        ({ status }: Readonly<PromiseSettledResult<unknown>>) =>
          status === "rejected",
      ),
    ).toBe(true);
    expect(readFileSync(path, "utf8")).toBe(contents);
  });
}

test("refuses a dangling state symlink without replacing it", async () => {
  const directory = storeDirectory();
  const path = join(directory, "bindings.json");
  symlinkSync(join(directory, "missing-state"), path);
  const results = await Promise.allSettled([
    createBindingStore(directory).read(),
    createBindingStore(directory).patch("w2:p1", identity),
  ]);
  expect(
    results.every(
      ({ status }: Readonly<PromiseSettledResult<unknown>>) =>
        status === "rejected",
    ),
  ).toBe(true);
  expect(lstatSync(path).isSymbolicLink()).toBe(true);
});
