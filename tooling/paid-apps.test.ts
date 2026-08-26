import { afterEach, expect, test } from "bun:test";
import {
  existsSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  rmSync,
  symlinkSync,
  writeFileSync,
} from "node:fs";
import { join } from "node:path";
import { tmpdir } from "node:os";

const makefile = join(import.meta.dir, "..", "Makefile");
const provider = join(import.meta.dir, "paid-apps-test-provider.ts");
const fixtures: string[] = [];

type PaidApp = Readonly<{
  bundle: string;
  id: string;
  target: string;
}>;

const paidApps: PaidApp[] = [
  { bundle: "Flow.app", id: "1423210932", target: "flow" },
  { bundle: "Things3.app", id: "904280696", target: "things-3" },
  { bundle: "DaisyDisk.app", id: "411643860", target: "daisydisk" },
] satisfies PaidApp[];

type Fixture = Readonly<{
  apps: string;
  destination: string;
  environment: Readonly<NodeJS.ProcessEnv>;
  root: string;
  trace: string;
}>;

type MakeResult = Readonly<{
  exitCode: number;
  stderr: string;
  stdout: string;
}>;

afterEach(() => {
  for (const root of fixtures.splice(0)) {
    rmSync(root, { force: true, recursive: true });
  }
});

function createFixture(bundle: string): Fixture {
  const root = mkdtempSync(join(tmpdir(), "dotfiles-paid-apps-"));
  const apps = join(root, "Applications");
  const bin = join(root, "bin");
  const brewBin = join(root, "homebrew", "bin");
  const voltaBin = join(root, "volta", "bin");
  const trace = join(root, "provider-trace");
  fixtures.push(root);
  mkdirSync(apps);
  mkdirSync(bin);
  mkdirSync(brewBin, { recursive: true });
  mkdirSync(voltaBin, { recursive: true });
  symlinkSync("/usr/bin/true", join(bin, "brew"));
  writeFileSync(join(brewBin, "brew"), "");
  writeFileSync(join(brewBin, "mas"), "");
  writeFileSync(join(brewBin, "volta"), "");
  writeFileSync(join(voltaBin, "node"), "");
  writeFileSync(join(voltaBin, "thangs"), "");
  symlinkSync("/usr/bin/true", join(voltaBin, "npm"));
  return {
    apps,
    destination: join(apps, bundle),
    environment: {
      ...process.env,
      PATH: `${bin}:/usr/bin:/bin`,
      PAID_APPS_TEST_DESTINATION: join(apps, bundle),
      PAID_APPS_TEST_TRACE: trace,
    },
    root,
    trace,
  };
}

async function runTarget(
  fixture: Fixture,
  target: string,
  skipPaidApps: boolean,
): Promise<MakeResult> {
  const child = Bun.spawn(
    [
      "make",
      "--no-print-directory",
      "-f",
      makefile,
      target,
      `APP_BIN=${fixture.apps}`,
      `BREW_BIN=${join(fixture.root, "homebrew", "bin")}`,
      `VOLTA_BIN=${join(fixture.root, "volta", "bin")}`,
      `MAS=${process.execPath} ${provider}`,
      "HAS_BREW_TRUST=no",
      `SKIP_PAID_APPS=${skipPaidApps ? "1" : "0"}`,
    ],
    { env: fixture.environment, stderr: "pipe", stdout: "pipe" },
  );
  const [exitCode, stdout, stderr] = await Promise.all([
    child.exited,
    new Response(child.stdout).text(),
    new Response(child.stderr).text(),
  ]);
  return { exitCode, stderr, stdout };
}

function readTrace(fixture: Fixture): string {
  try {
    return readFileSync(fixture.trace, "utf8");
  } catch (error) {
    if (error instanceof Error && "code" in error && error.code === "ENOENT") {
      return "";
    }
    throw error;
  }
}

test("the installation provider fails closed without its destination", async () => {
  const child = Bun.spawn(
    [process.execPath, provider, "install", "1423210932"],
    {
      env: {},
      stderr: "pipe",
      stdout: "pipe",
    },
  );
  const [exitCode, stderr] = await Promise.all([
    child.exited,
    new Response(child.stderr).text(),
  ]);

  expect(exitCode).not.toBe(0);
  expect(stderr).toContain("PAID_APPS_TEST_DESTINATION");
});

test.each(paidApps)(
  "$target: initial and repeated skips do not create $bundle",
  async ({ bundle, target }) => {
    const fixture = createFixture(bundle);

    const first = await runTarget(fixture, target, true);
    const second = await runTarget(fixture, target, true);

    expect(first).toMatchObject({ exitCode: 0, stderr: "" });
    expect(second).toMatchObject({ exitCode: 0, stderr: "" });
    expect(first.stdout).toContain("skipped (SKIP_PAID_APPS=1)");
    expect(second.stdout).toContain("skipped (SKIP_PAID_APPS=1)");
    expect(existsSync(fixture.destination)).toBe(false);
    expect(readTrace(fixture)).toBe("");
  },
);

test.each(paidApps)(
  "$target: a run after skip installs missing $bundle exactly once",
  async ({ bundle, id, target }) => {
    const fixture = createFixture(bundle);

    const skipped = await runTarget(fixture, target, true);
    const installed = await runTarget(fixture, target, false);
    const converged = await runTarget(fixture, target, false);

    expect(skipped).toMatchObject({ exitCode: 0, stderr: "" });
    expect(installed).toMatchObject({ exitCode: 0, stderr: "" });
    expect(converged).toMatchObject({ exitCode: 0, stderr: "" });

    expect(existsSync(fixture.destination)).toBe(true);
    expect(readTrace(fixture)).toBe(`install ${id}\n`);
  },
);

test.each(paidApps)(
  "$target: an existing $bundle remains intact and converged",
  async ({ bundle, target }) => {
    const fixture = createFixture(bundle);
    const marker = join(fixture.destination, "existing-data");
    mkdirSync(fixture.destination);
    writeFileSync(marker, "preserve");

    const result = await runTarget(fixture, target, false);

    expect(result).toMatchObject({ exitCode: 0, stderr: "" });
    expect(readFileSync(marker, "utf8")).toBe("preserve");
    expect(readTrace(fixture)).toBe("");
  },
);
