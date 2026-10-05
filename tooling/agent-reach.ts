import {
  fetchAgentReachSource,
  runAgentReachCommand,
} from "./agent-reach-source.ts";
import { readFile, realpath } from "node:fs/promises";
import { copyAgentReachFile } from "./agent-reach-config.ts";
import { join } from "node:path";
import { prepareAgentReachPlugin } from "./agent-reach-package.ts";
import { z } from "zod";

const sourceSchema = z
  .strictObject({
    version: z.string().regex(/^\d+\.\d+\.\d+$/u),
    revision: z.string().regex(/^[a-f0-9]{40}$/u),
    python: z.array(z.string().min(1)).min(1).readonly(),
    node: z.array(z.string().min(1)).min(1).readonly(),
  })
  .readonly();
type Source = z.infer<typeof sourceSchema>;
const invocationSchema = z.tuple([
  z.enum([
    "plan",
    "runtime",
    "python",
    "node",
    "package",
    "configuration",
    "claude",
    "codex",
  ]),
  z.string().min(1),
  z.string().min(1),
]);
const argumentOffset = 2;
const indentation = 2;

function installationEnvironment(
  home: string,
): Readonly<Record<string, string>> {
  return {
    HOME: home,
    VOLTA_HOME: join(home, ".volta"),
    PATH: `${join(home, ".local/bin")}:${join(home, ".volta/bin")}:/opt/homebrew/bin:/usr/local/bin:${process.env.PATH ?? ""}`,
  };
}

async function packagePlugin(
  configuration: Source,
  home: string,
): Promise<void> {
  const source = await fetchAgentReachSource({
    home,
    repository: "https://github.com/Panniantong/Agent-Reach.git",
    revision: configuration.revision,
  });
  const marketplace = await prepareAgentReachPlugin({
    source,
    home,
    version: configuration.version,
    revision: configuration.revision,
  });
  await copyAgentReachFile({
    home,
    source: join(
      marketplace,
      "plugins/agent-reach/scripts/transcribe-xiaoyuzhou.sh",
    ),
    destination: join(home, ".agent-reach/tools/xiaoyuzhou/transcribe.sh"),
    executable: true,
  });
  process.stdout.write(`${marketplace}\n`);
}

async function installPython(
  specifications: readonly string[],
  home: string,
): Promise<void> {
  const environment = {
    ...installationEnvironment(home),
    UV_TOOL_DIR: join(home, ".local/share/agent-reach/python-tools"),
    UV_TOOL_BIN_DIR: join(home, ".local/bin"),
  };
  for (const specification of specifications) {
    process.stdout.write(
      await runAgentReachCommand(
        ["uv", "tool", "install", "--python", "3.13", specification],
        environment,
      ),
    );
  }
}

async function installPlugin(
  host: "claude" | "codex",
  home: string,
): Promise<void> {
  const executable =
    host === "claude"
      ? join(home, ".local/bin/claude")
      : join(home, ".volta/bin/codex");
  const environment = {
    ...installationEnvironment(home),
    CLAUDE_CONFIG_DIR: join(home, ".claude"),
  };
  const marketplace = join(home, ".local/share/agent-reach/marketplace");
  process.stdout.write(
    await runAgentReachCommand(
      [executable, "plugin", "marketplace", "add", marketplace],
      environment,
    ),
  );
  const installation =
    host === "claude"
      ? [
          executable,
          "plugin",
          "install",
          "agent-reach@dotfiles-agent-reach",
          "--scope",
          "user",
        ]
      : [
          executable,
          "plugin",
          "add",
          "agent-reach@dotfiles-agent-reach",
          "--json",
        ];
  process.stdout.write(await runAgentReachCommand(installation, environment));
}

async function installTools(
  operation: "runtime" | "python" | "node",
  configuration: Source,
  home: string,
): Promise<void> {
  if (operation === "runtime") {
    await installPython(
      [
        `agent-reach @ git+https://github.com/Panniantong/Agent-Reach.git@${configuration.revision}`,
      ],
      home,
    );
    return;
  }
  if (operation === "python") {
    await installPython(configuration.python, home);
    return;
  }
  for (const specification of configuration.node) {
    process.stdout.write(
      await runAgentReachCommand(
        ["volta", "install", specification],
        installationEnvironment(home),
      ),
    );
  }
}

async function main(): Promise<void> {
  const [operation, repository, homeArgument] = invocationSchema.parse(
    process.argv.slice(argumentOffset),
  );
  const home = await realpath(homeArgument);
  const configuration = sourceSchema.parse(
    JSON.parse(
      await readFile(
        join(repository, "harness/plugins/agent-reach/source.json"),
        "utf8",
      ),
    ),
  );
  if (operation === "plan") {
    process.stdout.write(
      `${JSON.stringify(configuration, null, indentation)}\n`,
    );
    return;
  }
  if (operation === "package") {
    await packagePlugin(configuration, home);
    return;
  }
  if (process.platform !== "darwin") {
    throw new Error("Agent-Reach installation targets macOS");
  }
  if (operation === "configuration") {
    await copyAgentReachFile({
      home,
      source: join(repository, "home/.config/agent-reach/mcporter.json"),
      destination: join(home, ".config/agent-reach/mcporter.json"),
    });
    return;
  }
  if (operation === "claude" || operation === "codex") {
    await installPlugin(operation, home);
    return;
  }
  await installTools(operation, configuration, home);
}

if (import.meta.main) {
  try {
    await main();
  } catch (error) {
    process.stderr.write(
      `Agent-Reach: ${error instanceof Error ? error.message : String(error)}\n`,
    );
    process.exitCode = 1;
  }
}
