import {
  cpSync,
  existsSync,
  mkdirSync,
  mkdtempSync,
  readdirSync,
  realpathSync,
  renameSync,
  rmSync,
} from "node:fs";
import { dirname, join } from "node:path";
import { deployLink } from "./deploy-link.ts";
import { tmpdir } from "node:os";
import { z } from "zod";

const argumentsSchema = z.tuple([z.string().min(1), z.string().min(1)]);
const argumentOffset = 2;

function partialPluginPaths(directory: string): string[] {
  const functions = join(directory, "functions");
  const wrappers = existsSync(functions)
    ? readdirSync(functions)
        .filter((name) => name.startsWith("_fzf") && name.endsWith(".fish"))
        .map((name) => join("functions", name))
    : [];
  return [
    ...wrappers,
    "functions/fzf_configure_bindings.fish",
    "completions/fzf_configure_bindings.fish",
    "conf.d/fzf.fish",
  ].filter((path) => existsSync(join(directory, path)));
}

function installFishPlugins(directory: string): void {
  const bindings = join(directory, "functions", "fzf_configure_bindings.fish");
  const configuration = join(directory, "conf.d", "fzf.fish");
  if (existsSync(bindings) && existsSync(configuration)) {
    return;
  }
  const backup = mkdtempSync(join(tmpdir(), "fish-plugins-"));
  try {
    for (const path of partialPluginPaths(directory)) {
      mkdirSync(dirname(join(backup, path)), { recursive: true });
      renameSync(join(directory, path), join(backup, path));
    }
    const result = Bun.spawnSync(
      ["fish", "-c", "fisher install PatrickF1/fzf.fish"],
      { stderr: "inherit", stdout: "inherit" },
    );
    if (
      result.exitCode !== 0 ||
      !existsSync(bindings) ||
      !existsSync(configuration)
    ) {
      throw new Error(
        `Fisher did not install ${bindings} and ${configuration}`,
      );
    }
  } catch (error) {
    cpSync(backup, realpathSync(directory), {
      recursive: true,
      verbatimSymlinks: true,
    });
    throw error;
  } finally {
    rmSync(backup, { force: true, recursive: true });
  }
}

if (import.meta.main) {
  try {
    const [source, destination] = argumentsSchema.parse(
      process.argv.slice(argumentOffset),
    );
    deployLink(source, destination);
    installFishPlugins(destination);
  } catch (error) {
    process.stderr.write(
      `Error: ${error instanceof Error ? error.message : String(error)}\n`,
    );
    process.exitCode = 1;
  }
}

export { installFishPlugins };
