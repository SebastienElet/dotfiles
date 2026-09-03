import {
  active,
  marginalAblation,
  registry,
  targetSkillPath,
  verifiedVerification,
} from "./invariant-registry-test-support.ts";
import { afterEach, expect, test } from "bun:test";
import { mkdir, mkdtemp, rm, symlink, writeFile } from "node:fs/promises";
import { join } from "node:path";
import { tmpdir } from "node:os";
import { validateInvariantRegistryText } from "./invariant-registry-repository-validator.ts";

const temporaryRoots: string[] = [];
const effectiveAblation = {
  ...marginalAblation,
  conditionalSkillActivation: {
    with: { activated: 6, total: 6 },
    without: { activated: 0, total: 6 },
  },
};
const triggerableDescription =
  "Write refusal controls. Use when a gate changes. Make sure to use this skill whenever bypass resistance matters, even if no guard is named.";
const skillDestinations = {
  claude: ".claude",
  codex: ".agents",
  cursor: ".cursor",
} as const;

type ConsumerName = keyof typeof skillDestinations;

type FixtureOptions = Readonly<{
  category?: string;
  description?: string;
  installedFor?: readonly ConsumerName[];
  linkedFor?: readonly ConsumerName[];
  name?: string;
  target?: "missing" | "regular" | "symlink";
  tracked?: boolean;
}>;

const skillSource = (
  name: string,
  description: string,
  category: string,
): string => `---
name: ${name}
description: >
  ${description}
metadata:
  category: ${category}
---

# Test Skill
`;

const manifestSource = (agents: readonly ConsumerName[]): string => `version: 1
skills:
  - slug: enforcement-code
    installations:
${agents.map((agent) => `      - { agent: ${agent}, scope: user }`).join("\n")}
`;

const makefileSource = (agents: readonly ConsumerName[]): string =>
  agents
    .map((agent) => {
      const directory = skillDestinations[agent];
      return `~/${directory}/skills/enforcement-code: \${DOTFILES_PATH}/harness/skills/enforcement-code FORCE | ~/${directory}/skills\n\t@\${CREATE_SYMLINK}\n`;
    })
    .join("");

const conditionalRegistryText = (target: string = targetSkillPath): string =>
  JSON.stringify(
    registry(
      active({
        controlKind: "probabilistic",
        marginalAblation: effectiveAblation,
        oracle: undefined,
        surface: "conditional-skill",
        targetSkillPath: target,
        verification: verifiedVerification,
      }),
    ),
  );

const writeDeploymentFixture = async (
  root: string,
  options: FixtureOptions,
): Promise<void> => {
  await Promise.all([
    writeFile(
      join(root, "home/.arnes.yaml"),
      manifestSource(options.installedFor ?? ["claude", "codex", "cursor"]),
    ),
    writeFile(
      join(root, "Makefile"),
      makefileSource(options.linkedFor ?? ["claude", "codex", "cursor"]),
    ),
  ]);
};

const writeSkillTarget = async (
  root: string,
  options: FixtureOptions,
): Promise<void> => {
  if (options.target === "missing") {
    return;
  }
  const skillPath = join(root, targetSkillPath);
  const source = skillSource(
    options.name ?? "enforcement-code",
    options.description ?? triggerableDescription,
    options.category ?? "dev",
  );
  if (options.target === "symlink") {
    const actualPath = join(root, "actual-skill.md");
    await writeFile(actualPath, source);
    await symlink(actualPath, skillPath);
    return;
  }
  await writeFile(skillPath, source);
};

const trackFixture = (root: string, options: FixtureOptions): void => {
  const initialized = Bun.spawnSync(["git", "-C", root, "init", "--quiet"]);
  if (initialized.exitCode !== 0) {
    throw new Error("fixture-git-init-failed");
  }
  const files = ["home/.arnes.yaml", "Makefile"];
  if (options.target !== "missing" && options.tracked !== false) {
    files.push(targetSkillPath);
  }
  const added = Bun.spawnSync(["git", "-C", root, "add", "--", ...files]);
  if (added.exitCode !== 0) {
    throw new Error("fixture-git-add-failed");
  }
};

const initializeFixture = async (
  options: FixtureOptions = {},
): Promise<string> => {
  const root = await mkdtemp(join(tmpdir(), "skill-target-"));
  temporaryRoots.push(root);
  await Promise.all([
    mkdir(join(root, "home"), { recursive: true }),
    mkdir(join(root, targetSkillPath, ".."), { recursive: true }),
  ]);
  await writeDeploymentFixture(root, options);
  await writeSkillTarget(root, options);
  trackFixture(root, options);
  return root;
};

afterEach(async () => {
  await Promise.all(
    temporaryRoots
      .splice(0)
      .map((root) => rm(root, { force: true, recursive: true })),
  );
});

test("validates a tracked triggerable target with three declared deployments", async () => {
  const root = await initializeFixture();
  expect(() =>
    validateInvariantRegistryText(conditionalRegistryText(), root),
  ).not.toThrow();
});

test.each([
  [
    "missing target",
    { target: "missing" },
    "Conditional skill target does not exist",
  ],
  [
    "untracked target",
    { tracked: false },
    "Conditional skill target must be tracked by Git",
  ],
  [
    "symlink target",
    { target: "symlink" },
    "Conditional skill target must be a regular file",
  ],
  [
    "invalid name",
    { name: "Different Skill" },
    "Conditional skill target frontmatter is invalid",
  ],
  [
    "invalid category",
    { category: "unknown" },
    "Conditional skill target frontmatter is invalid",
  ],
  [
    "non-triggerable description",
    { description: "A reusable helper." },
    "Conditional skill description must support implicit discovery",
  ],
  [
    "undeclared Cursor deployment",
    { installedFor: ["claude", "codex"] },
    "Declared user-skill consumer has no matching user deployment",
  ],
  [
    "missing Cursor deployment link",
    { linkedFor: ["claude", "codex"] },
    "Declared user-skill consumer has no matching user deployment",
  ],
] as const)("rejects %s", async (_name, options, expected) => {
  const root = await initializeFixture(options);
  expect(() =>
    validateInvariantRegistryText(conditionalRegistryText(), root),
  ).toThrow(expected);
});
