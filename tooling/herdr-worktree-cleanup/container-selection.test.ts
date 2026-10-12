import { afterEach, expect, test } from "bun:test";
import { mkdirSync, mkdtempSync, rmSync, symlinkSync } from "node:fs";
import type { Container } from "./container-selection.ts";
import { fixtureContainer } from "./cleanup-test-support.ts";
import { join } from "node:path";
import { selectWorktreeContainers } from "./container-selection.ts";
import { tmpdir } from "node:os";

type Paths = Readonly<{
  root: string;
  checkout: string;
  child: string;
  neighbor: string;
  alias: string;
}>;
const roots: string[] = [];
afterEach(() => {
  for (const root of roots.splice(0)) {
    rmSync(root, { recursive: true, force: true });
  }
});

function paths(): Paths {
  const root = mkdtempSync(join(tmpdir(), "container-attribution-"));
  roots.push(root);
  const checkout = join(root, "work tree");
  const child = join(checkout, "app");
  const neighbor = join(root, "work tree-other");
  const alias = join(root, "alias");
  mkdirSync(child, { recursive: true });
  mkdirSync(neighbor);
  symlinkSync(checkout, alias);
  symlinkSync(neighbor, join(checkout, "outside"));
  return { root, checkout, child, neighbor, alias };
}

test.each(["exact", "child", "symlink"])(
  "selects a Compose %s path, with spaces and a colliding project name",
  (kind) => {
    const { checkout, child, neighbor, alias } = paths();
    const variants: Readonly<Record<string, string>> = {
      exact: checkout,
      child,
      symlink: alias,
    };
    const owned = fixtureContainer("a", variants[kind] ?? checkout);
    const other = fixtureContainer("b", neighbor);
    const result = selectWorktreeContainers([owned, other], checkout);
    expect(result.containers.map((container) => container.Id)).toEqual([
      owned.Id,
    ]);
    expect(result.skipped).toHaveLength(1);
  },
);

test.each(["parent", "neighbor", "relative", "missing", "symlink-outside"])(
  "skips unsafe Compose path: %s",
  (kind) => {
    const { root, checkout, neighbor } = paths();
    const variants: Readonly<Record<string, string>> = {
      parent: root,
      neighbor,
      relative: ".",
      missing: join(root, "gone"),
      "symlink-outside": join(checkout, "outside"),
    };
    const container = fixtureContainer("a", variants[kind] ?? "");
    const result = selectWorktreeContainers([container], checkout);
    expect(result.containers).toEqual([]);
    expect(result.skipped[0]).toContain(container.Id);
  },
);

function uncertainContainers(
  checkout: string,
  neighbor: string,
): Readonly<Record<string, Container>> {
  const container = fixtureContainer("a", checkout);
  return {
    conflict: {
      ...container,
      Config: {
        Labels: {
          ...container.Config.Labels,
          "dotfiles.worktree.path": neighbor,
        },
      },
    },
    shared: {
      ...container,
      Config: {
        Labels: {
          ...container.Config.Labels,
          "dotfiles.worktree.shared": "true",
        },
      },
    },
    "bind-only": {
      ...container,
      Config: { Labels: null },
      Mounts: [{ Type: "bind", Source: checkout }],
    },
    "name-only": {
      ...container,
      Config: { Labels: { "com.docker.compose.project": "same-project" } },
    },
    "no-labels": { ...container, Config: { Labels: null } },
    "outside-bind": {
      ...container,
      Mounts: [{ Type: "bind", Source: neighbor }],
    },
    "unknown-mount": {
      ...container,
      Mounts: [{ Type: "unknown", Source: neighbor }],
    },
    "unnamed-volume": { ...container, Mounts: [{ Type: "volume", Name: "" }] },
  };
}

test.each([
  "conflict",
  "shared",
  "bind-only",
  "name-only",
  "no-labels",
  "outside-bind",
  "unknown-mount",
  "unnamed-volume",
])("skips uncertain attribution: %s", (kind) => {
  const { checkout, neighbor } = paths();
  const container = uncertainContainers(checkout, neighbor)[kind];
  expect(container).toBeDefined();
  const result = selectWorktreeContainers(
    container === undefined ? [] : [container],
    checkout,
  );
  expect(result.containers).toEqual([]);
  expect(result.skipped).toHaveLength(1);
});

test("accepts an explicitly attributed bind mount", () => {
  const { checkout } = paths();
  const container = {
    ...fixtureContainer("a", checkout),
    Config: { Labels: { "dotfiles.worktree.path": checkout } },
    Mounts: [{ Type: "bind", Source: checkout }],
  };
  expect(selectWorktreeContainers([container], checkout).containers).toEqual([
    container,
  ]);
});

test.each(["bind", "volume"])(
  "refuses a %s mount shared with an unattributed container",
  (type) => {
    const { checkout } = paths();
    const mounts = [{ Type: type, Source: checkout, Name: "shared-data" }];
    const owned = { ...fixtureContainer("a", checkout), Mounts: mounts };
    const shared = {
      ...fixtureContainer("b", checkout),
      Config: { Labels: null },
      Mounts: mounts,
    };
    expect(
      selectWorktreeContainers([owned, shared], checkout).containers,
    ).toEqual([]);
  },
);
