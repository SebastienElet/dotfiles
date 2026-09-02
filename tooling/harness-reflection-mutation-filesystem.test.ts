import { afterEach, expect, test } from "bun:test";
import { mkdtemp, readFile, rm, symlink, writeFile } from "node:fs/promises";
import { createRepositoryMutationAdapter } from "./harness-reflection-mutation-filesystem.ts";
import { join } from "node:path";
import { tmpdir } from "node:os";

const temporaryRoots: string[] = [];

const temporaryRoot = async (): Promise<string> => {
  const root = await mkdtemp(join(tmpdir(), "harness-mutation-"));
  temporaryRoots.push(root);
  return root;
};

afterEach(async () => {
  for (const root of temporaryRoots.splice(0)) {
    await rm(root, { force: true, recursive: true });
  }
});

test("applies content compare-and-swap on a regular filesystem file", async () => {
  const root = await temporaryRoot();
  await writeFile(join(root, "surface.md"), "old surface", "utf8");
  const adapter = createRepositoryMutationAdapter(root);

  expect(
    adapter.compareAndSwap("surface.md", "old surface", "new surface"),
  ).toBe(true);
  expect(await readFile(join(root, "surface.md"), "utf8")).toBe("new surface");
  expect(
    adapter.compareAndSwap("surface.md", "old surface", "other surface"),
  ).toBe(false);
  expect(await readFile(join(root, "surface.md"), "utf8")).toBe("new surface");
});

test("validates registry JSON and surface shape inside the repository", async () => {
  const root = await temporaryRoot();
  const adapter = createRepositoryMutationAdapter(root);

  expect(
    adapter.validatePreparedRegistry(
      JSON.stringify({ invariants: [], version: 1 }),
    ),
  ).toEqual({ invariants: [], version: 1 });
  expect(() =>
    adapter.validatePreparedSurfaces(
      [{ contents: "surface", path: "surface.md" }],
      "approved-mutation",
    ),
  ).not.toThrow();
  expect(() =>
    adapter.validatePreparedSurfaces([], "approved-mutation"),
  ).toThrow("prepared-surface-count-invalid");
});

test("refuses a final surface symlink", async () => {
  const root = await temporaryRoot();
  await writeFile(join(root, "target.md"), "target", "utf8");
  await symlink("target.md", join(root, "surface.md"));
  const adapter = createRepositoryMutationAdapter(root);

  expect(() => adapter.read("surface.md")).toThrow(
    "mutation-path-not-regular-file",
  );
});
