import type {
  ConsumerName,
  SkillTargetInspection,
} from "./invariant-registry-validation-options.ts";
import {
  type MakefileSnapshot,
  inspectCanonicalMakefileDeployments,
} from "./invariant-registry-skill-target-deployment-manifest.ts";
import { isAbsolute, relative, resolve, sep } from "node:path";
import { lstatSync, readFileSync, realpathSync } from "node:fs";
import { createHash } from "node:crypto";
import { inspectSkillFrontmatter } from "./invariant-registry-skill-target-frontmatter.ts";
import { z } from "zod";

const consumerNames = ["claude", "codex", "cursor"] as const;
const maxSkillNameLength = 64;
type SkillInstallationSet = Readonly<{
  installations: readonly Readonly<{ agent: ConsumerName }>[];
}>;
const skillNameSchema = z
  .string()
  .min(1)
  .max(maxSkillNameLength)
  .regex(/^[a-z0-9]+(?:-[a-z0-9]+)*$/u);
const installationSchema = z
  .object({
    agent: z.enum(consumerNames),
    scope: z.literal("user"),
  })
  .strict()
  .readonly();
const skillDeploymentSchema = z
  .object({
    slug: skillNameSchema,
    installations: z.array(installationSchema).min(1).readonly(),
  })
  .strict()
  .refine((skill: SkillInstallationSet): boolean => {
    const agents = skill.installations.map(
      (installation) => installation.agent,
    );
    return new Set(agents).size === agents.length;
  }, "Duplicate agent installation")
  .readonly();
const deploymentManifestSchema = z
  .looseObject({
    version: z.literal(1),
    skills: z.array(skillDeploymentSchema).readonly(),
  })
  .readonly();

type DeploymentInspection = Readonly<{
  deploymentManifestValid: boolean;
  installedFor: readonly ConsumerName[];
}>;
const isMissingPathError = (error: unknown): boolean =>
  error instanceof Error && Reflect.get(error, "code") === "ENOENT";

const isOutside = (root: string, path: string): boolean => {
  const fromRoot = relative(root, path);
  return (
    fromRoot === ".." || fromRoot.startsWith(`..${sep}`) || isAbsolute(fromRoot)
  );
};

const decode = (path: string): string =>
  new TextDecoder("utf-8", { fatal: true }).decode(readFileSync(path));

const parseYaml = (source: string): unknown => {
  try {
    return Bun.YAML.parse(source);
  } catch {
    return undefined;
  }
};

const readDeploymentInput = (root: string): unknown => {
  try {
    return parseYaml(decode(resolve(root, "home/.arnes.yaml")));
  } catch {
    return undefined;
  }
};

const readMakefileSnapshot = (root: string): MakefileSnapshot | undefined => {
  try {
    const bytes = readFileSync(resolve(root, "Makefile"));
    const source = new TextDecoder("utf-8", { fatal: true }).decode(bytes);
    return {
      lines: source.split("\n"),
      sha256: createHash("sha256").update(bytes).digest("hex"),
    };
  } catch {
    return undefined;
  }
};

const inspectDeployment = (
  root: string,
  slug: string,
): DeploymentInspection => {
  const parsed = deploymentManifestSchema.safeParse(readDeploymentInput(root));
  if (!parsed.success) {
    return { deploymentManifestValid: false, installedFor: [] };
  }
  const uniqueSlugs = new Set(parsed.data.skills.map((skill) => skill.slug));
  const matches = parsed.data.skills.filter((skill) => skill.slug === slug);
  const [match] = matches;
  if (
    uniqueSlugs.size !== parsed.data.skills.length ||
    matches.length !== 1 ||
    match === undefined
  ) {
    return { deploymentManifestValid: false, installedFor: [] };
  }
  const declaredFor = match.installations.map(
    (installation) => installation.agent,
  );
  const makefile = readMakefileSnapshot(root);
  if (makefile === undefined) {
    return { deploymentManifestValid: false, installedFor: [] };
  }
  const installedFor = inspectCanonicalMakefileDeployments(
    makefile,
    slug,
    declaredFor,
  );
  if (installedFor === undefined) {
    return { deploymentManifestValid: false, installedFor: [] };
  }
  return {
    deploymentManifestValid: true,
    installedFor,
  };
};

const gitTracked = (root: string, path: string): boolean => {
  const result = Bun.spawnSync([
    "git",
    "-C",
    root,
    "ls-files",
    "-s",
    "--",
    path,
  ]);
  if (result.exitCode !== 0) {
    return false;
  }
  const output = new TextDecoder("utf-8", { fatal: true }).decode(
    result.stdout,
  );
  const match =
    /^(?<mode>[0-9]{6}) [0-9a-f]{40,64} 0\t(?<path>[^\n]+)\n?$/u.exec(output);
  return (
    match?.groups?.path === path &&
    (match.groups.mode === "100644" || match.groups.mode === "100755")
  );
};

const unavailable = (
  kind: Extract<SkillTargetInspection["kind"], "missing" | "non-regular">,
): SkillTargetInspection => ({
  deploymentManifestValid: false,
  descriptionTriggerable: false,
  frontmatterValid: false,
  installedFor: [],
  kind,
  name: undefined,
  tracked: false,
});

const canonicalizeRoot = (root: string): string | undefined => {
  try {
    return realpathSync(root);
  } catch {
    return undefined;
  }
};

const inspectSkillTarget = (
  root: string,
  repositoryPath: string,
): SkillTargetInspection => {
  const canonicalRoot = canonicalizeRoot(root);
  if (canonicalRoot === undefined) {
    return unavailable("missing");
  }
  const path = resolve(canonicalRoot, repositoryPath);
  if (isOutside(canonicalRoot, path)) {
    return unavailable("missing");
  }
  try {
    const stats = lstatSync(path);
    if (!stats.isFile()) {
      return unavailable("non-regular");
    }
    const target = realpathSync(path);
    if (isOutside(canonicalRoot, target)) {
      return unavailable("non-regular");
    }
    const frontmatter = inspectSkillFrontmatter(decode(path));
    const deployment = inspectDeployment(
      canonicalRoot,
      repositoryPath.split("/")[2] ?? "",
    );
    return {
      ...deployment,
      ...frontmatter,
      kind: "regular-file",
      tracked: gitTracked(canonicalRoot, repositoryPath),
    };
  } catch (error) {
    if (isMissingPathError(error)) {
      return unavailable("missing");
    }
    return unavailable("non-regular");
  }
};

export { inspectSkillTarget };
