import { afterEach, expect, test } from "bun:test";
import {
  mkdir,
  mkdtemp,
  readFile,
  realpath,
  rm,
  symlink,
  writeFile,
} from "node:fs/promises";
import { copyAgentReachFile } from "./agent-reach-config.ts";
import { join } from "node:path";
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
  Readonly<{ home: string; source: string; destination: string }>
> {
  const home = await realpath(
    await mkdtemp(join(tmpdir(), "agent-reach-copy-")),
  );
  directories.push(home);
  const source = join(home, "source.json");
  await writeFile(source, "configuration");
  return {
    home,
    source,
    destination: join(home, ".config/agent-reach/mcporter.json"),
  };
}

test("publishes a regular configuration file and preserves replay", async () => {
  const options = await fixture();
  await copyAgentReachFile(options);
  expect(await readFile(options.destination, "utf8")).toBe("configuration");
  await copyAgentReachFile(options);
});

test("preserves foreign files and refuses destination symlinks", async () => {
  const options = await fixture();
  await mkdir(join(options.home, ".config/agent-reach"), { recursive: true });
  await writeFile(options.destination, "foreign");
  await rejects(copyAgentReachFile(options), /divergent/u);
  expect(await readFile(options.destination, "utf8")).toBe("foreign");
  await rm(options.destination);
  await symlink(options.source, options.destination);
  await rejects(copyAgentReachFile(options), /divergent/u);
});
