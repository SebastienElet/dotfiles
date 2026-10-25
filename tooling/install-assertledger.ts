import { dirname, join } from "node:path";
import {
  lstatSync,
  mkdirSync,
  readFileSync,
  renameSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { readNodeVersion, verifyNodeRuntime } from "./node-version-contract.ts";
import { randomUUID } from "node:crypto";
import { z } from "zod";

const argumentsSchema = z.tuple([z.string().min(1), z.string().min(1)]);
const versionSchema = z
  .string()
  .regex(/^ASSERTLEDGER_VERSION=\d+\.\d+\.\d+\s*$/u);
const packageSchema = z.object({
  version: z.string(),
  engines: z.object({ node: z.string() }),
});
const ownerReadAndExecute = 0o500;
const argumentOffset = 2;

function shellQuote(value: string): string {
  return `'${value.replaceAll("'", String.raw`'\''`)}'`;
}

function launcherContent(home: string, version: string): string {
  const node = join(home, ".volta/tools/image/node", version, "bin/node");
  const cli = join(
    home,
    ".bun/install/global/node_modules/assertledger/dist/cli.js",
  );
  return `#!/usr/bin/env bash\nexec ${shellQuote(node)} ${shellQuote(cli)} "$@"\n`;
}

function verifyInstalledCli(
  repository: string,
  home: string,
  version: string,
): void {
  const node = join(home, ".volta/tools/image/node", version, "bin/node");
  const declaredVersion = versionSchema
    .parse(
      readFileSync(
        join(
          repository,
          "node_modules/@sebastienelet/agent-skills/skills/assertledger/version.env",
        ),
        "utf8",
      ),
    )
    .trim()
    .slice("ASSERTLEDGER_VERSION=".length);
  const cli = join(
    home,
    ".bun/install/global/node_modules/assertledger/dist/cli.js",
  );
  const packagePath = join(dirname(dirname(cli)), "package.json");
  const installed = packageSchema.parse(
    JSON.parse(readFileSync(packagePath, "utf8")),
  );
  if (
    installed.version !== declaredVersion ||
    !Bun.semver.satisfies(version, installed.engines.node)
  ) {
    throw new Error(
      `AssertLedger ${declaredVersion} requires Node ${installed.engines.node}; found ${installed.version} with Node ${version}`,
    );
  }
  const probe = Bun.spawnSync([node, cli, "--version"]);
  if (
    probe.exitCode !== 0 ||
    probe.stdout.toString().trim() !== `assertledger ${declaredVersion}`
  ) {
    throw new Error(
      `AssertLedger ${declaredVersion} is required at ${cli}: ${probe.stderr.toString()}`,
    );
  }
}

function publishLauncher(home: string, version: string): void {
  const destination = join(home, ".local/bin/assertledger");
  const content = launcherContent(home, version);
  const metadata = lstatSync(destination, { throwIfNoEntry: false });
  if (metadata !== undefined) {
    if (!metadata.isFile()) {
      throw new Error(`${destination} is not a managed launcher`);
    }
    const previous = readFileSync(destination, "utf8");
    const previousVersion =
      /\/node\/(?<version>\d+\.\d+\.\d+)\/bin\/node/u.exec(previous)?.groups
        ?.version;
    if (
      previousVersion === undefined ||
      previous !== launcherContent(home, previousVersion)
    ) {
      throw new Error(`${destination} is not a managed launcher`);
    }
    if (
      previous === content &&
      (metadata.mode & ownerReadAndExecute) === ownerReadAndExecute
    ) {
      return;
    }
  }
  mkdirSync(dirname(destination), { recursive: true });
  const temporary = `${destination}.${randomUUID()}`;
  try {
    writeFileSync(temporary, content, { mode: 0o755, flag: "wx" });
    renameSync(temporary, destination);
  } finally {
    rmSync(temporary, { force: true });
  }
}

async function installAssertledger(
  repository: string,
  home: string,
): Promise<void> {
  const version = await readNodeVersion(join(repository, "package.json"));
  const node = join(home, ".volta/tools/image/node", version, "bin/node");
  await verifyNodeRuntime(version, node);
  verifyInstalledCli(repository, home, version);
  publishLauncher(home, version);
}

if (import.meta.main) {
  try {
    const [repository, home] = argumentsSchema.parse(
      process.argv.slice(argumentOffset),
    );
    await installAssertledger(repository, home);
  } catch (error) {
    process.stderr.write(
      `${error instanceof Error ? error.message : String(error)}\n`,
    );
    process.exitCode = 1;
  }
}

export { installAssertledger };
