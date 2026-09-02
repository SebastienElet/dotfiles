import { afterEach, expect, test } from "bun:test";
import { join, resolve } from "node:path";
import {
  link,
  mkdir,
  mkdtemp,
  readFile,
  readdir,
  rm,
  stat,
  symlink,
  writeFile,
} from "node:fs/promises";
import type { MutationWorkflowAdapter } from "./harness-reflection-mutation-workflow-types.ts";
import { candidate } from "./invariant-registry-test-support.ts";
import { createRepositoryMutationAdapter } from "./harness-reflection-mutation-filesystem.ts";
import { parseInvariantRegistry } from "./invariant-registry-contract.ts";
import { pathToFileURL } from "node:url";
import { tmpdir } from "node:os";

const temporaryRoots: string[] = [];
const hardStopExitCode = 91;
const mutationModuleUrl = pathToFileURL(
  resolve(import.meta.dir, "harness-reflection-mutation-filesystem.ts"),
).href;

const adapterWithRename = (
  root: string,
  renameFile: (source: string, target: string) => void,
): MutationWorkflowAdapter =>
  createRepositoryMutationAdapter(root, { renameFile });

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

test("replaces only matching content while the adapter lock is held", async () => {
  const root = await temporaryRoot();
  await writeFile(join(root, "surface.md"), "old surface", "utf8");
  const adapter = createRepositoryMutationAdapter(root);

  await adapter.withMutationLock(async () => {
    expect(
      await adapter.replaceMatching("surface.md", "old surface", "new surface"),
    ).toBe(true);
  });
  expect(await readFile(join(root, "surface.md"), "utf8")).toBe("new surface");
  await adapter.withMutationLock(async () => {
    expect(
      await adapter.replaceMatching(
        "surface.md",
        "old surface",
        "other surface",
      ),
    ).toBe(false);
  });
  expect(await readFile(join(root, "surface.md"), "utf8")).toBe("new surface");
});

test("validates registry JSON and surface shape inside the repository", async () => {
  const root = await temporaryRoot();
  await mkdir(join(root, "harness"));
  const adapter = createRepositoryMutationAdapter(root);
  const [target] = parseInvariantRegistry({
    invariants: [candidate()],
    version: 1,
  }).invariants;
  if (target === undefined) {
    throw new Error("mutation-target-missing");
  }
  const transition = { kind: "approved-mutation", target } as const;

  expect(
    adapter.validatePreparedRegistry(
      JSON.stringify({ invariants: [], version: 1 }),
    ),
  ).toEqual({ invariants: [], version: 1 });
  expect(() =>
    adapter.validatePreparedSurfaces(
      [{ contents: "surface", path: "harness/AGENTS.md" }],
      transition,
    ),
  ).not.toThrow();
  expect(() => adapter.validatePreparedSurfaces([], transition)).toThrow(
    "prepared-surface-count-invalid",
  );
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

test("refuses an internal hardlink to an external file", async () => {
  const repositoryRoot = await temporaryRoot();
  const externalRoot = await temporaryRoot();
  const externalPath = join(externalRoot, "external.md");
  await writeFile(externalPath, "external contents", "utf8");
  await link(externalPath, join(repositoryRoot, "surface.md"));
  const adapter = createRepositoryMutationAdapter(repositoryRoot);

  expect(() => adapter.read("surface.md")).toThrow("mutation-path-hard-linked");
  expect(await readFile(externalPath, "utf8")).toBe("external contents");
});

test("replaces an existing file with a new inode", async () => {
  const root = await temporaryRoot();
  const path = join(root, "surface.md");
  await writeFile(path, "old surface", "utf8");
  const before = await stat(path, { bigint: true });
  const adapter = createRepositoryMutationAdapter(root);

  await adapter.withMutationLock(() =>
    adapter.replaceMatching("surface.md", "old surface", "new surface"),
  );

  const after = await stat(path, { bigint: true });
  expect(after.ino).not.toBe(before.ino);
  expect(await readFile(path, "utf8")).toBe("new surface");
});

test("keeps the target intact when rename fails", async () => {
  const root = await temporaryRoot();
  const path = join(root, "surface.md");
  await writeFile(path, "old surface", "utf8");
  const adapter = adapterWithRename(root, () => {
    throw new Error("injected-rename-failure");
  });

  expect(
    adapter.withMutationLock(() =>
      adapter.replaceMatching("surface.md", "old surface", "new surface"),
    ),
  ).rejects.toThrow("injected-rename-failure");
  expect(await readFile(path, "utf8")).toBe("old surface");
  expect(await readdir(root)).toEqual(["surface.md"]);
});

test("leaves the target intact after a hard stop before rename", async () => {
  const root = await temporaryRoot();
  const path = join(root, "surface.md");
  await writeFile(path, "old surface", "utf8");
  const script = `
    import { createRepositoryMutationAdapter } from ${JSON.stringify(mutationModuleUrl)};
    const adapter = createRepositoryMutationAdapter(${JSON.stringify(root)}, {
      renameFile: () => process.exit(${hardStopExitCode}),
    });
    await adapter.withMutationLock(() =>
      adapter.replaceMatching("surface.md", "old surface", "new surface"),
    );
  `;

  const result = Bun.spawnSync([
    Bun.argv[0] ?? "bun",
    "--no-env-file",
    "-e",
    script,
  ]);

  expect(result.exitCode).toBe(hardStopExitCode);
  expect(await readFile(path, "utf8")).toBe("old surface");
  const entries = await readdir(root);
  expect(entries).toContain(".harness-reflection-mutation.lock");
  expect(entries.some((entry) => entry.endsWith(".tmp"))).toBe(true);
});
