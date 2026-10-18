import { afterEach, expect, test } from "bun:test";
import {
  mkdir,
  mkdtemp,
  readFile,
  readdir,
  realpath,
  rm,
  symlink,
  writeFile,
} from "node:fs/promises";
import { join } from "node:path";
import { prepareAgentReachPlugin } from "./agent-reach-package.ts";
import { rejects } from "node:assert/strict";
import { tmpdir } from "node:os";

const directories: string[] = [];

afterEach(async () => {
  await Promise.all(
    directories
      .splice(0)
      .map((directory) => rm(directory, { recursive: true, force: true })),
  );
});

async function fixture(): Promise<
  Readonly<{ source: string; home: string; version: string; revision: string }>
> {
  const directory = await mkdtemp(join(tmpdir(), "agent-reach-package-"));
  directories.push(directory);
  const source = join(directory, "upstream");
  const home = join(directory, "home");
  await mkdir(join(source, "agent_reach/skill/references"), {
    recursive: true,
  });
  await mkdir(join(source, "agent_reach/scripts"));
  await mkdir(home);
  await writeFile(join(source, "LICENSE"), "Upstream license\n");
  await writeFile(
    join(source, "agent_reach/skill/SKILL_en.md"),
    '---\nname: agent-reach\ndescription: MUST USE for anything\n---\n# Upstream skill\n\n## Routing table\n\n[Web](references/web.md)\n\ncurl -s "https://r.jina.ai/URL"\n\n## Configure a channel\n\nInstall everything automatically.\n',
  );
  await writeFile(
    join(source, "agent_reach/skill/references/web.md"),
    'curl URL\nmcporter call exa.web_search_exa query="query"\nagent-reach doctor --json\n## Web Reader (MCP)\nmcporter call web-reader.webReader\n## RSS (feedparser)\n',
  );
  await writeFile(
    join(source, "agent_reach/skill/references/video.md"),
    "yt-dlp URL\n~/.agent-reach/tools/xiaoyuzhou/transcribe.sh URL\n",
  );
  await writeFile(
    join(source, "agent_reach/scripts/transcribe_xiaoyuzhou.sh"),
    "#!/bin/bash\nprintf transcript\n",
  );
  return {
    source,
    home: await realpath(home),
    version: "0.1.0",
    revision: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
  };
}

test("packages upstream resources for both hosts and preserves an unchanged replay", async () => {
  const options = await fixture();
  const marketplace = await prepareAgentReachPlugin(options);
  expect(marketplace).toBe(
    join(options.home, ".local/share/agent-reach/adapter/1.0.0/marketplace"),
  );
  const plugin = join(marketplace, "plugins/agent-reach");
  expect(await readFile(join(plugin, "LICENSE"), "utf8")).toBe(
    "Upstream license\n",
  );
  const skill = await readFile(
    join(plugin, "skills/agent-reach/SKILL.md"),
    "utf8",
  );
  expect(skill).toContain("[Web](references/web.md)");
  expect(skill).not.toContain("MUST USE for anything");
  expect(skill).not.toContain("Install everything automatically");
  expect(skill).toContain("explicitly authorizes inspection");
  const web = await readFile(
    join(plugin, "skills/agent-reach/references/web.md"),
    "utf8",
  );
  expect(web).toContain(
    'mcporter --config "$HOME/.config/agent-reach/mcporter.json" call',
  );
  expect(web).toContain(
    'MCPORTER_CONFIG="$HOME/.config/agent-reach/mcporter.json" agent-reach doctor --json',
  );
  const video = await readFile(
    join(plugin, "skills/agent-reach/references/video.md"),
    "utf8",
  );
  expect(video).toContain("yt-dlp --js-runtimes node");
  expect(video).toContain("scripts/transcribe-xiaoyuzhou.sh");
  const before = await Bun.file(
    join(plugin, "skills/agent-reach/SKILL.md"),
  ).stat();
  expect(await prepareAgentReachPlugin(options)).toBe(marketplace);
  const after = await Bun.file(
    join(plugin, "skills/agent-reach/SKILL.md"),
  ).stat();
  expect(after.ino).toBe(before.ino);
});

