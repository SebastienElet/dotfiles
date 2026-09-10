import {
  appendFile,
  chmod,
  copyFile,
  mkdir,
  readdir,
  rename,
} from "node:fs/promises";
import { basename, join } from "node:path";
import { z } from "zod";

const environment = z
  .object({
    HOME: z.string(),
    UPGRADE_LOG: z.string(),
    UPGRADE_FAILURE: z.string().default(""),
    UPGRADE_PLUGINS: z
      .string()
      .default('[{"id":"first@fixture"},{"id":"second@fixture"}]'),
    UPGRADE_LOCK: z.string().default("updated\n"),
    UPGRADE_NODE_EFFECT: z.enum(["", "publication", "cleanup"]).default(""),
    REAL_GIT: z.string(),
  })
  .parse(process.env);
const argumentOffset = 2;
const command = basename(process.argv[1] ?? "");
const commandArguments = process.argv.slice(argumentOffset);
const invocation = [command, ...commandArguments].join(" ");
await appendFile(environment.UPGRADE_LOG, `${invocation}\n`);
const failureCode = 42;
if (
  invocation.includes(environment.UPGRADE_FAILURE) &&
  environment.UPGRADE_FAILURE !== ""
) {
  process.stderr.write(`simulated failure: ${invocation}\n`);
  process.exit(failureCode);
}
if (command === "git") {
  const child = Bun.spawn([environment.REAL_GIT, ...commandArguments], {
    stdin: "inherit",
    stdout: "inherit",
    stderr: "inherit",
  });
  process.exit(await child.exited);
}
if (command === "date") {
  process.stdout.write("2026-09-07T00:00:00Z\n");
}
if (invocation === "claude plugin list --json") {
  process.stdout.write(`${environment.UPGRADE_PLUGINS}\n`);
}
if (command === "nvim") {
  await Bun.write(
    join(environment.HOME, ".dotfiles/home/.config/nvim/lazy-lock.json"),
    environment.UPGRADE_LOCK,
  );
}
if (invocation === "volta pin node@lts") {
  await copyFile(join(environment.HOME, "pinned-package.json"), "package.json");
}
if (invocation.startsWith("volta install ")) {
  await Bun.write(
    join(environment.HOME, "installed-node"),
    commandArguments.join(" "),
  );
  const repository = join(environment.HOME, ".dotfiles");
  if (environment.UPGRADE_NODE_EFFECT === "publication") {
    await rename(
      join(repository, "package.json"),
      join(environment.HOME, "previous-package.json"),
    );
    await mkdir(join(repository, "package.json"));
  }
  if (environment.UPGRADE_NODE_EFFECT === "cleanup") {
    const entries = await readdir(repository);
    const stage = entries.find((entry) => entry.startsWith(".node-pin."));
    if (stage === undefined) {
      throw new Error("Node staging directory missing");
    }
    const blocked = join(repository, stage, "blocked");
    await mkdir(blocked);
    await Bun.write(join(blocked, "entry"), "cleanup obstruction");
    await chmod(blocked, 0);
  }
}
