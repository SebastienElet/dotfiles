import { afterEach, expect, test } from "bun:test";
import {
  cleanupFixtures,
  createFixture,
  runInstaller,
  serve,
  sha256,
  temporaryFiles,
  url,
} from "./install-hunspell-dictionary-test-support.ts";
import {
  lstatSync,
  mkdirSync,
  readFileSync,
  statSync,
  writeFileSync,
} from "node:fs";

const content = "SET UTF-8\nTRY abcdef\n";
const checksum = sha256(content);
const servers: ReturnType<typeof Bun.serve>[] = [];
const fileModeMask = 0o777;
const publishedFileMode = 0o644;
const privateFileMode = 0o600;
const concurrentRequestCount = 2;

afterEach(async () => {
  for (const server of servers.splice(0)) {
    await server.stop(true);
  }
  cleanupFixtures();
});

function contentServer(onRequest?: () => void): ReturnType<typeof Bun.serve> {
  const server = serve(() => {
    onRequest?.();
    return new Response(content);
  });
  servers.push(server);
  return server;
}

test("publishes verified content with the expected mode", async () => {
  const fixture = createFixture();
  const server = contentServer();
  const installation = await runInstaller(fixture, url(server), checksum);

  expect(installation).toEqual({ exitCode: 0, stderr: "", stdout: "" });
  expect(readFileSync(fixture.destination, "utf8")).toBe(content);
  expect(statSync(fixture.destination).mode & fileModeMask).toBe(
    publishedFileMode,
  );
  expect(temporaryFiles(fixture)).toEqual([]);
});

test("leaves an existing correct file unchanged without downloading", async () => {
  const fixture = createFixture();
  mkdirSync(fixture.spelling, { recursive: true });
  writeFileSync(fixture.destination, content, { mode: privateFileMode });
  const before = statSync(fixture.destination);
  let requests = 0;
  const server = contentServer(() => {
    requests += 1;
  });
  const replay = await runInstaller(fixture, url(server), checksum);
  const after = statSync(fixture.destination);

  expect(replay).toEqual({ exitCode: 0, stderr: "", stdout: "" });
  expect(requests).toBe(0);
  expect({
    inode: after.ino,
    mode: after.mode,
    mtime: after.mtimeMs,
  }).toEqual({
    inode: before.ino,
    mode: before.mode,
    mtime: before.mtimeMs,
  });
});

test("accepts concurrent publication of the same verified content", async () => {
  const fixture = createFixture();
  const responsesReleased = Promise.withResolvers<undefined>();
  let requests = 0;
  const bothRequested = Promise.withResolvers<undefined>();
  const server = serve(async () => {
    requests += 1;
    if (requests === concurrentRequestCount) {
      bothRequested.resolve();
    }
    await responsesReleased.promise;
    return new Response(content);
  });
  servers.push(server);

  const first = runInstaller(fixture, url(server), checksum);
  const second = runInstaller(fixture, url(server), checksum);
  await bothRequested.promise;
  responsesReleased.resolve();
  const results = await Promise.all([first, second]);

  expect(results.map(({ exitCode }) => exitCode)).toEqual([0, 0]);
  expect(readFileSync(fixture.destination, "utf8")).toBe(content);
  expect(temporaryFiles(fixture)).toEqual([]);
});

test("refuses concurrent verified content that loses publication", async () => {
  const fixture = createFixture();
  mkdirSync(fixture.spelling, { recursive: true });
  const firstContent = "first dictionary\n";
  const secondContent = "second dictionary\n";
  const responsesReleased = Promise.withResolvers<undefined>();
  let requests = 0;
  const bothRequested = Promise.withResolvers<undefined>();
  const server = serve(async (request) => {
    requests += 1;
    if (requests === concurrentRequestCount) {
      bothRequested.resolve();
    }
    await responsesReleased.promise;
    return new Response(
      request.url.endsWith("/first") ? firstContent : secondContent,
    );
  });
  servers.push(server);

  const first = runInstaller(
    fixture,
    url(server, "/first"),
    sha256(firstContent),
  );
  const second = runInstaller(
    fixture,
    url(server, "/second"),
    sha256(secondContent),
  );
  await bothRequested.promise;
  responsesReleased.resolve();
  const results = await Promise.all([first, second]);

  expect(
    results
      .map(({ exitCode }) => exitCode)
      .toSorted((left, right) => left - right),
  ).toEqual([0, 1]);
  expect([firstContent, secondContent]).toContain(
    readFileSync(fixture.destination, "utf8"),
  );
  expect(temporaryFiles(fixture)).toEqual([]);
});

test("does not expose a partial destination while downloading", async () => {
  const fixture = createFixture();
  const requested = Promise.withResolvers<undefined>();
  const response = Promise.withResolvers<undefined>();
  const server = serve(async () => {
    requested.resolve();
    await response.promise;
    return new Response(content);
  });
  servers.push(server);

  const installation = runInstaller(fixture, url(server), checksum);
  await requested.promise;
  expect(() => lstatSync(fixture.destination)).toThrow();
  response.resolve();

  const installationResult = await installation;
  expect(installationResult.exitCode).toBe(0);
  expect(readFileSync(fixture.destination, "utf8")).toBe(content);
});
