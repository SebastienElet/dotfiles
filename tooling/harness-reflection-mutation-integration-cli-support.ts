import { copyFile, symlink } from "node:fs/promises";
import { join, resolve } from "node:path";

const sourceRoot = resolve(import.meta.dir, "..");
const cliFiles = [
  "harness-reflection-mutation-surfaces.ts",
  "invariant-registry-ablation-policy.ts",
  "invariant-registry-cli.ts",
  "invariant-registry-consumers.ts",
  "invariant-registry-contract.ts",
  "invariant-registry-oracle-inspection.ts",
  "invariant-registry-oracle-policy.ts",
  "invariant-registry-policy.ts",
  "invariant-registry-repository-validator.ts",
  "invariant-registry-runtime-oracles.ts",
  "invariant-registry-schema.ts",
  "invariant-registry-skill-target-deployment-manifest.ts",
  "invariant-registry-skill-target-frontmatter.ts",
  "invariant-registry-skill-target-inspection.ts",
  "invariant-registry-skill-target-policy.ts",
  "invariant-registry-source.ts",
  "invariant-registry-validation-options.ts",
  "invariant-registry-workflow-state-oracle.ts",
] as const;

type CliOutcome = Readonly<{
  exitCode: number;
  stderr: string;
  stdout: string;
}>;

const installFixtureCli = async (root: string): Promise<void> => {
  await Promise.all(
    cliFiles.map((file) =>
      copyFile(join(sourceRoot, "tooling", file), join(root, "tooling", file)),
    ),
  );
  await symlink(
    join(sourceRoot, "node_modules"),
    join(root, "node_modules"),
    "dir",
  );
};

const runFixtureCli = async (root: string): Promise<CliOutcome> => {
  const child = Bun.spawn(
    [process.execPath, "tooling/invariant-registry-cli.ts"],
    { cwd: root, stderr: "pipe", stdout: "pipe" },
  );
  const [exitCode, stdout, stderr] = await Promise.all([
    child.exited,
    new Response(child.stdout).text(),
    new Response(child.stderr).text(),
  ]);
  return { exitCode, stderr, stdout };
};

export { installFixtureCli, runFixtureCli };
