import { copyFile, mkdir, writeFile } from "node:fs/promises";
import { expect, test } from "bun:test";
import {
  git,
  gitEnvironment,
  repository,
  scan,
  sourceRoot,
  token,
} from "./secret-scan-test-support.ts";
import { join } from "node:path";

test("rejects an empty forced .env and accepts the exact clean template", async () => {
  const root = await repository();
  await writeFile(join(root, ".env"), "");
  git(root, "add", "-f", ".env");
  expect(scan(root).exitCode).not.toBe(0);
  git(root, "rm", "--cached", ".env");
  await writeFile(join(root, ".env.example"), "TOKEN=\n");
  git(root, "add", ".env.example");
  expect(scan(root).exitCode).toBe(0);
});

test("rejects the staged fictitious token after the working file is cleaned", async () => {
  const root = await repository();
  await writeFile(join(root, "file with spaces.txt"), `TOKEN=${token}\n`);
  git(root, "add", ".");
  await writeFile(join(root, "file with spaces.txt"), "safe\n");
  const result = scan(root);
  expect(result.exitCode).not.toBe(0);
  expect(result.stdout.toString() + result.stderr.toString()).not.toContain(
    token,
  );
});

test("scans only the index for a partially staged file", async () => {
  const root = await repository();
  await writeFile(join(root, "partial.txt"), "safe\n");
  git(root, "add", "partial.txt");
  await writeFile(join(root, "partial.txt"), `TOKEN=${token}\n`);
  expect(scan(root).exitCode).toBe(0);
  git(root, "add", "partial.txt");
  expect(scan(root).exitCode).not.toBe(0);
});

test("rejects renaming a tracked empty file to a nested private path", async () => {
  const root = await repository();
  await mkdir(join(root, "directory with spaces"));
  await writeFile(join(root, "safe.txt"), "");
  git(root, "add", "safe.txt");
  git(root, "commit", "-qm", "empty file");
  git(root, "mv", "safe.txt", "directory with spaces/.env.production");
  expect(scan(root).exitCode).not.toBe(0);
});

test("accepts version.env and public keys but detects runtime-generated private keys", async () => {
  const root = await repository();
  await writeFile(join(root, "version.env"), "VERSION=1\n");
  const generated = Bun.spawnSync([
    "ssh-keygen",
    "-q",
    "-t",
    "ed25519",
    "-N",
    "",
    "-f",
    join(root, "temporary-key"),
  ]);
  expect(generated.exitCode).toBe(0);
  git(root, "add", "version.env", "temporary-key.pub");
  expect(scan(root).exitCode).toBe(0);
  git(root, "add", "temporary-key");
  expect(scan(root).exitCode).not.toBe(0);
});

test("detects a token in a permitted template despite inline or environment bypasses", async () => {
  const root = await repository();
  await writeFile(
    join(root, ".env.example"),
    `TOKEN=${token} # gitleaks:allow\n`,
  );
  await writeFile(join(root, ".gitleaksignore"), ".env.example:github-pat:1\n");
  await writeFile(join(root, "disabled.toml"), "[extend]\nuseDefault=false\n");
  git(root, "add", ".env.example");
  expect(
    scan(root, "staged", {
      GITLEAKS_CONFIG: join(root, "disabled.toml"),
      GITLEAKS_CONFIG_TOML: "invalid",
      SKIP: "gitleaks",
    }).exitCode,
  ).not.toBe(0);
});

test("the narrow AWS exception does not allow a neighboring fictitious token", async () => {
  const root = await repository();
  const fictitiousAwsId = ["AKIA", "ABCDEFGHIJKLMNOP"].join("");
  const path =
    "tooling/arnes/tests/measure_hook/collection_boundaries/redaction.rs";
  await mkdir(
    join(root, "tooling/arnes/tests/measure_hook/collection_boundaries"),
    { recursive: true },
  );
  await writeFile(join(root, path), `let fictitious = "${fictitiousAwsId}";\n`);
  git(root, "add", path);
  expect(scan(root).exitCode).toBe(0);
  git(root, "commit", "-qm", "allowed fictitious AWS fixture");
  expect(scan(root, "current").exitCode).toBe(0);
  await writeFile(
    join(root, path),
    `let fictitious = "${fictitiousAwsId}";\nTOKEN=${token}\n`,
  );
  git(root, "add", path);
  expect(scan(root).exitCode).not.toBe(0);
});

test("fails closed for an absent scanner and invalid indexed configuration", async () => {
  const root = await repository();
  expect(scan(root, "staged", { PATH: "/usr/bin:/bin" }).exitCode).not.toBe(0);
  await writeFile(join(root, ".gitleaks.toml"), "invalid TOML [");
  git(root, "add", ".gitleaks.toml");
  expect(scan(root).exitCode).not.toBe(0);
});

test.each([".ENV", ".EnV.production", "ID_RSA", "backup.P12", "backup.PFX"])(
  "private path %s is ignored and refused regardless of case",
  async (name) => {
    const root = await repository();
    await copyFile(join(sourceRoot, ".gitignore"), join(root, ".gitignore"));
    git(root, "config", "core.ignorecase", "false");
    const ignored = Bun.spawnSync(
      ["git", "-C", root, "check-ignore", "--no-index", name],
      { env: gitEnvironment },
    );
    expect(ignored.exitCode).toBe(0);
    await writeFile(join(root, name), "");
    git(root, "add", "-f", name);
    expect(scan(root).exitCode).not.toBe(0);
    git(root, "commit", "-qm", "empty private path fixture");
    expect(scan(root, "current").exitCode).not.toBe(0);
  },
);
