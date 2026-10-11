import {
  event,
  git,
  repository,
  scan,
  token,
} from "./secret-scan-test-support.ts";
import { expect, test } from "bun:test";
import { mkdir, writeFile } from "node:fs/promises";
import { join } from "node:path";

test("the CI command rejects a bypassed token and an empty forced .env at HEAD", async () => {
  const root = await repository();
  const base = git(root, "rev-parse", "HEAD");
  await writeFile(join(root, "secret.txt"), `TOKEN=${token}\n`);
  git(root, "add", "secret.txt");
  git(
    root,
    "-c",
    "core.hooksPath=/dev/null",
    "commit",
    "-qm",
    "bypass fixture hook",
  );
  const head = git(root, "rev-parse", "HEAD");
  expect(
    scan(
      root,
      "ci",
      await event(root, {
        pull_request: { base: { sha: base }, head: { sha: head } },
      }),
    ).exitCode,
  ).not.toBe(0);
  const clean = await repository();
  await writeFile(join(clean, ".env"), "");
  git(clean, "add", "-f", ".env");
  git(clean, "commit", "-qm", "empty private path");
  expect(scan(clean, "current").exitCode).not.toBe(0);
});

test("the CI command detects a fictitious token added then deleted in two introduced commits", async () => {
  const root = await repository();
  const base = git(root, "rev-parse", "HEAD");
  await writeFile(join(root, "secret.txt"), `TOKEN=${token}\n`);
  git(root, "add", "secret.txt");
  git(root, "commit", "-qm", "add fictitious token");
  git(root, "rm", "-q", "secret.txt");
  git(root, "commit", "-qm", "delete fictitious token");
  const head = git(root, "rev-parse", "HEAD");
  expect(scan(root, "current").exitCode).toBe(0);
  const env = await event(root, {
    pull_request: { base: { sha: base }, head: { sha: head } },
  });
  const refusal = scan(root, "ci", env);
  expect(refusal.exitCode).not.toBe(0);
  expect(refusal.stderr.toString()).toContain("github-pat");
  expect(refusal.stderr.toString()).not.toContain(token);
});

test("CI rejects missing refs, malformed events and absent event metadata", async () => {
  const root = await repository();
  const head = git(root, "rev-parse", "HEAD");
  const shaLength = 40;
  const missing = "f".repeat(shaLength);
  expect(
    scan(
      root,
      "ci",
      await event(root, {
        pull_request: { base: { sha: missing }, head: { sha: head } },
      }),
    ).exitCode,
  ).not.toBe(0);
  expect(scan(root, "ci", await event(root, {})).exitCode).not.toBe(0);
  expect(
    scan(root, "ci", { GITHUB_EVENT_NAME: "", GITHUB_EVENT_PATH: "" }).exitCode,
  ).not.toBe(0);
});

test("CI scans committed blobs even when attributes or the working tree hide them", async () => {
  const root = await repository();
  await writeFile(join(root, "hidden.txt"), `TOKEN=${token}\n`);
  await writeFile(join(root, ".gitattributes"), "hidden.txt export-ignore\n");
  git(root, "add", "hidden.txt", ".gitattributes");
  git(root, "commit", "-qm", "hidden fictitious token");
  await writeFile(join(root, "hidden.txt"), "safe\n");
  expect(scan(root, "current").exitCode).not.toBe(0);
});

test("scanner error or missing report never becomes a successful scan", async () => {
  const root = await repository();
  const bin = join(root, "bin");
  await mkdir(bin);
  await writeFile(join(bin, "gitleaks"), "#!/bin/sh\nexit 2\n", {
    mode: 0o755,
  });
  expect(
    scan(root, "staged", { PATH: `${bin}:/usr/bin:/bin` }).exitCode,
  ).not.toBe(0);
  await writeFile(join(bin, "gitleaks"), "#!/bin/sh\nexit 0\n", {
    mode: 0o755,
  });
  expect(
    scan(root, "staged", { PATH: `${bin}:/usr/bin:/bin` }).exitCode,
  ).not.toBe(0);
});

test("CI rejects an event head unrelated to the checked-out commit", async () => {
  const root = await repository();
  const base = git(root, "rev-parse", "HEAD");
  git(root, "checkout", "-qb", "other");
  await writeFile(join(root, "other.txt"), "safe\n");
  git(root, "add", "other.txt");
  git(root, "commit", "-qm", "other head");
  const head = git(root, "rev-parse", "HEAD");
  git(root, "checkout", "-q", "--detach", base);
  const env = await event(root, {
    pull_request: { base: { sha: base }, head: { sha: head } },
  });
  expect(scan(root, "ci", env).exitCode).not.toBe(0);
});

test("CI passes documentation-only and scanner-configuration-only commits", async () => {
  const root = await repository();
  const base = git(root, "rev-parse", "HEAD");
  await writeFile(join(root, "README.md"), "Safe documentation\n");
  git(root, "add", "README.md");
  git(root, "commit", "-qm", "documentation only");
  const first = git(root, "rev-parse", "HEAD");
  expect(
    scan(
      root,
      "ci",
      await event(root, {
        pull_request: { base: { sha: base }, head: { sha: first } },
      }),
    ).exitCode,
  ).toBe(0);
  const config = await Bun.file(join(root, ".gitleaks.toml")).text();
  await writeFile(join(root, ".gitleaks.toml"), `${config}\n`);
  git(root, "add", ".gitleaks.toml");
  git(root, "commit", "-qm", "configuration only");
  const head = git(root, "rev-parse", "HEAD");
  expect(
    scan(
      root,
      "ci",
      await event(root, {
        pull_request: { base: { sha: first }, head: { sha: head } },
      }),
    ).exitCode,
  ).toBe(0);
});

test.each(["push", "merge_group", "workflow_dispatch", "new-branch"])(
  "%s scans all introduced history or falls back to a complete scan",
  async (kind) => {
    const root = await repository();
    const base = git(root, "rev-parse", "HEAD");
    await writeFile(join(root, "transient.txt"), `TOKEN=${token}\n`);
    git(root, "add", "transient.txt");
    git(root, "commit", "-qm", "fictitious token");
    git(root, "rm", "-q", "transient.txt");
    git(root, "commit", "-qm", "remove fictitious token");
    const head = git(root, "rev-parse", "HEAD");
    const shaLength = 40;
    const data =
      kind === "merge_group"
        ? { merge_group: { base_sha: base, head_sha: head } }
        : {
            before: kind === "new-branch" ? "0".repeat(shaLength) : base,
            after: head,
          };
    const env = {
      ...(await event(root, data)),
      GITHUB_EVENT_NAME: kind === "new-branch" ? "push" : kind,
    };
    expect(scan(root, "ci", env).exitCode).not.toBe(0);
  },
);
