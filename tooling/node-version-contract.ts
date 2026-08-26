import { fileURLToPath } from "node:url";

const exactNodeVersionPattern =
  /^(?:0|[1-9]\d*)\.(?:0|[1-9]\d*)\.(?:0|[1-9]\d*)$/u;
const maximumArgumentCount = 2;
const runtimeArgumentStart = 2;
const runtimeVersionPattern = /^v\d+\.\d+\.\d+$/u;

const defaultPackageJsonPath = fileURLToPath(
  new URL("../package.json", import.meta.url),
);

async function readNodeVersion(packageJsonPath: string): Promise<string> {
  try {
    const packageJson: unknown = await Bun.file(packageJsonPath).json();
    if (
      typeof packageJson !== "object" ||
      packageJson === null ||
      !("volta" in packageJson) ||
      typeof packageJson.volta !== "object" ||
      packageJson.volta === null ||
      !("node" in packageJson.volta)
    ) {
      throw new Error("missing volta.node");
    }

    return parseExactNodeVersion(packageJson.volta.node);
  } catch (error) {
    const detail = error instanceof Error ? error.message : String(error);
    throw new Error(
      `Cannot read an exact Node pin from ${packageJsonPath}: ${detail}`,
      { cause: error },
    );
  }
}

function parseExactNodeVersion(value: unknown): string {
  if (typeof value !== "string" || !exactNodeVersionPattern.test(value)) {
    throw new Error("Node version must be an exact major.minor.patch value");
  }

  return value;
}

function nodeInstallSpec(nodeVersion: string): string {
  return `node@${parseExactNodeVersion(nodeVersion)}`;
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

  const rawRuntimeVersion = stdout.trim();
  if (!runtimeVersionPattern.test(rawRuntimeVersion)) {
    throw new Error(
      `node --version returned an invalid value: ${rawRuntimeVersion}`,
    );
  }

  const runtimeVersion = rawRuntimeVersion.slice(1);
  if (runtimeVersion !== nodeVersion) {
    throw new Error(
      `Node runtime ${runtimeVersion} does not match project pin ${nodeVersion}`,
    );
  }
}

async function runNodeVersionContract(
  args: readonly string[],
): Promise<string> {
  if (args.length === 0 || args.length > maximumArgumentCount) {
    throw new Error("Expected a command and an optional package.json path");
  }

  const [command, packageJsonPath = defaultPackageJsonPath] = args;
  if (command !== "install-spec" && command !== "verify-runtime") {
    throw new Error(`Unknown command: ${command}`);
  }

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
