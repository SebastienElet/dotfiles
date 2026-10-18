import { dirname, join, resolve } from "node:path";
import {
  ensureDirectories,
  metadata,
  preserveExisting,
  publishMarketplace,
  regularText,
} from "./agent-reach-files.ts";
import { readdir, realpath } from "node:fs/promises";
import packageMetadata from "../package.json";
import { z } from "zod";

const optionsSchema = z
  .object({
    source: z.string().min(1),
    home: z.string().min(1),
    version: z.string().regex(/^\d+\.\d+\.\d+$/u),
    revision: z.string().regex(/^[a-f0-9]{40}$/u),
  })
  .readonly();
type PluginOptions = z.infer<typeof optionsSchema>;
const marketplaceName = "agent-reach-adapter";
const adapterDirectory = `.local/share/agent-reach/adapter/${packageMetadata.version}`;
const pluginDirectory = "plugins/agent-reach";
const skillDirectory = `${pluginDirectory}/skills/agent-reach`;
const description =
  "Read supported platforms using Agent-Reach and upstream CLI tools.";
const indentation = 2;
const pythonToolsPath =
  "$HOME/.local/share/agent-reach/python-tools/agent-reach/bin";
const pythonCommand = `bash "$HOME/${adapterDirectory}/marketplace/plugins/agent-reach/scripts/agent-reach-python"`;

function json(value: unknown): string {
  return `${JSON.stringify(value, null, indentation)}\n`;
}

