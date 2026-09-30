import {
  type Agent,
  type HookRoots,
  withoutOwnedHooks,
} from "./clean-deployment-hooks.ts";
import {
  type McpRegistration,
  mcpRegistrations,
} from "./clean-deployment-inventory.ts";
import {
  closeSync,
  lstatSync,
  openSync,
  readFileSync,
  renameSync,
  unlinkSync,
  writeFileSync,
} from "node:fs";
import { isDeepStrictEqual } from "node:util";
import { join } from "node:path";
import { parsePreservingJson } from "./clean-deployment-json.ts";
import { verifyParents } from "./clean-deployment-paths.ts";
import { z } from "zod";

const objectSchema = z.record(z.string(), z.unknown());
const serverSchema = z
  .looseObject({
    command: z.string().optional(),
    args: z.array(z.string()).readonly().optional(),
  })
  .readonly();
type ConfigurationUpdate = Readonly<{
  path: string;
  original: string;
  updated: string;
  mode: number;
}>;
const jsonIndentation = 2;

function ownedServer(value: unknown, registration: McpRegistration): boolean {
  const server = serverSchema.parse(value);
  return (
    server.command === registration.command &&
    isDeepStrictEqual(server.args ?? [], registration.args)
  );
}

function withoutServers(
  value: unknown,
  registrations: readonly McpRegistration[],
): Record<string, unknown> {
  const servers = objectSchema.parse(value);
  return Object.fromEntries(
    Object.entries(servers).filter(
      ([name, server]: readonly [string, unknown]) =>
        !registrations.some(
          (registration) =>
            registration.name === name && ownedServer(server, registration),
        ),
    ),
  );
}

function readConfigurationText(path: string): string {
  const bytes = readFileSync(path);
  try {
    return new TextDecoder("utf-8", { fatal: true }).decode(bytes);
  } catch {
    throw new Error(`Shared configuration is not valid UTF-8: ${path}`);
  }
}

function readConfiguration(
  repository: string,
  home: string,
  relativePath: string,
): Readonly<{ path: string; original: string; mode: number }> | undefined {
  const path = join(home, relativePath);
  verifyParents(home, repository, path);
  const metadata = lstatSync(path, { throwIfNoEntry: false });
  if (metadata === undefined) {
    return undefined;
  }
  if (!metadata.isFile()) {
    throw new Error(`Shared configuration must be a regular file: ${path}`);
  }
  return { path, original: readConfigurationText(path), mode: metadata.mode };
}

function jsonUpdate(
  roots: HookRoots,
  selection: Readonly<{
    relativePath: string;
    agent: Agent | undefined;
    registrations: readonly McpRegistration[];
  }>,
): ConfigurationUpdate | undefined {
  const { repository, home } = roots;
  const { relativePath, agent, registrations } = selection;
  const file = readConfiguration(repository, home, relativePath);
  if (file === undefined) {
    return undefined;
  }
  const document = objectSchema.parse(parsePreservingJson(file.original));
  const updated = { ...document };
  if (agent !== undefined && document.hooks !== undefined) {
    updated.hooks = withoutOwnedHooks(document.hooks, agent, roots);
  }
  if (document.mcpServers !== undefined) {
    updated.mcpServers = withoutServers(document.mcpServers, registrations);
  }
  if (isDeepStrictEqual(updated, document)) {
    return undefined;
  }
  return {
    ...file,
    updated: `${JSON.stringify(updated, null, jsonIndentation)}\n`,
  };
}

function withoutMcpTables(
  original: string,
  removed: readonly string[],
): string {
  let excluded = false;
  return original
    .split(/(?<=\n)/u)
    .filter((line) => {
      const heading = /^\s*\[\[?(?<namespace>[^\]]+)\]\]?\s*(?:#.*)?$/u.exec(
        line.trimEnd(),
      );
      if (heading !== null) {
        excluded = removed.some(
          (name) =>
            heading.groups?.namespace === `mcp_servers.${name}` ||
            heading.groups?.namespace?.startsWith(`mcp_servers.${name}.`) ===
              true,
        );
      }
      return !excluded;
    })
    .join("");
}

function tomlUpdate(
  repository: string,
  home: string,
  registrations: readonly McpRegistration[],
): ConfigurationUpdate | undefined {
  const file = readConfiguration(repository, home, ".codex/config.toml");
  if (file === undefined) {
    return undefined;
  }
  const document = objectSchema.parse(Bun.TOML.parse(file.original));
  if (document.mcp_servers === undefined) {
    return undefined;
  }
  const servers = objectSchema.parse(document.mcp_servers);
  const retained = withoutServers(servers, registrations);
  const removed = Object.keys(servers).filter((name) => !(name in retained));
  if (removed.length === 0) {
    return undefined;
  }
  const updated = withoutMcpTables(file.original, removed);
  const expected: Record<string, unknown> = {
    ...document,
    mcp_servers: retained,
  };
  if (Object.keys(retained).length === 0) {
    delete expected.mcp_servers;
  }
  if (
    !isDeepStrictEqual(objectSchema.parse(Bun.TOML.parse(updated)), expected)
  ) {
    throw new Error(`Cannot safely remove owned TOML namespaces: ${file.path}`);
  }
  return { ...file, updated };
}

function configurationUpdates(
  repository: string,
  home: string,
  aliases: HookRoots["aliases"] = [],
): readonly ConfigurationUpdate[] {
  const registrations = mcpRegistrations(repository);
  const roots = { repository, home, aliases };
  return [
    jsonUpdate(roots, {
      relativePath: ".claude/settings.json",
      agent: "claude",
      registrations: [],
    }),
    jsonUpdate(roots, {
      relativePath: ".codex/hooks.json",
      agent: "codex",
      registrations: [],
    }),
    jsonUpdate(roots, {
      relativePath: ".cursor/hooks.json",
      agent: "cursor",
      registrations: [],
    }),
    jsonUpdate(roots, {
      relativePath: ".claude.json",
      agent: undefined,
      registrations: registrations.filter((entry) => entry.agent === "claude"),
    }),
    tomlUpdate(
      repository,
      home,
      registrations.filter((entry) => entry.agent === "codex"),
    ),
  ].filter((update) => update !== undefined);
}

function applyConfiguration(
  update: ConfigurationUpdate,
  repository: string,
  home: string,
): void {
  verifyParents(home, repository, update.path);
  const metadata = lstatSync(update.path);
  if (
    !metadata.isFile() ||
    readConfigurationText(update.path) !== update.original
  ) {
    throw new Error(
      `Shared configuration changed during cleanup: ${update.path}`,
    );
  }
  const temporary = `${update.path}.${process.pid}.clean.tmp`;
  let created = false;
  try {
    const descriptor = openSync(temporary, "wx", update.mode);
    created = true;
    try {
      writeFileSync(descriptor, update.updated);
    } finally {
      closeSync(descriptor);
    }
    renameSync(temporary, update.path);
  } finally {
    if (
      created &&
      lstatSync(temporary, { throwIfNoEntry: false })?.isFile() === true
    ) {
      unlinkSync(temporary);
    }
  }
}

export { applyConfiguration, configurationUpdates };
