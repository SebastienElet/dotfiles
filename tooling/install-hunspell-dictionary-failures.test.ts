import { afterEach, expect, test } from "bun:test";
import {
  cleanupFixtures,
  createFixture,
  runArguments,
  runInstaller,
  serve,
  sha256,
  temporaryFiles,
  url,
} from "./install-hunspell-dictionary-test-support.ts";
import {
  mkdirSync,
  readFileSync,
  renameSync,
  symlinkSync,
  writeFileSync,
} from "node:fs";
import { join } from "node:path";

const content = "dictionary\n";
const checksum = sha256(content);
const servers: ReturnType<typeof Bun.serve>[] = [];
const usageFailureExitCode = 64;

afterEach(async () => {
  for (const server of servers.splice(0)) {
    await server.stop(true);
  }
  cleanupFixtures();
});

function trackedServer(
  fetch: (request: Readonly<{ url: string }>) => Response | Promise<Response>,
): ReturnType<typeof Bun.serve> {
  const server = serve(fetch);
  servers.push(server);
  return server;
}

test.each([
  [[], "missing source URL"],
  [["https://example.test/file"], "missing SHA-256 checksum"],
  [["https://example.test/file", checksum], "missing destination path"],
])(
  "rejects missing command arguments",
  async (
    ...[commandArguments, diagnostic]: readonly [readonly string[], string]
  ) => {
    const installation = await runArguments(createFixture(), commandArguments);

    expect(installation.exitCode).toBe(1);
    expect(installation.stdout).toBe("");
    expect(installation.stderr).toContain(diagnostic);
  },
);

test("rejects an invalid checksum before download", async () => {
  const fixture = createFixture();
  let requests = 0;
  const server = trackedServer(() => {
    requests += 1;
    return new Response(content);
  });
  const installation = await runInstaller(fixture, url(server), "not-a-sha");

  expect(installation.exitCode).toBe(usageFailureExitCode);
  expect(installation.stdout).toBe("");
  expect(installation.stderr).toContain("Invalid SHA-256 checksum");
  expect(requests).toBe(0);
  expect(temporaryFiles(fixture)).toEqual([]);
});

test("removes temporary content after checksum mismatch", async () => {
  const fixture = createFixture();
  const server = trackedServer(() => new Response("corrupted\n"));
  const installation = await runInstaller(fixture, url(server), checksum);

  expect(installation.exitCode).toBe(1);
  expect(installation.stderr).toContain("SHA-256 mismatch");
  expect(Bun.file(fixture.destination).size).toBe(0);
  expect(temporaryFiles(fixture)).toEqual([]);
});

test("removes temporary content after a network failure", async () => {
  const fixture = createFixture();
  const server = trackedServer(
    () => new Response("unavailable", { status: 503 }),
  );
  const installation = await runInstaller(fixture, url(server), checksum);

  expect(installation.exitCode).not.toBe(0);
  expect(installation.stderr).toContain("Dictionary download failed");
  expect(Bun.file(fixture.destination).size).toBe(0);
  expect(temporaryFiles(fixture)).toEqual([]);
});

test("leaves an existing divergent file unchanged", async () => {
  const fixture = createFixture();
  mkdirSync(fixture.spelling, { recursive: true });
  writeFileSync(fixture.destination, "local dictionary\n");
  const server = trackedServer(() => new Response(content));
  const installation = await runInstaller(fixture, url(server), checksum);

  expect(installation.exitCode).toBe(1);
  expect(installation.stderr).toContain(
    "Refusing to replace existing dictionary",
  );
  expect(readFileSync(fixture.destination, "utf8")).toBe("local dictionary\n");
});

test("does not follow an existing destination symlink", async () => {
  const fixture = createFixture();
  mkdirSync(fixture.spelling, { recursive: true });
  const victim = join(fixture.root, "victim");
  writeFileSync(victim, "keep\n");
  symlinkSync(victim, fixture.destination);
  const server = trackedServer(() => new Response(content));
  const installation = await runInstaller(fixture, url(server), checksum);

  expect(installation.exitCode).toBe(1);
  expect(installation.stderr).toContain("non-regular dictionary destination");
  expect(readFileSync(victim, "utf8")).toBe("keep\n");
});

test("refuses an existing destination FIFO without waiting for a writer", async () => {
  const fixture = createFixture();
  mkdirSync(fixture.spelling, { recursive: true });
  expect(
    Bun.spawnSync([["mk", "fifo"].join(""), fixture.destination]).exitCode,
  ).toBe(0);
  const server = trackedServer(() => new Response(content));
  const installation = await runInstaller(fixture, url(server), checksum);

  expect(installation.exitCode).toBe(1);
  expect(installation.stderr).toContain("non-regular dictionary destination");
});

test.each(["Library", "Library/Spelling"])(
  "does not install through the %s directory symlink",
  async (symlinkPath) => {
    const fixture = createFixture();
    const target = join(fixture.root, "outside");
    mkdirSync(target);
    if (symlinkPath === "Library/Spelling") {
      mkdirSync(join(fixture.home, "Library"));
    }
    symlinkSync(target, join(fixture.home, symlinkPath));
    const server = trackedServer(() => new Response(content));
    const installation = await runInstaller(fixture, url(server), checksum);

    expect(installation.exitCode).toBe(1);
    expect(installation.stderr).toContain("non-regular dictionary directory");
    expect(Bun.file(join(target, "fr.aff")).size).toBe(0);
  },
);

test("does not install through a symlinked home directory", async () => {
  const fixture = createFixture();
  const actualHome = join(fixture.root, "actual-home");
  renameSync(fixture.home, actualHome);
  symlinkSync(actualHome, fixture.home);
  const server = trackedServer(() => new Response(content));
  const installation = await runInstaller(fixture, url(server), checksum);

  expect(installation.exitCode).toBe(1);
  expect(installation.stderr).toContain("non-regular home directory");
  expect(Bun.file(join(actualHome, "Library", "Spelling", "fr.aff")).size).toBe(
    0,
  );
});

test("rejects a destination outside the spelling directory", async () => {
  const fixture = createFixture();
  const outside = join(fixture.home, "fr.aff");
  const server = trackedServer(() => new Response(content));
  const installation = await runArguments(fixture, [
    url(server),
    checksum,
    outside,
  ]);

  expect(installation.exitCode).toBe(1);
  expect(installation.stderr).toContain("outside");
  expect(Bun.file(outside).size).toBe(0);
});

test("cleans up when publication loses to a directory", async () => {
  const fixture = createFixture();
  const server = trackedServer(() => {
    mkdirSync(fixture.destination);
    return new Response(content);
  });
  const installation = await runInstaller(fixture, url(server), checksum);

  expect(installation.exitCode).toBe(1);
  expect(installation.stderr).toContain("concurrent dictionary destination");
  expect(temporaryFiles(fixture)).toEqual([]);
});
