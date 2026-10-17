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
    REAL_INSTALL: z.string(),
    UPGRADE_PROTO_STUB: z.string(),
  })
  .parse(process.env);
const argumentOffset = 2;
const command = basename(process.argv[1] ?? "");
const commandArguments = process.argv.slice(argumentOffset);
const invocation = [command, ...commandArguments].join(" ");
await appendFile(environment.UPGRADE_LOG, `${invocation}\n`);
const failureCode = 42;
const executableMode = 0o755;
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
if (command === "curl") {
  process.stdout.write(
    'mkdir -p "$PROTO_HOME/bin"; cp "$UPGRADE_PROTO_STUB" "$PROTO_HOME/bin/proto"\n',
  );
}
if (command === "install") {
  const child = Bun.spawn([environment.REAL_INSTALL, ...commandArguments], {
    stdin: "inherit",
    stdout: "inherit",
    stderr: "inherit",
  });
  process.exit(await child.exited);
}
if (command === "proto") {
  const configuration = z
    .object({ moon: z.string().regex(/^\d+\.\d+\.\d+$/u) })
    .safeParse(Bun.TOML.parse(await Bun.file(".prototools").text()));
  if (!configuration.success) {
    process.stderr.write("invalid Moon pin\n");
    process.exit(1);
  }
  const pin = configuration.data.moon;
  const selected = join(environment.HOME, "selected-moon");
  if (
    commandArguments.join(" ") === "--config-mode local run moon -- --version"
  ) {
    const source = `#!${process.execPath}\nconst version = ${JSON.stringify(pin)};\nconst args = process.argv.slice(2).join(" ");\nawait Bun.write(process.env.UPGRADE_LOG, await Bun.file(process.env.UPGRADE_LOG).text() + "moon " + args + "\\n" + "moon " + version + " " + args + "\\n");\nif (args === "--version") { if (process.env.UPGRADE_FAILURE === "moon --version") { console.error("simulated failure: moon --version"); process.exit(42); } process.stdout.write("moon " + version + "\\n"); }\nif (args.startsWith("exec ") && process.env.UPGRADE_FAILURE === "moon exec") { console.error("simulated failure: moon exec"); process.exit(42); }\n`;
    await Bun.write(selected, source);
    await chmod(selected, executableMode);
    process.stdout.write(`moon ${pin}\n`);
  } else if (commandArguments.join(" ") === "--config-mode local bin moon") {
    process.stdout.write(selected);
  } else {
    process.exit(1);
  }
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
