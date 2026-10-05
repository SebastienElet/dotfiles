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
    "---\nname: agent-reach\ndescription: MUST USE for anything\n---\n# Upstream skill\n\n## Routing table\n\n[Web](references/web.md)\n\n## Configure a channel\n\nInstall everything automatically.\n",
  );
  await writeFile(
    join(source, "agent_reach/skill/references/web.md"),
    'curl URL\nmcporter call exa.web_search_exa query="query"\nagent-reach doctor --json\n',
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
    join(options.home, ".local/share/agent-reach/marketplace"),
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

test("routes RSS and podcast commands through the isolated Python environment", async () => {
  const options = await fixture();
  await writeFile(
    join(options.source, "agent_reach/skill/references/web.md"),
    'python3 -c "import feedparser"\n',
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
    ".local/share/agent-reach/marketplace",
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
