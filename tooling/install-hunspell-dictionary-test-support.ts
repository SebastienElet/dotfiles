import { dirname, join } from "node:path";
import {
  mkdirSync,
  mkdtempSync,
  readdirSync,
  rmSync,
  symlinkSync,
} from "node:fs";
import { createHash } from "node:crypto";
import { tmpdir } from "node:os";

const entryPoint = join(import.meta.dir, "install-hunspell-dictionary");
const fixtures: Fixture[] = [];

type Fixture = Readonly<{
  root: string;
  home: string;
  spelling: string;
  destination: string;
  environment: Readonly<NodeJS.ProcessEnv>;
}>;

type RunResult = Readonly<{
  exitCode: number;
  stderr: string;
  stdout: string;
}>;

function createFixture(): Fixture {
  const root = mkdtempSync(join(tmpdir(), "hunspell-dictionary-"));
  const home = join(root, "home");
  const binaries = join(root, "bin");
  mkdirSync(home);
  mkdirSync(binaries);
  symlinkSync(process.execPath, join(binaries, "bun"));
  const fixture = {
    destination: join(home, "Library", "Spelling", "fr.aff"),
    environment: {
      ...process.env,
      HOME: home,
      PATH: `${binaries}:${process.env.PATH ?? ""}`,
    },
    home,
    root,
    spelling: join(home, "Library", "Spelling"),
  };
  fixtures.push(fixture);
  return fixture;
}

function runInstaller(
  fixture: Fixture,
  sourceUrl: string,
  checksum: string,
): Promise<RunResult> {
  return runArguments(fixture, [sourceUrl, checksum, fixture.destination]);
}

async function runArguments(
  fixture: Fixture,
  arguments_: readonly string[],
): Promise<RunResult> {
  const process = Bun.spawn([entryPoint, ...arguments_], {
    cwd: dirname(import.meta.dir),
    env: fixture.environment,
    stderr: "pipe",
    stdout: "pipe",
  });
  const [exitCode, stdout, stderr] = await Promise.all([
    process.exited,
    new Response(process.stdout).text(),
    new Response(process.stderr).text(),
  ]);
  return { exitCode, stderr, stdout };
}

function serve(
  fetch: (request: Readonly<{ url: string }>) => Response | Promise<Response>,
): ReturnType<typeof Bun.serve> {
  return Bun.serve({ fetch, hostname: "127.0.0.1", port: 0 });
}

function url(
  server: Readonly<{ port: number | undefined }>,
  path = "/file",
): string {
  if (server.port === undefined) {
    throw new Error("test server has no assigned port");
  }
  return `http://127.0.0.1:${server.port}${path}`;
}

function sha256(content: string | Readonly<ArrayLike<number>>): string {
  const hashContent =
    typeof content === "string" ? content : Uint8Array.from(content);
  return createHash("sha256").update(hashContent).digest("hex");
}

function temporaryFiles(fixture: Fixture): string[] {
  try {
    return readdirSync(fixture.spelling).filter((name) =>
      name.startsWith(".hunspell-dictionary."),
    );
  } catch {
    return [];
  }
}

function cleanupFixtures(): void {
  for (const fixture of fixtures.splice(0)) {
    rmSync(fixture.root, { force: true, recursive: true });
  }
}

export {
  cleanupFixtures,
  createFixture,
  runArguments,
  runInstaller,
  serve,
  sha256,
  temporaryFiles,
  url,
};
export type { Fixture, RunResult };
