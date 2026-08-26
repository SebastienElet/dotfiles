import { fileURLToPath } from "node:url";
import { z } from "zod";

const exactNodeVersionSchema = z
  .string()
  .regex(/^(?:0|[1-9]\d*)\.(?:0|[1-9]\d*)\.(?:0|[1-9]\d*)$/u);
const packageJsonSchema = z.object({
  volta: z.object({
    node: exactNodeVersionSchema,
  }),
});
const maximumArgumentCount = 2;
const runtimeArgumentStart = 2;
const argumentsSchema = z.array(z.string()).min(1).max(maximumArgumentCount);
const commandSchema = z.enum(["install-spec", "verify-runtime"]);
const runtimeVersionSchema = z.string().regex(/^v\d+\.\d+\.\d+$/u);

const defaultPackageJsonPath = fileURLToPath(
  new URL("../package.json", import.meta.url),
);

async function readNodeVersion(packageJsonPath: string): Promise<string> {
  try {
    const packageJson: unknown = await Bun.file(packageJsonPath).json();
    return packageJsonSchema.parse(packageJson).volta.node;
  } catch (error) {
    const detail = error instanceof Error ? error.message : String(error);
    throw new Error(
      `Cannot read an exact Node pin from ${packageJsonPath}: ${detail}`,
      { cause: error },
    );
  }
}

function nodeInstallSpec(nodeVersion: string): string {
  return `node@${exactNodeVersionSchema.parse(nodeVersion)}`;
}

async function verifyNodeRuntime(
  nodeVersion: string,
  nodeExecutable = "node",
): Promise<void> {
  const node = Bun.spawn([nodeExecutable, "--version"], {
    stdout: "pipe",
    stderr: "pipe",
  });
  const [status, stdout, stderr] = await Promise.all([
    node.exited,
    new Response(node.stdout).text(),
    new Response(node.stderr).text(),
  ]);

  if (status !== 0) {
    throw new Error(
      `node --version failed with status ${status}: ${stderr.trim()}`,
    );
  }

  const runtimeVersion = runtimeVersionSchema.parse(stdout.trim()).slice(1);
  if (runtimeVersion !== nodeVersion) {
    throw new Error(
      `Node runtime ${runtimeVersion} does not match project pin ${nodeVersion}`,
    );
  }
}

async function runNodeVersionContract(
  args: readonly string[],
): Promise<string> {
  const [rawCommand, packageJsonPath = defaultPackageJsonPath] =
    argumentsSchema.parse(args);
  const command = commandSchema.parse(rawCommand);
  const nodeVersion = await readNodeVersion(packageJsonPath);

  if (command === "install-spec") {
    return nodeInstallSpec(nodeVersion);
  }

  await verifyNodeRuntime(nodeVersion);
  return `Node runtime matches project pin ${nodeVersion}`;
}

if (import.meta.main) {
  try {
    process.stdout.write(
      `${await runNodeVersionContract(Bun.argv.slice(runtimeArgumentStart))}\n`,
    );
  } catch (error) {
    process.stderr.write(
      `${error instanceof Error ? error.message : String(error)}\n`,
    );
    process.exitCode = 1;
  }
}

export {
  nodeInstallSpec,
  readNodeVersion,
  runNodeVersionContract,
  verifyNodeRuntime,
};
