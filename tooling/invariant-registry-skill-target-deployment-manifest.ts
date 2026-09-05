import type { ConsumerName } from "./invariant-registry-validation-options.ts";

const canonicalMakefileSha256 =
  "92561b1c29588f1d6f16038b79d8e98260278a154cf300bffbcce36ddf8e76a6";
const deploymentRoutes: Readonly<
  Record<ConsumerName, Readonly<{ aggregate: string; destination: string }>>
> = {
  claude: { aggregate: "claude-code", destination: ".claude" },
  codex: { aggregate: "codex", destination: ".agents" },
  cursor: { aggregate: "cursor", destination: ".cursor" },
};
const createSymlinkRecipe = `\t@\${CREATE_SYMLINK}`;

type MakefileSnapshot = Readonly<{
  lines: readonly string[];
  sha256: string;
}>;

const hasExactRoute = (lines: readonly string[], header: string): boolean => {
  let occurrences = 0;
  for (let index = 0; index < lines.length; index += 1) {
    if (lines[index] === header && lines[index + 1] === createSymlinkRecipe) {
      occurrences += 1;
    }
  }
  return occurrences === 1;
};

const aggregateIncludesTarget = (
  lines: readonly string[],
  aggregate: string,
  target: string,
): boolean => {
  const prefix = `${aggregate}: `;
  const matches = lines.filter((line) => line.startsWith(prefix));
  if (matches.length !== 1 || matches[0] === undefined) {
    return false;
  }
  return (
    matches[0]
      .slice(prefix.length)
      .split(" ")
      .filter((dependency) => dependency === target).length === 1
  );
};

const makefileDeclaresDeployment = (
  snapshot: MakefileSnapshot,
  agent: ConsumerName,
  slug: string,
): boolean => {
  const route = deploymentRoutes[agent];
  const target = `~/${route.destination}/skills/${slug}`;
  const header = `${target}: \${DOTFILES_PATH}/harness/skills/${slug} FORCE | ~/${route.destination}/skills`;
  return (
    hasExactRoute(snapshot.lines, header) &&
    aggregateIncludesTarget(snapshot.lines, route.aggregate, target)
  );
};

const inspectCanonicalMakefileDeployments = (
  snapshot: MakefileSnapshot,
  slug: string,
  declaredFor: readonly ConsumerName[],
): readonly ConsumerName[] | undefined => {
  if (snapshot.sha256 !== canonicalMakefileSha256) {
    return undefined;
  }
  const installedFor = declaredFor.filter((agent) =>
    makefileDeclaresDeployment(snapshot, agent, slug),
  );
  return installedFor.length === declaredFor.length ? installedFor : undefined;
};

export { inspectCanonicalMakefileDeployments };
export type { MakefileSnapshot };
