import { dirname, resolve } from "node:path";
import {
  extractParseTimeCommands,
  inventoryComposeSources,
  inventorySources,
} from "./software-source-inventory.ts";
import { readFileSync } from "node:fs";
import { z } from "zod";

const SUCCESS = 0;
const FAILURE = 1;
const CLI_ARGUMENT_START = 2;
const cliArgumentsSchema = z.tuple([z.string().min(1)]);
const allowedSources = [
  "channel:docker",
  "channel:fisher",
  "channel:homebrew",
  "channel:mas",
  "channel:npm",
  "docker:cloakhq/cloakbrowser:0.5.3",
  "docker:pyd4vinci/scrapling",
  "https://claude.ai/install.sh",
  "https://github.com/catppuccin/bat/raw/main/themes/Catppuccin%20Latte.tmTheme",
  "https://github.com/catppuccin/bat/raw/main/themes/Catppuccin%20Mocha.tmTheme",
  "https://github.com/tmux-plugins/tpm",
  "https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh",
  "https://raw.githubusercontent.com/LibreOffice/dictionaries/f2ff99058268502bdcf4cad25c1ca2935ad8aa7d/en/en_US.aff",
  "https://raw.githubusercontent.com/LibreOffice/dictionaries/f2ff99058268502bdcf4cad25c1ca2935ad8aa7d/en/en_US.dic",
  "https://raw.githubusercontent.com/LibreOffice/dictionaries/f2ff99058268502bdcf4cad25c1ca2935ad8aa7d/fr_FR/dictionaries/fr.aff",
  "https://raw.githubusercontent.com/LibreOffice/dictionaries/f2ff99058268502bdcf4cad25c1ca2935ad8aa7d/fr_FR/dictionaries/fr.dic",
] as const;

function findUndeclaredSources(
  sources: readonly string[],
  allowed: readonly string[] = allowedSources,
): readonly string[] {
  return sources.filter((source) => !allowed.includes(source));
}

function dryRunInstallationGraph(
  makefile: string,
  makeCommand = "make",
): string {
  readFileSync(makefile, "utf8");
  const result = Bun.spawnSync({
    cmd: [
      makeCommand,
      "-B",
      "-n",
      "-f",
      makefile,
      "all",
      "SKIP_PAID_APPS=1",
      "SHELL=/usr/bin/false",
      "MAKE=/usr/bin/false",
    ],
    cwd: dirname(makefile),
    stderr: "pipe",
    stdout: "pipe",
  });
  const stderr = result.stderr.toString();
  if (result.exitCode !== SUCCESS) {
    throw new Error(`make dry-run failed:\n${stderr}`);
  }
  const output = result.stdout.toString();
  if (output.trim() === "") {
    throw new Error("make dry-run returned no installation graph");
  }
  return output;
}

function checkSoftwareSources(makefile: string): readonly string[] {
  const contents = readFileSync(makefile, "utf8");
  const evidence = [
    dryRunInstallationGraph(makefile),
    ...extractParseTimeCommands(contents),
  ].join("\n");
  const sources = [
    ...inventorySources(evidence),
    ...inventoryComposeSources(evidence, makefile),
  ].toSorted();
  if (!sources.includes("channel:homebrew")) {
    throw new Error("installation graph canary missing: channel:homebrew");
  }
  return findUndeclaredSources(sources);
}

function errorMessage(error: unknown): string {
  return error instanceof Error
    ? error.message
    : "unknown software source failure";
}

function main(): number {
  try {
    const [makefile] = cliArgumentsSchema.parse(
      Bun.argv.slice(CLI_ARGUMENT_START),
    );
    const undeclared = checkSoftwareSources(resolve(makefile));
    if (undeclared.length > 0) {
      process.stderr.write(
        `undeclared software sources:\n${undeclared.join("\n")}\n`,
      );
      return FAILURE;
    }
    return SUCCESS;
  } catch (error) {
    process.stderr.write(`${errorMessage(error)}\n`);
    return FAILURE;
  }
}

if (import.meta.main) {
  process.exitCode = main();
}

export { checkSoftwareSources, dryRunInstallationGraph, findUndeclaredSources };
