import { dirname, join, resolve } from "node:path";
import {
  ensureDirectories,
  metadata,
  preserveExisting,
  publishMarketplace,
  regularText,
} from "./agent-reach-files.ts";
import { readdir, realpath } from "node:fs/promises";
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
const marketplaceName = "dotfiles-agent-reach";
const pluginDirectory = "plugins/agent-reach";
const skillDirectory = `${pluginDirectory}/skills/agent-reach`;
const description =
  "Read supported platforms using Agent-Reach and upstream CLI tools.";
const indentation = 2;
const pythonToolsPath =
  "$HOME/.local/share/agent-reach/python-tools/agent-reach/bin";
const pythonCommand =
  'bash "$HOME/.local/share/agent-reach/marketplace/plugins/agent-reach/scripts/agent-reach-python"';

function json(value: unknown): string {
  return `${JSON.stringify(value, null, indentation)}\n`;
}

function adaptedCommands(content: string): string {
  return content
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
      'bash "$HOME/.local/share/agent-reach/marketplace/plugins/agent-reach/scripts/transcribe-xiaoyuzhou.sh"',
    )
    .replaceAll(
      "uvx mcp-server-linkedin@latest --login",
      "moon run harness:agent-reach-linkedin-login",
    )
    .replaceAll(
      /agent-reach install[^\n`]*|pipx install[^\n`]*|brew install ffmpeg/gu,
      "moon run harness:agent-reach",
    );
}

const skillHeader = `---
name: agent-reach
description: >
  Read YouTube transcripts and supported social platforms through Agent-Reach.
  Use for Twitter/X, Reddit, Facebook, Instagram, LinkedIn, Bilibili, XiaoHongShu,
  Xiaoyuzhou, Boss Zhipin, Xueqiu, V2EX and RSS, or explicit Agent-Reach requests.
  Prefer existing web and GitHub tools for ordinary URLs and code searches.
license: MIT
metadata:
  category: ops
---

# Agent Reach

## Overview

Use the packaged upstream platform recipes. Existing web fetching and the
fetch → Scrapling → CloakBrowser escalation retain their priority.

## Usage

Select the reference for the requested platform. Publishing, commenting, liking
and account changes require a separate explicit user instruction.

## Workflow

1. Read the matching reference below and verify actual content with the requested
   platform's read command. Do not run the global \`agent-reach doctor\` unless
   the user explicitly authorizes inspection of all configured browser sessions;
   its Boss backend can read Chrome cookies through CDP.
2. Install dependencies only through \`moon run harness:agent-reach\` in the
   dotfiles checkout, explicitly requested by the user. These instructions
   supersede setup commands in upstream references. Never install upstream skills,
   fetch unpinned setup guides, or check for updates during research.
3. Credentials are entered by the user directly into hidden tool prompts.
   Never request credentials in chat or automatically read browser cookies.
4. LinkedIn uses the named Docker service. Its explicit startup task is
   \`harness:agent-reach-linkedin-start\`; its interactive login task is
   \`harness:agent-reach-linkedin-login\`. Do not automate account login.

`;
const skillFooter = `
## References

Read the packaged platform files under \`references/\`: search, social, career,
dev, web, video and finance. All upstream channels are retained.

## Gotchas

- Doctor availability does not establish a successful search or transcript.
- OpenCLI requires its Chrome extension and an existing user-controlled session.
- LinkedIn's container must be stopped before its interactive login.

## Constraints

- Never install tools or update this plugin as a side effect of research.
- Never export cookies, automate login or use another platform's credentials.
- Never treat challenges, empty results or partial responses as retrieved content.
- Prefer installed dedicated skills and existing web and GitHub retrieval.
`;

function adaptedSkill(content: string): string {
  const routing = content.indexOf("## Routing table");
  const end = content.indexOf("## Environment check", routing);
  const fallback = content.indexOf("## Configure a channel", routing);
  if (routing === -1 || fallback === -1) {
    throw new Error("Upstream skill routing sections changed");
  }
  return adaptedCommands(
    `${skillHeader}${content.slice(routing, end === -1 ? fallback : end)}${skillFooter}`,
  );
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
    license: "MIT",
    skills: "./skills/",
  };
  files.set(
    `${skillDirectory}/SKILL.md`,
    adaptedSkill(await regularText(source, "agent_reach/skill/SKILL_en.md")),
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
      name === "web.md"
        ? original.replaceAll(/\bpython3(?=\s)/gu, pythonCommand)
        : original;
    files.set(`${skillDirectory}/references/${name}`, adaptedCommands(content));
  }
  files.set(`${pluginDirectory}/LICENSE`, await regularText(source, "LICENSE"));
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
    resolve(import.meta.dir, "../harness/plugins/agent-reach"),
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
  const directory = join(home, ".local/share/agent-reach");
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
