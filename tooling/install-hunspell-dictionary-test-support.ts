import { createHash } from "node:crypto";
import {
  mkdirSync,
  mkdtempSync,
  readdirSync,
  rmSync,
  symlinkSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";

const entryPoint = join(import.meta.dir, "install-hunspell-dictionary");
const fixtures: Fixture[] = [];

export type Fixture = Readonly<{
  root: string;
  home: string;
  spelling: string;
  destination: string;
  environment: NodeJS.ProcessEnv;
}>;

export function createFixture(): Fixture {
  const root = mkdtempSync(join(tmpdir(), "hunspell-dictionary-"));
  const home = join(root, "home");
  const binaries = join(root, "bin");
  mkdirSync(home);
  mkdirSync(binaries);
  symlinkSync(process.execPath, join(binaries, "bun"));
  const fixture = {
    root,
    home,
    spelling: join(home, "Library", "Spelling"),
    destination: join(home, "Library", "Spelling", "fr.aff"),
    environment: {
      ...process.env,
      HOME: home,
      PATH: `${binaries}:${process.env.PATH ?? ""}`,
    },
  };
  fixtures.push(fixture);
  return fixture;
}

export async function runInstaller(
  fixture: Fixture,
  url: string,
  checksum: string,
  destination = fixture.destination,
) {
  return runArguments(fixture, [url, checksum, destination]);
}

export async function runArguments(
  fixture: Fixture,
  arguments_: readonly string[],
) {
  const process = Bun.spawn([entryPoint, ...arguments_], {
    cwd: dirname(import.meta.dir),
    env: fixture.environment,
    stdout: "pipe",
    stderr: "pipe",
  });
  const [exitCode, stdout, stderr] = await Promise.all([
    process.exited,
    new Response(process.stdout).text(),
    new Response(process.stderr).text(),
  ]);
  return { exitCode, stdout, stderr };
}

export function serve(
  fetch: (request: Request) => Response | Promise<Response>,
) {
  return Bun.serve({ hostname: "127.0.0.1", port: 0, fetch });
}

export function url(server: ReturnType<typeof Bun.serve>, path = "/file") {
  return `http://127.0.0.1:${server.port}${path}`;
}

export function sha256(content: string | Uint8Array): string {
  return createHash("sha256").update(content).digest("hex");
}

export function temporaryFiles(fixture: Fixture): string[] {
  try {
    return readdirSync(fixture.spelling).filter((name) =>
      name.startsWith(".hunspell-dictionary."),
    );
  } catch {
    return [];
  }
}

export function cleanupFixtures(): void {
  for (const fixture of fixtures.splice(0)) {
    rmSync(fixture.root, { force: true, recursive: true });
  }
}
