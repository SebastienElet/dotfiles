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
import { projectAgentReachCursor } from "./agent-reach-cursor.ts";
import { rejects } from "node:assert/strict";
import { tmpdir } from "node:os";

const directories: string[] = [];
afterEach(async () => {
  await Promise.all(
    directories
      .splice(0)
      .map((path) => rm(path, { recursive: true, force: true })),
  );
});

async function fixture(): Promise<
  Readonly<{ home: string; marketplace: string; plugin: string }>
> {
  const home = await realpath(
    await mkdtemp(join(tmpdir(), "agent-reach-cursor-")),
  );
  directories.push(home);
  const marketplace = join(home, "marketplace");
  const plugin = join(marketplace, "plugins/agent-reach");
  await mkdir(join(plugin, ".claude-plugin"), { recursive: true });
  await mkdir(join(plugin, "skills/agent-reach"), { recursive: true });
  await writeFile(
    join(plugin, ".claude-plugin/plugin.json"),
    '{"name":"agent-reach","version":"0.0.1","skills":"./skills/"}\n',
  );
  await writeFile(
    join(plugin, "skills/agent-reach/SKILL.md"),
    "# Agent Reach\n",
  );
  return { home, marketplace, plugin };
}

test("projects the packaged manifest and assets into Cursor with an unchanged replay", async () => {
  const { home, marketplace, plugin } = await fixture();
  const destination = await projectAgentReachCursor(home, marketplace);
  expect(destination).toBe(join(home, ".cursor/plugins/local/agent-reach"));
  expect(
    await readFile(join(destination, ".cursor-plugin/plugin.json"), "utf8"),
  ).toBe(await readFile(join(plugin, ".claude-plugin/plugin.json"), "utf8"));
  expect(
    await readFile(join(destination, "skills/agent-reach/SKILL.md"), "utf8"),
  ).toBe("# Agent Reach\n");
  await projectAgentReachCursor(home, marketplace);
  expect(await readdir(join(home, ".cursor/plugins/local"))).toEqual([
    "agent-reach",
  ]);
});

test("a failed copy leaves no partial plugin and can be replayed after fixing the resource", async () => {
  const { home, marketplace, plugin } = await fixture();
  await writeFile(join(plugin, ".cursor-plugin"), "conflicting resource");
  await rejects(projectAgentReachCursor(home, marketplace), /Divergent/u);
  expect(await readdir(join(home, ".cursor/plugins/local"))).toEqual([]);
  await rm(join(plugin, ".cursor-plugin"));
  const destination = await projectAgentReachCursor(home, marketplace);
  expect(
    await readFile(join(destination, ".cursor-plugin/plugin.json"), "utf8"),
  ).toContain('"name":"agent-reach"');
});

test("refuses foreign projections and linked destination ancestors without changing their contents", async () => {
  const { home, marketplace } = await fixture();
  const destination = join(home, ".cursor/plugins/local/agent-reach");
  await mkdir(destination, { recursive: true });
  await writeFile(join(destination, "foreign"), "keep");
  await rejects(projectAgentReachCursor(home, marketplace), /divergent/u);
  expect(await readdir(destination)).toEqual(["foreign"]);
  await rm(join(home, ".cursor"), { recursive: true });
  await mkdir(join(home, "foreign"));
  await symlink(join(home, "foreign"), join(home, ".cursor"));
  await rejects(projectAgentReachCursor(home, marketplace), /Divergent/u);
  expect(await readdir(join(home, "foreign"))).toEqual([]);
});

test("rejects missing manifests and linked source resources before creating a projection", async () => {
  const { home, marketplace, plugin } = await fixture();
  const skill = join(plugin, "skills/agent-reach/SKILL.md");
  await rm(skill);
  await symlink(join(plugin, ".claude-plugin/plugin.json"), skill);
  await rejects(
    projectAgentReachCursor(home, marketplace),
    /divergent resource/u,
  );
  expect(await readdir(home)).toEqual(["marketplace"]);
  await rm(skill);
  await writeFile(skill, "# Agent Reach\n");
  await rm(join(plugin, ".claude-plugin/plugin.json"));
  await rejects(projectAgentReachCursor(home, marketplace), /ENOENT/u);
  expect(await readdir(home)).toEqual(["marketplace"]);
});