test.each([
  "agent-reach install --env-only",
  "pipx run --spec twitter-cli==0.8.5 twitter search query",
  "pipx upgrade twitter-cli",
  "npm --global install mcporter@0.14.2",
  "pip install --upgrade twitter-cli==0.8.5",
  "uv pip install twitter-cli==0.8.5",
  "npm i mcporter@0.14.2",
  "brew install ffmpeg",
  "brew update && brew upgrade ffmpeg",
  "npm update mcporter",
  "bun add mcporter@0.14.2",
  "pnpm dlx mcporter@0.14.2 list",
  "npx --package=mcporter@0.14.2 mcporter list",
  "bunx mcporter@0.14.2 list",
  "uvx twitter-cli==0.8.5 search query",
  "uv run --with 'git+https://github.com/can4hou6joeng4/boss-agent-cli.git@4c991b77086a203173bf08a4cb64a23af6514fe6' boss search query",
])(
  "rewrites package acquisition in packaged references: %s",
  async (command) => {
    const options = await fixture();
    await writeFile(
      join(options.source, "agent_reach/skill/references/social.md"),
      `Retry \`${command}\`.\n\`\`\`bash\n${command}\n\`\`\`\ntwitter search query\npipx list\n`,
    );
    const marketplace = await prepareAgentReachPlugin(options);
    const reference = await readFile(
      join(
        marketplace,
        "plugins/agent-reach/skills/agent-reach/references/social.md",
      ),
      "utf8",
    );
    expect(reference).toBe(
      "Retry `install the declared prerequisites explicitly; see the adapter README`.\n```bash\ninstall the declared prerequisites explicitly; see the adapter README\n```\ntwitter search query\npipx list\n",
    );
  },
);

test("routes RSS and podcast commands through the isolated Python environment", async () => {
  const options = await fixture();
  await writeFile(
    join(options.source, "agent_reach/skill/references/web.md"),
    '## Web Reader (MCP)\nmcporter call web-reader.webReader\n## RSS (feedparser)\npython3 -c "import feedparser"\n',
  );
  const marketplace = await prepareAgentReachPlugin(options);
  const plugin = join(marketplace, "plugins/agent-reach");
  const web = await readFile(
    join(plugin, "skills/agent-reach/references/web.md"),
    "utf8",
  );
  expect(web).toContain("scripts/agent-reach-python");
  const podcast = await readFile(
    join(plugin, "scripts/transcribe-xiaoyuzhou.sh"),
    "utf8",
  );
  expect(podcast).toContain(
    'export PATH="$HOME/.local/share/agent-reach/python-tools/agent-reach/bin:$PATH"',
  );
});

test("packages bounded Jina reads without advertising the unconfigured Web Reader", async () => {
  const options = await fixture();
  const recipeCount = 2;
  await writeFile(
    join(options.source, "agent_reach/skill/references/web.md"),
    [
      "## Jina Reader",
      'curl -s "https://r.jina.ai/URL"',
      'curl -s "https://r.jina.ai/https://example.com/article"',
      "## Web Reader (MCP)",
      '```bash\nmcporter call web-reader.webReader url="https://example.com"\n```',
      "## RSS (feedparser)",
      'python3 -c "import feedparser"',
      "| Images | web-reader MCP |",
      "",
    ].join("\n"),
  );
  const marketplace = await prepareAgentReachPlugin(options);
  const web = await readFile(
    join(
      marketplace,
      "plugins/agent-reach/skills/agent-reach/references/web.md",
    ),
    "utf8",
  );
  expect(web).not.toContain("web-reader");
  expect(web).not.toContain("Web Reader (MCP)");
  expect(web.match(/--max-time 30/gu)).toHaveLength(recipeCount);
  expect(web.match(/--connect-timeout 10/gu)).toHaveLength(recipeCount);
  expect(web.match(/--include/gu)).toHaveLength(recipeCount);
  expect(web.match(/X-No-Cache: true/gu)).toHaveLength(recipeCount);
  expect(web.match(/DNT: 1/gu)).toHaveLength(recipeCount);
  expect(web).toContain("scripts/agent-reach-python");
  expect(web).toContain("https://r.jina.ai/https://example.com/article");
  const skill = await readFile(
    join(marketplace, "plugins/agent-reach/skills/agent-reach/SKILL.md"),
    "utf8",
  );
  expect(skill).toContain("--max-time 30");
  expect(skill).not.toContain('curl -s "https://r.jina.ai/');
});

