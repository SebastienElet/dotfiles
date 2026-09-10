import { checkCommand, reportCheckFailure } from "./check-command.ts";
import { mkdtempSync, rmSync } from "node:fs";
import { requiredFiles, skillMarkdownPaths } from "./skill-markdown-paths.ts";
import { join } from "node:path";
import { tmpdir } from "node:os";
import { z } from "zod";

const argumentOffset = 2;
const filesSchema = z.array(z.string().min(1)).min(1);

function runCSpell(
  arguments_: readonly string[],
  env: Readonly<NodeJS.ProcessEnv>,
): Bun.SyncSubprocess<"pipe", "pipe"> {
  return checkCommand(
    ["bun", "x", "--bun", "--no-install", "cspell", ...arguments_],
    { env },
  );
}

function verifyDictionary(
  config: string,
  env: Readonly<NodeJS.ProcessEnv>,
): void {
  const trace = runCSpell(
    [
      "trace",
      "--config",
      config,
      "--dictionary-path",
      "full",
      "--all",
      "rclone",
    ],
    env,
  );
  process.stdout.write(trace.stdout);
  process.stderr.write(trace.stderr);
  if (
    !trace.stdout
      .toString()
      .includes(join(z.string().parse(env.HOME), ".config/cspell/user.txt"))
  ) {
    throw new Error(
      "CSpell trace did not resolve the deployed user dictionary",
    );
  }
}

function main(): void {
  const files = requiredFiles(
    filesSchema.parse(process.argv.slice(argumentOffset)),
  );
  const skills = skillMarkdownPaths();
  const originalHome = z.string().min(1).parse(process.env.HOME);
  const home = mkdtempSync(join(tmpdir(), "cspell-check-"));
  const env = {
    ...process.env,
    HOME: home,
    MOON_HOME: process.env.MOON_HOME ?? join(originalHome, ".moon"),
    PROTO_HOME: process.env.PROTO_HOME ?? join(originalHome, ".proto"),
  };
  const config = join(home, "cspell.json");
  try {
    checkCommand(
      ["moon", "exec", "--quiet", "--ignore-ci-checks", "home:cspell-config"],
      { env },
    );
    verifyDictionary(config, env);
    for (const selection of [
      ["--file", ...files],
      ["--force-check", "--file", ...skills],
    ]) {
      const lint = runCSpell(["lint", "--config", config, ...selection], env);
      process.stdout.write(lint.stdout);
      process.stderr.write(lint.stderr);
    }
  } finally {
    rmSync(home, { recursive: true, force: true });
  }
}

try {
  main();
} catch (error) {
  reportCheckFailure(error);
}
