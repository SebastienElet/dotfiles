import { basename, join } from "node:path";
import {
  lstatSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  realpathSync,
  renameSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { isDeepStrictEqual } from "node:util";
import { z } from "zod";

const endpoint = "https://mcp.context7.com/mcp";
const configurationSchema = z.looseObject({
  mcp_servers: z.record(z.string(), z.unknown()).optional(),
});
const serverSchema = z.looseObject({
  command: z.string().optional(),
  args: z.array(z.string()).optional(),
  url: z.string().optional(),
  env: z.record(z.string(), z.string()).optional(),
  env_vars: z.array(z.string()).optional(),
});
const localArgumentsSchema = z.union([
  z.tuple([]),
  z.tuple([z.literal("--api-key"), z.string().min(1)]),
]);
const launchFields = new Set(["command", "args", "env", "env_vars", "cwd"]);
const argumentOffset = 2;
type Configuration = z.infer<typeof configurationSchema>;
type ConfigurationFile = Readonly<{
  path: string;
  original: string;
  identity: readonly [number, number] | undefined;
}>;

function parseConfiguration(original: string): Configuration {
  try {
    return configurationSchema.parse(Bun.TOML.parse(original));
  } catch {
    throw new Error("Codex configuration cannot be parsed");
  }
}

function parseLocalArguments(
  input: readonly string[],
): z.infer<typeof localArgumentsSchema> {
  const args = [...input];
  if (args[0] === "-y" || args[0] === "--yes") {
    args.shift();
  }
  const packageName = args.shift();
  if (!/^@upstash\/context7-mcp(?:@\d+\.\d+\.\d+)?$/u.test(packageName ?? "")) {
    throw new Error("Unrecognized Context7 package");
  }
  return localArgumentsSchema.parse(args);
}

function remoteServer(value: unknown): Record<string, unknown> {
  const server = serverSchema.parse(value);
  if (server.url !== undefined) {
    if (server.url !== endpoint || server.command !== undefined) {
      throw new Error("Divergent Context7 transport");
    }
    return server;
  }
  if (server.command === undefined || basename(server.command) !== "npx") {
    throw new Error("Unrecognized Context7 launcher");
  }
  const options = parseLocalArguments(server.args ?? []);
  if (
    Object.keys(server.env ?? {}).some((name) => name !== "CONTEXT7_API_KEY") ||
    (server.env_vars ?? []).some((name) => name !== "CONTEXT7_API_KEY")
  ) {
    throw new Error("Unrecognized Context7 environment");
  }
  const settings = Object.fromEntries(
    Object.entries(server).filter(
      ([name]: readonly [string, unknown]) => !launchFields.has(name),
    ),
  );
  const apiKey = options[1] ?? server.env?.CONTEXT7_API_KEY;
  if (apiKey !== undefined) {
    return {
      ...settings,
      url: endpoint,
      http_headers: { Authorization: `Bearer ${apiKey}` },
    };
  }
  if (server.env_vars?.includes("CONTEXT7_API_KEY") === true) {
    return {
      ...settings,
      url: endpoint,
      bearer_token_env_var: "CONTEXT7_API_KEY",
    };
  }
  return { ...settings, url: endpoint };
}

function migrateContext7Configuration(original: string): string {
  const document = parseConfiguration(original);
  const servers = document.mcp_servers ?? {};
  const context7 =
    servers.context7 === undefined
      ? { url: endpoint }
      : remoteServer(servers.context7);
  if (isDeepStrictEqual(servers.context7, context7)) {
    return original;
  }
  const updated = {
    ...document,
    mcp_servers: { ...servers, context7 },
  };
  const rendered = Bun.TOML.stringify(updated);
  if (
    rendered === undefined ||
    !isDeepStrictEqual(parseConfiguration(rendered), updated)
  ) {
    throw new Error("Codex configuration cannot be preserved");
  }
  return rendered;
}

function readConfiguration(directory: string): ConfigurationFile {
  const parent = lstatSync(directory, { throwIfNoEntry: false });
  if (parent !== undefined && !parent.isDirectory()) {
    throw new Error("Codex directory must be a real directory");
  }
  const path = join(directory, "config.toml");
  const metadata = lstatSync(path, { throwIfNoEntry: false });
  if (metadata !== undefined && (!metadata.isFile() || metadata.nlink !== 1)) {
    throw new Error("Codex configuration must be a regular file with one link");
  }
  const original = metadata === undefined ? "" : readFileSync(path, "utf8");
  return {
    path,
    original,
    identity: metadata === undefined ? undefined : [metadata.dev, metadata.ino],
  };
}

function installContext7(home: string): void {
  const directory = join(realpathSync(home), ".codex");
  const { path, original, identity } = readConfiguration(directory);
  const updated = migrateContext7Configuration(original);
  if (updated === original) {
    return;
  }
  mkdirSync(directory, { recursive: true, mode: 0o700 });
  const staging = mkdtempSync(join(directory, ".context7-"));
  try {
    const temporary = join(staging, "config.toml");
    writeFileSync(temporary, updated, { mode: 0o600 });
    const current = lstatSync(path, { throwIfNoEntry: false });
    if (
      current?.dev !== identity?.[0] ||
      current?.ino !== identity?.[1] ||
      (current !== undefined && (!current.isFile() || current.nlink !== 1)) ||
      (current !== undefined && readFileSync(path, "utf8") !== original)
    ) {
      throw new Error("Codex configuration changed during deployment");
    }
    renameSync(temporary, path);
  } finally {
    rmSync(staging, { recursive: true, force: true });
  }
}

if (import.meta.main) {
  try {
    const [home] = z
      .tuple([z.string().min(1)])
      .parse(process.argv.slice(argumentOffset));
    installContext7(home);
  } catch {
    process.stderr.write("Context7 deployment failed.\n");
    process.exitCode = 1;
  }
}

export { installContext7, migrateContext7Configuration };