function adaptedCommands(content: string): string {
  return content
    .replaceAll(
      'curl -s "https://r.jina.ai/',
      'curl --silent --show-error --include --connect-timeout 10 --max-time 30 -H "X-No-Cache: true" -H "DNT: 1" "https://r.jina.ai/',
    )
    .replaceAll(
      /\bagent-reach doctor\b/gu,
      'MCPORTER_CONFIG="$HOME/.config/agent-reach/mcporter.json" agent-reach doctor',
    )
    .replaceAll(
      /\bmcporter(?=\s+(?:call|list|config|auth)\b)/gu,
      'mcporter --config "$HOME/.config/agent-reach/mcporter.json"',
    )
    .replaceAll(/\byt-dlp(?=\s+(?:--|URL))/gu, "yt-dlp --js-runtimes node")
    .replaceAll(
      "~/.agent-reach/tools/xiaoyuzhou/transcribe.sh",
      `bash "$HOME/${adapterDirectory}/marketplace/plugins/agent-reach/scripts/transcribe-xiaoyuzhou.sh"`,
    )
    .replaceAll(
      "uvx mcp-server-linkedin@latest --login",
      "user-controlled LinkedIn login with the configured Docker service",
    )
    .replaceAll(
      /\b(?:(?:agent-reach|pipx|pip3?|brew|npm|pnpm|yarn|bun|uv(?:\s+(?:pip|tool))?)\s+(?:-[^\s`]+\s+)*(?:install|i|reinstall(?:-all)?|add|upgrade(?:-all)?|update|sync|ci|fetch|exec|dlx|x)\b|(?:uv|pipx)\s+(?:-[^\s`]+\s+)*run\b|uvx\b|npx\b|bunx\b)[^\n`]*/gu,
      "install the declared prerequisites explicitly; see the adapter README",
    );
}

function adaptedSkill(content: string, template: string): string {
  const routing = content.indexOf("## Routing table");
  const end = content.indexOf("## Environment check", routing);
  const fallback = content.indexOf("## Configure a channel", routing);
  if (routing === -1 || fallback === -1) {
    throw new Error("Upstream skill routing sections changed");
  }
  return adaptedCommands(
    template.replace(
      "{{UPSTREAM_ROUTING}}",
      content.slice(routing, end === -1 ? fallback : end),
    ),
  );
}

function adaptedWebReference(original: string): string {
  const withoutWebReader = original.replace(
    /^## Web Reader \(MCP\)\n[\s\S]*?(?=^## RSS \(feedparser\)\n)/mu,
    "",
  );
  const content = withoutWebReader.replaceAll(/^.*web-reader MCP.*\n/gmu, "");
  if (
    withoutWebReader === original ||
    /web-reader|Web Reader \(MCP\)/u.test(content)
  ) {
    throw new Error("Upstream web reference sections changed");
  }
  return content.replaceAll(/\bpython3(?=\s)/gu, pythonCommand);
}

async function bundle(
  options: PluginOptions,
): Promise<ReadonlyMap<string, string>> {
  const source = await realpath(options.source);
  const files = new Map<string, string>();
  const manifest = {
    name: "agent-reach",
    version: options.version,
    description,
    license: "BSD-2-Clause AND MIT",
    skills: "./skills/",
  };
  files.set(
    `${skillDirectory}/SKILL.md`,
    adaptedSkill(
      await regularText(source, "agent_reach/skill/SKILL_en.md"),
      await regularText(resolve(import.meta.dir, "../resources"), "SKILL.md"),
    ),
  );
  const entries = await readdir(join(source, "agent_reach/skill/references"));
  const references = entries.filter((name) => name.endsWith(".md")).toSorted();
  if (references.length === 0) {
    throw new Error("No upstream platform references");
  }
  for (const name of references) {
    const original = await regularText(
      source,
      `agent_reach/skill/references/${name}`,
    );
    const content =
      name === "web.md" ? adaptedWebReference(original) : original;
    files.set(`${skillDirectory}/references/${name}`, adaptedCommands(content));
  }
  files.set(`${pluginDirectory}/LICENSE`, await regularText(source, "LICENSE"));
  files.set(
    `${pluginDirectory}/ADAPTER-LICENSE`,
    await regularText(resolve(import.meta.dir, ".."), "LICENSE"),
  );
  for (const [path, content] of await scriptResources(source)) {
    files.set(path, content);
  }
  return addMetadata([...files], options, manifest);
}

async function scriptResources(
  source: string,
): Promise<readonly (readonly [string, string])[]> {
  const podcast = await regularText(
    source,
    "agent_reach/scripts/transcribe_xiaoyuzhou.sh",
  );
  const python = await regularText(
    resolve(import.meta.dir, "../resources"),
    "agent-reach-python",
  );
  return [
    [
      `${pluginDirectory}/scripts/transcribe-xiaoyuzhou.sh`,
      podcast.replace(
        /^#![^\n]*\n/u,
        `$&export PATH="${pythonToolsPath}:$PATH"\n`,
      ),
    ],
    [`${pluginDirectory}/scripts/agent-reach-python`, python],
  ];
}

function addMetadata(
  input: readonly (readonly [string, string])[],
  options: PluginOptions,
  manifest: Readonly<Record<string, string>>,
): ReadonlyMap<string, string> {
  const claude = json({
    name: marketplaceName,
    owner: { name: "Sebastien Elet" },
    plugins: [
      { name: "agent-reach", source: `./${pluginDirectory}`, description },
    ],
  });
  const codex = json({
    name: marketplaceName,
    plugins: [
      {
        name: "agent-reach",
        source: { source: "local", path: `./${pluginDirectory}` },
        policy: { installation: "AVAILABLE", authentication: "ON_INSTALL" },
        category: "Productivity",
      },
    ],
  });
  return new Map<string, string>([
    ...input,
    [`${pluginDirectory}/.claude-plugin/plugin.json`, json(manifest)],
    [`${pluginDirectory}/.codex-plugin/plugin.json`, json(manifest)],
    [".claude-plugin/marketplace.json", claude],
    [".agents/plugins/marketplace.json", codex],
    [
      "source.json",
      json({ version: options.version, revision: options.revision }),
    ],
  ]);
}

async function prepareAgentReachPlugin(input: PluginOptions): Promise<string> {
  const options = optionsSchema.parse(input);
  const files = [...(await bundle(options))];
  const home = await realpath(options.home);
  const directory = join(home, adapterDirectory);
  const destination = join(directory, "marketplace");
  await ensureDirectories(home, directory);
  const existing = await metadata(destination);
  if (existing !== null) {
    const target = await realpath(destination);
    if (
      existing.isSymbolicLink() &&
      (!target.startsWith(join(directory, ".marketplace-")) ||
        dirname(target) !== directory)
    ) {
      throw new Error(`divergent marketplace: ${destination}`);
    }
    if (!existing.isDirectory() && !existing.isSymbolicLink()) {
      throw new Error(`divergent marketplace: ${destination}`);
    }
    await preserveExisting(destination, files);
    return destination;
  }
  await publishMarketplace(directory, destination, files);
  return destination;
}

export { prepareAgentReachPlugin };
