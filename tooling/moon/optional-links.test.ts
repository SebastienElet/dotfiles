import { afterEach, expect, test } from "bun:test";
import { dirname, join } from "node:path";
import {
  lstatSync,
  mkdtempSync,
  readFileSync,
  readlinkSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { fileURLToPath } from "node:url";
import { tmpdir } from "node:os";

const repository = fileURLToPath(new URL("../..", import.meta.url));
const psqlrcSource = join(repository, "home", ".psqlrc");
const fixtures: string[] = [];

function makePostgresql(home: string): Readonly<{
  exitCode: number;
  stdout: string;
  stderr: string;
}> {
  const result = Bun.spawnSync(
    [
      "/usr/bin/make",
      "--no-print-directory",
      "-f",
      join(repository, "Makefile"),
      "postgresql",
    ],
    {
      env: {
        HOME: home,
        PATH: `${dirname(process.execPath)}:/usr/bin:/bin`,
      },
      stdout: "pipe",
      stderr: "pipe",
    },
  );
  return {
    exitCode: result.exitCode,
    stdout: result.stdout.toString(),
    stderr: result.stderr.toString(),
  };
}

function createHome(): string {
  const home = mkdtempSync(join(tmpdir(), "optional-links-test-"));
  fixtures.push(home);
  return home;
}

afterEach(() => {
  for (const fixture of fixtures.splice(0)) {
    rmSync(fixture, { recursive: true, force: true });
  }
});

test("links psqlrc and replays silently", () => {
  const home = createHome();
  expect(makePostgresql(home).exitCode).toBe(0);
  const destination = join(home, ".psqlrc");
  expect(readlinkSync(destination)).toBe(psqlrcSource);
  const before = lstatSync(destination).ino;

  const replay = makePostgresql(home);

  expect(replay).toEqual({ exitCode: 0, stdout: "", stderr: "" });
  expect(lstatSync(destination).ino).toBe(before);
});

test("refuses a divergent psqlrc without replacing it", () => {
  const home = createHome();
  const destination = join(home, ".psqlrc");
  writeFileSync(destination, "personal\n");

  const result = makePostgresql(home);

  expect(result.exitCode).not.toBe(0);
  expect(result.stderr).toContain(
    `${destination} exists and is not the expected symbolic link`,
  );
  expect(readFileSync(destination, "utf8")).toBe("personal\n");
});
