import { dirname, join } from "node:path";
import { ensureDirectories, metadata } from "./agent-reach-files.ts";
import { lstat, mkdtemp, realpath, rm, symlink } from "node:fs/promises";
import { verifyGitSourceBytes } from "./agent-reach-integrity.ts";
import { z } from "zod";

const sourceSchema = z.object({
  home: z.string().min(1),
  repository: z.string().min(1),
  revision: z.string().regex(/^[a-f0-9]{40}$/u),
});
type SourceOptions = z.infer<typeof sourceSchema>;

async function runAgentReachCommand(
  args: readonly string[],
  environment: Readonly<Record<string, string | undefined>> = {},
): Promise<string> {
  const child = Bun.spawn([...args], {
    stdout: "pipe",
    stderr: "pipe",
    env: { ...process.env, ...environment },
    timeout: 180_000,
  });
  const [output, errors, status] = await Promise.all([
    new Response(child.stdout).text(),
    new Response(child.stderr).text(),
    child.exited,
  ]);
  if (status !== 0) {
    throw new Error(`${args[0]} failed (${status}): ${errors.trim()}`);
  }
  return output.trim();
}

async function verifySource(
  destination: string,
  revision: string,
): Promise<void> {
  const entry = await lstat(destination);
  const target = await realpath(destination);
  if (
    entry.isSymbolicLink() &&
    (dirname(target) !== dirname(destination) ||
      !target.startsWith(join(dirname(destination), ".source-")))
  ) {
    throw new Error(`divergent source: ${destination}`);
  }
  if (!entry.isDirectory() && !entry.isSymbolicLink()) {
    throw new Error(`divergent source: ${destination}`);
  }
  const head = await runAgentReachCommand([
    "git",
    "-C",
    destination,
    "rev-parse",
    "HEAD",
  ]);
  const tree = await runAgentReachCommand([
    "git",
    "--no-replace-objects",
    "-C",
    destination,
    "ls-tree",
    "-r",
    "-z",
    revision,
  ]);
  if (head !== revision) {
    throw new Error(`divergent source checkout: ${destination}`);
  }
  await verifyGitSourceBytes(target, tree);
}

async function fetchAgentReachSource(input: SourceOptions): Promise<string> {
  const options = sourceSchema.parse(input);
  const home = await realpath(options.home);
  const directory = join(home, ".local/share/agent-reach/upstream");
  await ensureDirectories(home, directory);
  const destination = join(directory, options.revision);
  if ((await metadata(destination)) !== null) {
    await verifySource(destination, options.revision);
    return destination;
  }
  const staging = await mkdtemp(join(directory, ".source-"));
  let published = false;
  try {
    await runAgentReachCommand(["git", "init", "--quiet", staging]);
    await runAgentReachCommand([
      "git",
      "-C",
      staging,
      "fetch",
      "--quiet",
      "--depth=1",
      options.repository,
      options.revision,
    ]);
    await runAgentReachCommand([
      "git",
      "-C",
      staging,
      "checkout",
      "--quiet",
      "--detach",
      "FETCH_HEAD",
    ]);
    await verifySource(staging, options.revision);
    await symlink(staging, destination);
    published = true;
  } finally {
    if (!published) {
      await rm(staging, { recursive: true, force: true });
    }
  }
  return destination;
}

export { fetchAgentReachSource, runAgentReachCommand };
