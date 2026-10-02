import { afterEach, expect, test } from "bun:test";
import {
  mkdirSync,
  mkdtempSync,
  readFileSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { join } from "node:path";
import { tmpdir } from "node:os";

const fixtures: string[] = [];
const npmFailureStatus = 9;
type PaidApp = Readonly<{ bundle: string; target: string }>;
const paidApps: PaidApp[] = [{ bundle: "DaisyDisk.app", target: "daisydisk" }];

afterEach(() => {
  for (const root of fixtures.splice(0)) {
    rmSync(root, { force: true, recursive: true });
  }
});

test.each(paidApps)(
  "$target: an explicit paid-app skip succeeds without $bundle",
  ({ bundle }) => {
    const fixture = createFixture();

    const result = runTarget(fixture, true);

    expect(result).toEqual({ exitCode: 0, stderr: "", stdout: "" });
    expect(() => readFileSync(join(fixture.apps, bundle))).toThrow();
  },
);

test.each(paidApps)(
  "$target: fails when Bundle succeeds without producing $bundle",
  () => {
    const fixture = createFixture();

    const result = runTarget(fixture, false);

    expect(result.exitCode).not.toBe(0);
    expect(result.stderr).toContain("Homebrew Bundle did not install");
  },
);

test.each(paidApps)(
  "$target: accepts an existing $bundle without changing it",
  ({ bundle }) => {
    const fixture = createFixture();
    const marker = join(fixture.apps, bundle, "existing-data");
    mkdirSync(join(fixture.apps, bundle));
    writeFileSync(marker, "preserve");

    const result = runTarget(fixture, false);
    const replay = runTarget(fixture, false);

    expect(replay).toEqual({ exitCode: 0, stderr: "", stdout: "" });
    expect(result).toEqual({ exitCode: 0, stderr: "", stdout: "" });
    expect(readFileSync(marker, "utf8")).toBe("preserve");
  },
);

function createFixture(): Readonly<{ apps: string; root: string }> {
  const root = mkdtempSync(join(tmpdir(), "dotfiles-paid-apps-"));
  const apps = join(root, "Applications");
  fixtures.push(root);
  mkdirSync(apps);
  return { apps, root };
}

function runTarget(
  fixture: ReturnType<typeof createFixture>,
  skipPaidApps: boolean,
): Readonly<{ exitCode: number; stderr: string; stdout: string }> {
  return runPaidAppCommand(
    fixture,
    ["verify-app", join(fixture.apps, "DaisyDisk.app")],
    { SKIP_PAID_APPS: skipPaidApps ? "1" : "0" },
  );
}

function runPaidAppCommand(
  fixture: ReturnType<typeof createFixture>,
  command: readonly string[],
  environment: Readonly<NodeJS.ProcessEnv> = {},
): Readonly<{ exitCode: number; stderr: string; stdout: string }> {
  const result = Bun.spawnSync(
    [
      process.execPath,
      "--config=/dev/null",
      "--no-env-file",
      join(import.meta.dir, "paid-apps-install.ts"),
      ...command,
    ],
    {
      cwd: fixture.root,
      env: { HOME: fixture.root, ...environment },
      stderr: "pipe",
      stdout: "pipe",
    },
  );
  return {
    exitCode: result.exitCode,
    stderr: result.stderr.toString(),
    stdout: result.stdout.toString(),
  };
}

test("things-3: paid skip avoids checking the application", () => {
  const fixture = createFixture();
  expect(
    runPaidAppCommand(
      fixture,
      ["verify-app", join(fixture.apps, "Things3.app")],
      { SKIP_PAID_APPS: "1" },
    ),
  ).toEqual({ exitCode: 0, stderr: "", stdout: "" });
});

test("things-3: rejects a missing application", () => {
  const fixture = createFixture();
  const result = runPaidAppCommand(fixture, [
    "verify-app",
    join(fixture.apps, "Things3.app"),
  ]);
  expect(result.exitCode).not.toBe(0);
  expect(result.stderr).toContain("Homebrew Bundle did not install");
});

test("things-3: accepts an existing application and preserves it on replay", () => {
  const fixture = createFixture();
  const application = join(fixture.apps, "Things3.app");
  mkdirSync(application);
  const marker = join(application, "existing-data");
  writeFileSync(marker, "preserve");
  for (const result of [
    runPaidAppCommand(fixture, ["verify-app", application]),
    runPaidAppCommand(fixture, ["verify-app", application]),
  ]) {
    expect(result).toEqual({ exitCode: 0, stderr: "", stdout: "" });
  }
  expect(readFileSync(marker, "utf8")).toBe("preserve");
});

test("things-3: a regular file cannot satisfy the application check", () => {
  const fixture = createFixture();
  const application = join(fixture.apps, "Things3.app");
  writeFileSync(application, "personal");
  expect(
    runPaidAppCommand(fixture, ["verify-app", application]).exitCode,
  ).not.toBe(0);
  expect(readFileSync(application, "utf8")).toBe("personal");
});

test("things-3: paid skip avoids npm even when the CLI is missing", () => {
  const fixture = createFixture();
  expect(
    runPaidAppCommand(fixture, ["install-cli"], {
      SKIP_PAID_APPS: "1",
      THINGS3_PAID_CONTEXT: "1",
    }),
  ).toEqual({ exitCode: 0, stderr: "", stdout: "" });
});

test("things3-cli-wrapper: standalone installation ignores the paid skip", () => {
  const fixture = createFixture();
  const npm = join(fixture.root, ".volta", "bin", "npm");
  mkdirSync(join(fixture.root, ".volta", "bin"), { recursive: true });
  writeFileSync(npm, '#!/bin/sh\nprintf "%s\\n" "$@"\n', { mode: 0o755 });
  const result = runPaidAppCommand(fixture, ["install-cli"], {
    SKIP_PAID_APPS: "1",
  });
  expect(result).toEqual({
    exitCode: 0,
    stderr: "",
    stdout: "install\n-g\n@dougskinner/thangs\n",
  });
});

test("things3-cli-wrapper: propagates the npm failure status", () => {
  const fixture = createFixture();
  const npm = join(fixture.root, ".volta", "bin", "npm");
  mkdirSync(join(fixture.root, ".volta", "bin"), { recursive: true });
  writeFileSync(npm, `#!/bin/sh\nexit ${npmFailureStatus}\n`, { mode: 0o755 });
  expect(runPaidAppCommand(fixture, ["install-cli"]).exitCode).toBe(
    npmFailureStatus,
  );
});

test("things3-cli-wrapper: rejects an unavailable npm executable", () => {
  const fixture = createFixture();
  expect(runPaidAppCommand(fixture, ["install-cli"]).exitCode).not.toBe(0);
});

test.each([
  { args: ["verify-app"] },
  { args: ["install-cli", "unexpected"] },
  { args: ["unknown"] },
])(
  "things-3: rejects malformed invocation $args",
  ({ args }: Readonly<{ args: readonly string[] }>) => {
    const fixture = createFixture();
    expect(runPaidAppCommand(fixture, args).exitCode).not.toBe(0);
  },
);

test.each(["", "true", "yes", "0"])(
  "things-3: only the explicit skip value 1 disables installation (%j)",
  (skip) => {
    const fixture = createFixture();
    expect(
      runPaidAppCommand(fixture, ["install-cli"], {
        SKIP_PAID_APPS: skip,
        THINGS3_PAID_CONTEXT: "1",
      }).exitCode,
    ).not.toBe(0);
    expect(
      runPaidAppCommand(
        fixture,
        ["verify-app", join(fixture.apps, "Things3.app")],
        {
          SKIP_PAID_APPS: skip,
        },
      ).exitCode,
    ).not.toBe(0);
  },
);

test("things3-cli-wrapper: rejects a missing home directory at the boundary", () => {
  const fixture = createFixture();
  const result = runPaidAppCommand(fixture, ["install-cli"], { HOME: "" });
  expect(result.exitCode).not.toBe(0);
  expect(result.stderr).toContain("HOME");
});

test("daisydisk: rejects and preserves a regular file at the application path", () => {
  const fixture = createFixture();
  const application = join(fixture.apps, "DaisyDisk.app");
  writeFileSync(application, "preserve");
  expect(runTarget(fixture, false).exitCode).not.toBe(0);
  expect(readFileSync(application, "utf8")).toBe("preserve");
});

test.each(["", "true", "yes", "0"])(
  "daisydisk: only the explicit skip value 1 disables verification (%j)",
  (skip) => {
    const fixture = createFixture();
    expect(
      runPaidAppCommand(
        fixture,
        ["verify-app", join(fixture.apps, "DaisyDisk.app")],
        {
          SKIP_PAID_APPS: skip,
        },
      ).exitCode,
    ).not.toBe(0);
  },
);
