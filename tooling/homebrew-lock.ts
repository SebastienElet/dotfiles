import { z } from "zod";

type Kind = "formulae" | "casks";
type Versions = Readonly<Record<string, string>>;
type Lock = Readonly<Record<Kind, Versions>>;
type Entry = readonly [string, string];
type Package = Readonly<{
  names: readonly string[];
  available: string;
  installed: string | undefined;
}>;

const kinds: readonly Kind[] = ["formulae", "casks"];
const argumentOffset = 2;
const brewfileOffset = 2;
const tapQualifiedSegments = 3;
const jsonIndentation = 2;
// ADR-013 mandates the WezTerm nightly cask, which Homebrew only publishes as version "latest".
const unlockedCasks = new Set(["claude", "wezterm@nightly"]);
const exactVersion = z
  .string()
  .regex(/^[0-9][^\s]*$/u, "expected an exact version");
const lockSchema = z
  .object({
    formulae: z.record(z.string(), exactVersion),
    casks: z.record(z.string(), exactVersion),
  })
  .strict();
const formulaSchema = z
  .object({
    name: z.string(),
    full_name: z.string(),
    versions: z.object({ stable: z.string() }).readonly(),
    revision: z.number().int().nonnegative(),
    linked_keg: z.string().nullable(),
    installed: z.array(z.object({ version: z.string() }).readonly()).readonly(),
  })
  .readonly();
const caskSchema = z.object({
  token: z.string(),
  full_token: z.string(),
  version: z.string(),
  installed: z.string().nullable(),
});
const infoSchema = z.object({
  formulae: z.array(formulaSchema),
  casks: z.array(caskSchema),
});
const invocationSchema = z.union([
  z.tuple([z.literal("verify-catalog"), z.string()]).rest(z.string()),
  z.tuple([z.literal("verify-installed"), z.string()]),
  z.tuple([z.literal("refresh"), z.string()]),
]);

function brew(
  commandArguments: readonly string[],
  stderr: "inherit" | "ignore" = "inherit",
): string {
  const result = Bun.spawnSync(["brew", ...commandArguments], {
    env: { ...process.env, HOMEBREW_NO_AUTO_UPDATE: "1" },
    stderr,
  });
  if (result.exitCode !== 0) {
    throw new Error(`brew ${commandArguments.join(" ")} failed`);
  }
  return result.stdout.toString();
}

function parsePackages(output: string): Record<Kind, Package[]> {
  const info = infoSchema.parse(JSON.parse(output));
  return {
    formulae: info.formulae.map((formula) => ({
      names: [formula.full_name, formula.name],
      available:
        formula.revision === 0
          ? formula.versions.stable
          : `${formula.versions.stable}_${formula.revision}`,
      installed: formula.linked_keg ?? formula.installed.at(-1)?.version,
    })),
    casks: info.casks.map((cask) => ({
      names: [cask.full_token, cask.token],
      available: cask.version,
      installed: cask.installed ?? undefined,
    })),
  };
}

function find(packages: readonly Package[], name: string): Package | undefined {
  return packages.find((candidate) => candidate.names.includes(name));
}

function isTapQualified(name: string): boolean {
  return name.split("/").length === tapQualifiedSegments;
}

function infoFor(
  kind: Kind,
  names: readonly string[],
  stderr: "inherit" | "ignore" = "inherit",
): Package[] {
  const flag = kind === "formulae" ? "--formula" : "--cask";
  return parsePackages(brew(["info", "--json=v2", flag, ...names], stderr))[
    kind
  ];
}

function tapPackage(kind: Kind, name: string): Package[] {
  try {
    return infoFor(kind, [name], "ignore");
  } catch {
    return [];
  }
}

function queryCatalog(lock: Lock): Record<Kind, Package[]> {
  const query = (kind: Kind): Package[] => {
    const names = Object.keys(lock[kind]);
    const core = names.filter((name) => !isTapQualified(name));
    return [
      ...(core.length === 0 ? [] : infoFor(kind, core)),
      ...names
        .filter((name) => isTapQualified(name))
        .flatMap((name) => tapPackage(kind, name)),
    ];
  };
  return { formulae: query("formulae"), casks: query("casks") };
}

function unlockedDeclarations(
  lock: Lock,
  brewfiles: readonly string[],
): string[] {
  const locked = (kind: Kind, name: string): boolean =>
    Object.keys(lock[kind]).some(
      (key) => key === name || key.endsWith(`/${name}`),
    );
  return brewfiles.flatMap((brewfile) =>
    kinds.flatMap((kind) =>
      brew([
        "bundle",
        "list",
        kind === "formulae" ? "--formula" : "--cask",
        `--file=${brewfile}`,
      ])
        .split("\n")
        .filter((name) => name !== "")
        .filter((name) => !(kind === "casks" && unlockedCasks.has(name)))
        .filter((name) => !locked(kind, name))
        .map((name) => `${brewfile}: ${name} has no locked version`),
    ),
  );
}

function verifyCatalog(lock: Lock, brewfiles: readonly string[]): string[] {
  const catalog = queryCatalog(lock);
  const mismatches = kinds.flatMap((kind) =>
    Object.entries(lock[kind]).flatMap(([name, version]: Entry) => {
      const available = find(catalog[kind], name)?.available;
      if (available === undefined && isTapQualified(name)) {
        return [];
      }
      return available === version
        ? []
        : [`${name}: locked ${version}, catalog ${available ?? "missing"}`];
    }),
  );
  return [...unlockedDeclarations(lock, brewfiles), ...mismatches];
}

function verifyInstalled(lock: Lock): string[] {
  const installed = parsePackages(brew(["info", "--json=v2", "--installed"]));
  return kinds.flatMap((kind) =>
    Object.entries(lock[kind]).flatMap(([name, version]: Entry) => {
      const current = find(installed[kind], name)?.installed;
      return current === undefined || current === version
        ? []
        : [`${name}: locked ${version}, installed ${current}`];
    }),
  );
}

async function refresh(path: string, lock: Lock): Promise<void> {
  const catalog = queryCatalog(lock);
  const refreshed = Object.fromEntries(
    kinds.map((kind) => [
      kind,
      Object.fromEntries(
        Object.entries(lock[kind]).map(([name, version]: Entry) => [
          name,
          find(catalog[kind], name)?.available ?? version,
        ]),
      ),
    ]),
  );
  await Bun.write(
    path,
    `${JSON.stringify(lockSchema.parse(refreshed), null, jsonIndentation)}\n`,
  );
}

async function main(): Promise<number> {
  const invocation = invocationSchema.parse(Bun.argv.slice(argumentOffset));
  const [command, path] = invocation;
  const lock = lockSchema.parse(await Bun.file(path).json());
  if (command === "refresh") {
    await refresh(path, lock);
    return 0;
  }
  const failures =
    command === "verify-catalog"
      ? verifyCatalog(lock, invocation.slice(brewfileOffset))
      : verifyInstalled(lock);
  if (failures.length === 0) {
    return 0;
  }
  for (const failure of failures) {
    process.stderr.write(`homebrew-lock: ${failure}\n`);
  }
  process.stderr.write(
    `homebrew-lock: bump ${path} explicitly with \`tooling/upgrade\` or \`bun tooling/homebrew-lock.ts refresh ${path}\` after \`brew update\`\n`,
  );
  return 1;
}

try {
  process.exitCode = await main();
} catch (error) {
  process.stderr.write(
    `homebrew-lock: ${error instanceof Error ? error.message : String(error)}\n`,
  );
  process.exitCode = 1;
}
