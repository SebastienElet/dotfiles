import { appendFileSync, mkdirSync, writeFileSync } from "node:fs";
import { join } from "node:path";

const mode = process.env.DEPLOYMENT_PROVIDER_MODE;

if (mode === "ln") {
  appendFileSync(required("DEPLOYMENT_MARKER"), "called\n");
  forward(required("DEPLOYMENT_REAL_COMMAND"));
} else if (mode === "git") {
  const expected = JSON.parse(required("DEPLOYMENT_FAIL_ARGUMENTS"));
  if (JSON.stringify(process.argv.slice(2)) === JSON.stringify(expected)) {
    process.exit(128);
  }
  forward(required("DEPLOYMENT_REAL_COMMAND"));
} else if (mode === "fish-success") {
  writeFileSync(
    required("DEPLOYMENT_MARKER"),
    `${process.argv.slice(2).join(" ")}\n`,
  );
  const functions = join(required("HOME"), ".config", "fish", "functions");
  mkdirSync(functions, { recursive: true });
  writeFileSync(join(functions, "fzf_configure_bindings.fish"), "");
} else if (mode === "fish-empty") {
  process.exit(0);
} else {
  process.stderr.write(
    `unknown deployment provider mode: ${mode ?? "missing"}\n`,
  );
  process.exit(64);
}

function forward(command: string): never {
  const result = Bun.spawnSync([command, ...process.argv.slice(2)], {
    stdin: "inherit",
    stdout: "inherit",
    stderr: "inherit",
  });
  process.exit(result.exitCode);
}

function required(name: string): string {
  const value = process.env[name];
  if (value === undefined || value === "")
    throw new Error(`${name} is required`);
  return value;
}
