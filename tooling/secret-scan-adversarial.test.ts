import {
  event,
  git,
  repository,
  scan,
  token,
} from "./secret-scan-test-support.ts";
import { expect, test } from "bun:test";
import { rm, symlink, writeFile } from "node:fs/promises";
import { join } from "node:path";

const disguises = ["binary", "textconv", "NUL"] as const;

async function hiddenToken(
  root: string,
  disguise: (typeof disguises)[number],
): Promise<void> {
  if (disguise === "binary") {
    await writeFile(join(root, ".gitattributes"), "hidden.txt -diff\n");
  }
  if (disguise === "textconv") {
    await writeFile(join(root, ".gitattributes"), "hidden.txt diff=quiet\n");
    git(root, "config", "diff.quiet.textconv", "true");
  }
  await writeFile(
    join(root, "hidden.txt"),
    `${disguise === "NUL" ? "\0" : ""}TOKEN=${token}\n`,
  );
  git(root, "add", "hidden.txt");
  if (disguise !== "NUL") {
    git(root, "add", ".gitattributes");
  }
}

test.each([...disguises])(
  "staged blobs cannot be hidden with %s",
  async (disguise) => {
    const root = await repository();
    await hiddenToken(root, disguise);
    expect(scan(root).exitCode).not.toBe(0);
  },
);

test("an indexed type change cannot be hidden by an unstaged binary attribute", async () => {
  const root = await repository();
  const path = join(root, "hidden.txt");
  await symlink("safe-reference", path);
  git(root, "add", "hidden.txt");
  git(root, "commit", "-qm", "safe symlink");
  await rm(path);
  await writeFile(path, `TOKEN=${token}\n`);
  git(root, "add", "hidden.txt");
  await writeFile(join(root, ".gitattributes"), "hidden.txt -diff\n");
  expect(scan(root).exitCode).not.toBe(0);
});

test.each([...disguises])(
  "introduced history cannot be hidden with %s",
  async (disguise) => {
    const root = await repository();
    const base = git(root, "rev-parse", "HEAD");
    await hiddenToken(root, disguise);
    git(root, "commit", "-qm", "hidden fictitious token");
    git(root, "rm", "-q", "hidden.txt");
    git(root, "commit", "-qm", "remove fictitious token");
    const head = git(root, "rev-parse", "HEAD");
    expect(scan(root, "current").exitCode).toBe(0);
    const env = await event(root, {
      pull_request: { base: { sha: base }, head: { sha: head } },
    });
    const refusal = scan(root, "ci", env);
    expect(refusal.exitCode).not.toBe(0);
    expect(refusal.stderr.toString()).toContain("github-pat");
    expect(refusal.stderr.toString()).not.toContain(token);
  },
);
