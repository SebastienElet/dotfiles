import { afterEach, expect, test } from "bun:test";
import { mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { tmpdir } from "node:os";

type Formula = Readonly<{
  name: string;
  full_name: string;
  stable: string;
  revision?: number;
  installed?: string;
}>;
type Cask = Readonly<{
  token: string;
  full_token: string;
  version: string;
  installed?: string;
}>;
type Catalog = Readonly<{
  formulae: readonly Formula[];
  casks: readonly Cask[];
  untrusted?: readonly string[];
  brewfile?: Readonly<{
    formulae: readonly string[];
    casks: readonly string[];
  }>;
}>;

const fixtures: string[] = [];
const fakeBrew = `#!/usr/bin/env bun
const catalog = await Bun.file(process.env.FAKE_BREW_CATALOG).json();
const args = Bun.argv.slice(2);
const formula = (f) => ({
  name: f.name,
  full_name: f.full_name,
  versions: { stable: f.stable },
  revision: f.revision ?? 0,
  linked_keg: f.installed ?? null,
  installed: f.installed === undefined ? [] : [{ version: f.installed }],
});
const cask = (c) => ({
  token: c.token,
  full_token: c.full_token,
  version: c.version,
  installed: c.installed ?? null,
});
if (args[0] === "bundle") {
  const kind = args[2] === "--formula" ? "formulae" : "casks";
  process.stdout.write(catalog.brewfile[kind].map((name) => name + "\\n").join(""));
  process.exit(0);
}
if (args[2] === "--installed") {
  process.stdout.write(JSON.stringify({
    formulae: catalog.formulae.filter((f) => f.installed !== undefined).map(formula),
    casks: catalog.casks.filter((c) => c.installed !== undefined).map(cask),
  }));
  process.exit(0);
}
const kind = args[2] === "--formula" ? "formulae" : "casks";
const names = args.slice(3);
if (names.some((name) => (catalog.untrusted ?? []).includes(name))) {
  process.exit(1);
}
const known = catalog[kind].filter((p) => names.includes(p.full_name ?? p.full_token));
process.stdout.write(JSON.stringify({
  formulae: kind === "formulae" ? known.map(formula) : [],
  casks: kind === "casks" ? known.map(cask) : [],
}));
`;

afterEach(() => {
  for (const root of fixtures.splice(0)) {
    rmSync(root, { force: true, recursive: true });
  }
});

function createFixture(lock: unknown, catalog: Catalog): string {
  const root = mkdtempSync(join(tmpdir(), "homebrew-lock-test-"));
  fixtures.push(root);
  writeFileSync(join(root, "brew"), fakeBrew, { mode: 0o755 });
  writeFileSync(join(root, "catalog.json"), JSON.stringify(catalog));
  writeFileSync(join(root, "homebrew.lock.json"), JSON.stringify(lock));
  return root;
}

function run(
  root: string,
  ...commandArguments: readonly string[]
): Readonly<{ exitCode: number; stdout: string; stderr: string }> {
  const result = Bun.spawnSync(
    [
      process.execPath,
      join(import.meta.dir, "homebrew-lock.ts"),
      ...commandArguments.map((argument) =>
        argument === "LOCK" ? join(root, "homebrew.lock.json") : argument,
      ),
    ],
    {
      env: {
        ...process.env,
        FAKE_BREW_CATALOG: join(root, "catalog.json"),
        PATH: `${root}:${process.env.PATH ?? ""}`,
      },
      stderr: "pipe",
      stdout: "pipe",
    },
  );
  return {
    exitCode: result.exitCode,
    stdout: result.stdout.toString(),
    stderr: result.stderr.toString(),
  };
}

const lock = {
  formulae: { bat: "0.26.1", mosh: "1.4.0_43", "owner/tap/tool": "1.0.0" },
  casks: { orbstack: "2.2.5,21124" },
};
const catalog: Catalog = {
  formulae: [
    { name: "bat", full_name: "bat", stable: "0.26.1" },
    { name: "mosh", full_name: "mosh", stable: "1.4.0", revision: 43 },
    { name: "tool", full_name: "owner/tap/tool", stable: "1.0.0" },
  ],
  casks: [
    { token: "orbstack", full_token: "orbstack", version: "2.2.5,21124" },
  ],
  brewfile: {
    formulae: ["bat", "owner/tap/tool"],
    casks: ["orbstack", "claude"],
  },
};

test("accepts a catalog that matches every locked version", () => {
  const root = createFixture(lock, catalog);

  expect(run(root, "verify-catalog", "LOCK", "Brewfile")).toEqual({
    exitCode: 0,
    stdout: "",
    stderr: "",
  });
});

test("refuses a catalog version that differs from the lock", () => {
  const root = createFixture(lock, {
    ...catalog,
    formulae: [
      ...catalog.formulae.filter((formula) => formula.name !== "bat"),
      { name: "bat", full_name: "bat", stable: "0.27.0" },
    ],
  });

  const result = run(root, "verify-catalog", "LOCK", "Brewfile");

  expect(result.exitCode).toBe(1);
  expect(result.stderr).toContain("bat: locked 0.26.1, catalog 0.27.0");
});

test("refuses a Brewfile entry without a locked version", () => {
  const root = createFixture(lock, {
    ...catalog,
    brewfile: { formulae: ["bat", "fd"], casks: [] },
  });

  const result = run(root, "verify-catalog", "LOCK", "Brewfile");

  expect(result.exitCode).toBe(1);
  expect(result.stderr).toContain("Brewfile: fd has no locked version");
});

test.each([
  { kind: "formulae", name: "owner/tap/tool" },
  { kind: "casks", name: "owner/tap/app" },
] as const)(
  "silently defers an unavailable tap $kind on repeated checks",
  ({ kind, name }) => {
    const root = createFixture(
      { ...lock, [kind]: { ...lock[kind], [name]: "1.0.0" } },
      { ...catalog, untrusted: [name] },
    );
    const expected = { exitCode: 0, stdout: "", stderr: "" };

    expect(run(root, "verify-catalog", "LOCK", "Brewfile")).toEqual(expected);
    expect(run(root, "verify-catalog", "LOCK", "Brewfile")).toEqual(expected);
  },
);

test("refuses an installed version that drifted from the lock", () => {
  const root = createFixture(lock, {
    ...catalog,
    formulae: [
      { name: "bat", full_name: "bat", stable: "0.27.0", installed: "0.27.0" },
      {
        name: "mosh",
        full_name: "mosh",
        stable: "1.4.0",
        installed: "1.4.0_43",
      },
    ],
    casks: [],
  });

  const result = run(root, "verify-installed", "LOCK");

  expect(result.exitCode).toBe(1);
  expect(result.stderr).toContain("bat: locked 0.26.1, installed 0.27.0");
  expect(result.stderr).not.toContain("mosh");
});

test("ignores locked packages that are not installed", () => {
  const root = createFixture(lock, catalog);

  expect(run(root, "verify-installed", "LOCK")).toEqual({
    exitCode: 0,
    stdout: "",
    stderr: "",
  });
});

test("refresh records the catalog versions and keeps unreachable entries", () => {
  const root = createFixture(lock, {
    ...catalog,
    formulae: [
      { name: "bat", full_name: "bat", stable: "0.27.0" },
      { name: "mosh", full_name: "mosh", stable: "1.4.0", revision: 44 },
    ],
    untrusted: ["owner/tap/tool"],
  });

  expect(run(root, "refresh", "LOCK").exitCode).toBe(0);

  expect(
    JSON.parse(readFileSync(join(root, "homebrew.lock.json"), "utf8")),
  ).toEqual({
    formulae: { bat: "0.27.0", mosh: "1.4.0_44", "owner/tap/tool": "1.0.0" },
    casks: { orbstack: "2.2.5,21124" },
  });
});

test("rejects a lock entry that is not an exact version", () => {
  const root = createFixture(
    { formulae: {}, casks: { "wezterm@nightly": "latest" } },
    catalog,
  );

  const result = run(root, "verify-installed", "LOCK");

  expect(result.exitCode).toBe(1);
  expect(result.stderr).toContain("expected an exact version");
});