test.each([
  "## Web Reader v2\nweb-reader.webReader\n## RSS (feedparser)\n",
  "## Web Reader (MCP)\nweb-reader.webReader\n## Feeds\n",
  "## RSS (feedparser)\n## Web Reader (MCP)\nweb-reader.webReader\n",
  "## Web Reader (MCP)\nweb-reader.webReader\n## RSS (feedparser)\nweb-reader.webReader\n",
])(
  "rejects changed upstream web structure without publishing: %s",
  async (web) => {
    const options = await fixture();
    await writeFile(
      join(options.source, "agent_reach/skill/references/web.md"),
      web,
    );
    await rejects(
      prepareAgentReachPlugin(options),
      /Upstream web reference sections changed/u,
    );
    expect(await readdir(options.home)).toEqual([]);
  },
);

test("missing upstream resources do not publish a partial marketplace", async () => {
  const options = await fixture();
  await rm(join(options.source, "agent_reach/skill/references"), {
    recursive: true,
  });
  await rejects(prepareAgentReachPlugin(options));
  expect(await readdir(options.home)).toEqual([]);
});

test("refuses a foreign destination and preserves its content", async () => {
  const options = await fixture();
  const destination = join(
    options.home,
    ".local/share/agent-reach/adapter/1.0.0/marketplace",
  );
  await mkdir(destination, { recursive: true });
  await writeFile(join(destination, "personal.txt"), "keep");
  await rejects(prepareAgentReachPlugin(options), /divergent/u);
  expect(await readFile(join(destination, "personal.txt"), "utf8")).toBe(
    "keep",
  );
});

test("refuses an edited installation and does not repair it implicitly", async () => {
  const options = await fixture();
  const destination = await prepareAgentReachPlugin(options);
  const path = join(destination, "plugins/agent-reach/LICENSE");
  await writeFile(path, "edited");
  await rejects(prepareAgentReachPlugin(options), /divergent/u);
  expect(await readFile(path, "utf8")).toBe("edited");
});

test("refuses symlinked source resources and destination parents", async () => {
  const options = await fixture();
  const resource = join(options.source, "agent_reach/skill/references/web.md");
  await rm(resource);
  await symlink(join(options.source, "LICENSE"), resource);
  await rejects(prepareAgentReachPlugin(options));
  await rm(resource);
  await writeFile(resource, "web");
  await symlink(options.source, join(options.home, ".local"));
  await rejects(prepareAgentReachPlugin(options));
  expect(await readdir(options.source)).not.toContain("share");
});

test("names the marketplace independently from the upstream plugin", async () => {
  const marketplace = await prepareAgentReachPlugin(await fixture());
  const claude = await readFile(
    join(marketplace, ".claude-plugin/marketplace.json"),
    "utf8",
  );
  const codex = await readFile(
    join(marketplace, ".agents/plugins/marketplace.json"),
    "utf8",
  );
  const skill = await readFile(
    join(marketplace, "plugins/agent-reach/skills/agent-reach/SKILL.md"),
    "utf8",
  );
  expect(skill).not.toContain("moon run");
  expect(
    await readFile(
      join(marketplace, "plugins/agent-reach/ADAPTER-LICENSE"),
      "utf8",
    ),
  ).toContain("Copyright (c) 2014, Sébastien ELET");
  expect(claude).toContain('"name": "agent-reach-adapter"');
  expect(codex).toContain('"name": "agent-reach-adapter"');
});
