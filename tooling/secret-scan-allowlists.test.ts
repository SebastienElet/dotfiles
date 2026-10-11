import { dirname, join } from "node:path";
import { expect, test } from "bun:test";
import { git, repository, scan, token } from "./secret-scan-test-support.ts";
import { mkdir, readFile, writeFile } from "node:fs/promises";
import { createHash } from "node:crypto";

const headerStart = ["-----", "BEGIN"].join("");
const headerBlock = [
  `        ("pem-pkcs8", "${headerStart} PRIVATE KEY-----"),`,
  `        ("pem-rsa-case", "${headerStart.toLowerCase()} rsa private key-----"),`,
  `        ("pem-openssh", "${headerStart} OPENSSH PRIVATE KEY-----"),`,
].join("\n");

test("header-only fixtures pass but a generated private key in the same file is refused", async () => {
  const root = await repository();
  const path = "tooling/agent-memory/tests/memory_validation.rs";
  await mkdir(dirname(join(root, path)), { recursive: true });
  await writeFile(join(root, path), `${headerBlock}\n`);
  git(root, "add", path);
  expect(scan(root).exitCode).toBe(0);
  git(root, "commit", "-qm", "header-only fixture");
  expect(scan(root, "current").exitCode).toBe(0);
  const generated = Bun.spawnSync([
    "ssh-keygen",
    "-q",
    "-t",
    "ed25519",
    "-N",
    "",
    "-f",
    join(root, "runtime-key"),
  ]);
  expect(generated.exitCode).toBe(0);
  await writeFile(
    join(root, path),
    `${headerBlock}\n${await readFile(join(root, "runtime-key"), "utf8")}`,
  );
  git(root, "add", path);
  expect(scan(root).exitCode).not.toBe(0);
});

test.each([
  [
    "docs/evidence/issue-160-calibration-2026-09-21/input-hashes.json",
    "skills/linear-workflow/references/bitbucket.md",
  ],
  [
    "harness/skills/linear-workflow/evals/2026-09-16/environment.json",
    "harness/skills/linear-workflow/references/bitbucket.md",
  ],
])(
  "SHA-256 metadata in %s passes while an adjacent fictitious token is refused",
  async (path, key) => {
    const root = await repository();
    const digest = createHash("sha256")
      .update("Fictitious source fixture")
      .digest("hex");
    await mkdir(dirname(join(root, path)), { recursive: true });
    await writeFile(join(root, path), `  "${key}": "${digest}",\n`);
    git(root, "add", path);
    expect(scan(root).exitCode).toBe(0);
    git(root, "commit", "-qm", "source digest fixture");
    expect(scan(root, "current").exitCode).toBe(0);
    await writeFile(
      join(root, path),
      `  "${key}": "${digest}",\nTOKEN=${token}\n`,
    );
    git(root, "add", path);
    expect(scan(root).exitCode).not.toBe(0);
  },
);
