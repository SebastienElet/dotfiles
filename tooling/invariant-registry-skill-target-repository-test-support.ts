import {
  active,
  marginalAblation,
  registry,
  targetSkillPath,
  verifiedVerification,
} from "./invariant-registry-test-support.ts";
import { join, resolve } from "node:path";
import {
  mkdir,
  mkdtemp,
  readFile,
  realpath,
  rm,
  symlink,
  writeFile,
} from "node:fs/promises";
import { tmpdir } from "node:os";

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
const canonicalMakefilePath = resolve(import.meta.dir, "../Makefile");
const cursorTarget = "~/.cursor/skills/enforcement-code";
const cursorRouteHeader = `${cursorTarget}: \${DOTFILES_PATH}/harness/skills/enforcement-code FORCE | ~/.cursor/skills`;
const cursorRoute = `${cursorRouteHeader}\n\t@\${CREATE_SYMLINK}`;

type ConsumerName = "claude" | "codex" | "cursor";
type MakefileMutation =
  | "altered-byte"
  | "fake-marker"
  | "inactive-route"
  | "malicious-target-override"
  | "missing-aggregate"
  | "missing-route"
  | "wrong-route";
type FixtureOptions = Readonly<{
  category?: string;
  description?: string;
  installedFor?: readonly ConsumerName[];
  makefileMutation?: MakefileMutation;
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

const makeExecutionMarker = (root: string): string =>
  join(root, "make-executed");

const removeCursorAggregateEntry = (source: string): string =>
  source
    .split("\n")
    .map((line) =>
      line.startsWith("cursor: ") ? line.replace(` ${cursorTarget}`, "") : line,
    )
    .join("\n");

const mutateMakefile = (
  source: string,
  mutation: MakefileMutation | undefined,
  root: string,
): string => {
  switch (mutation) {
    case "altered-byte": {
      return `${source}\n`;
    }
    case "fake-marker": {
      return source.replace(
        cursorRoute,
        `${cursorRouteHeader}\n\t@echo __INVARIANT_REGISTRY_SOURCE__${root}/harness/skills/enforcement-code`,
      );
    }
    case "inactive-route": {
      return source.replace(cursorRoute, `ifeq (1,0)\n${cursorRoute}\nendif`);
    }
    case "malicious-target-override": {
      const fakeSource = `${root}/harness/skills/enforcement-code`;
      const maliciousMake = `/usr/bin/printf 'echo __INVARIANT_%s%s\\n' 'REGISTRY_' 'SOURCE__${fakeSource}'; /usr/bin/touch '${makeExecutionMarker(root)}'`;
      return source.replace(
        cursorRoute,
        `${cursorTarget}: override SHELL = /bin/sh\n${cursorTarget}: override MAKE = ${maliciousMake}\n${cursorRouteHeader}\n\t@\${MAKE}`,
      );
    }
    case "missing-aggregate": {
      return removeCursorAggregateEntry(source);
    }
    case "missing-route": {
      return source.replace(cursorRoute, "");
    }
    case "wrong-route": {
      return source.replace(
        cursorRoute,
        cursorRoute.replace(
          `\${DOTFILES_PATH}/harness/skills/enforcement-code`,
          `\${DOTFILES_PATH}/harness/skills`,
        ),
      );
    }
    case undefined: {
      return source;
    }
    default: {
      throw new Error(`unsupported Makefile mutation: ${String(mutation)}`);
    }
  }
};

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
  const canonicalMakefile = await readFile(canonicalMakefilePath, "utf8");
  const canonicalRoot = await realpath(root);
  await Promise.all([
    writeFile(
      join(root, "home/.arnes.yaml"),
      manifestSource(options.installedFor ?? ["claude", "codex", "cursor"]),
    ),
    writeFile(
      join(root, "Makefile"),
      mutateMakefile(
        canonicalMakefile,
        options.makefileMutation,
        canonicalRoot,
      ),
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

const cleanupFixtures = async (): Promise<void> => {
  await Promise.all(
    temporaryRoots
      .splice(0)
      .map((root) => rm(root, { force: true, recursive: true })),
  );
};

export {
  cleanupFixtures,
  conditionalRegistryText,
  initializeFixture,
  makeExecutionMarker,
};
export type { FixtureOptions, MakefileMutation };
