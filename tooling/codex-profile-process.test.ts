import { afterEach, expect, test } from "bun:test";
import { existsSync, mkdtempSync, rmSync, symlinkSync } from "node:fs";
import { Readable } from "node:stream";
import { createInterface } from "node:readline";
import { join } from "node:path";
import { tmpdir } from "node:os";
import { z } from "zod";

const directories: string[] = [];
const deadlineMilliseconds = 4000;
const entry = join(import.meta.dir, "codex-profile.ts");
const oversizedRepetitions = 200;
const readySchema = z.object({
  pid: z.number().int().positive(),
  relay: z.string(),
});
type Scenario = Readonly<{
  lifetime: "wait" | "resist";
  signal: "SIGINT" | "SIGTERM";
}>;

async function stopChild(
  kill: (signal: "SIGINT" | "SIGTERM") => void,
  scenario: Scenario,
  iterator: Readonly<AsyncIterator<string>>,
): Promise<void> {
  kill(scenario.signal);
  if (scenario.lifetime === "resist") {
    const witness = await iterator.next();
    expect(witness.value).toBe("resisted termination");
    kill(scenario.signal);
  }
}

async function readReady(
  iterator: Readonly<AsyncIterator<string>>,
): Promise<z.infer<typeof readySchema>> {
  const first = await iterator.next();
  return readySchema.parse(JSON.parse(z.string().parse(first.value)));
}

function removeFixtureChild(pid: number | null): void {
  if (pid === null) {
    return;
  }
  try {
    process.kill(pid, "SIGKILL");
  } catch (error) {
    expect(
      z.object({ code: z.literal("ESRCH") }).safeParse(error).success,
    ).toBe(true);
  }
}

afterEach(() => {
  for (const directory of directories.splice(0)) {
    rmSync(directory, { recursive: true, force: true });
  }
});

function fixture(): string {
  const directory = mkdtempSync(join(tmpdir(), "codex-profile-process-"));
  directories.push(directory);
  symlinkSync(
    join(import.meta.dir, "codex-profile-test-command.ts"),
    join(directory, "codex"),
  );
  return directory;
}

for (const scenario of [
  { lifetime: "wait", signal: "SIGINT" },
  { lifetime: "wait", signal: "SIGTERM" },
  { lifetime: "resist", signal: "SIGTERM" },
] as const) {
  test(`waits for the ${scenario.lifetime} child and cleans the relay after ${scenario.signal}`, async () => {
    const directory = fixture();
    const child = Bun.spawn([process.execPath, entry, "gpt-6.1-sol", "high"], {
      env: {
        ...process.env,
        PATH: `${directory}:${process.env.PATH ?? ""}`,
        CODEX_PROFILE_TEST_LIFETIME: scenario.lifetime,
      },
      stdout: "pipe",
      stderr: "pipe",
      stdin: "ignore",
    });
    const deadline = setTimeout(() => {
      child.kill("SIGKILL");
    }, deadlineMilliseconds);
    let nativePid: number | null = null;
    const lines = createInterface({ input: Readable.fromWeb(child.stdout) });
    const iterator = lines[Symbol.asyncIterator]();
    try {
      const ready = await readReady(iterator);
      nativePid = ready.pid;
      expect(existsSync(ready.relay)).toBe(true);
      await stopChild(
        (signal) => {
          child.kill(signal);
        },
        scenario,
        iterator,
      );
      expect(await child.exited).not.toBe(0);
      expect(() => process.kill(ready.pid, 0)).toThrow();
      expect(existsSync(ready.relay)).toBe(false);
      expect(existsSync(join(ready.relay, ".."))).toBe(false);
    } finally {
      clearTimeout(deadline);
      lines.close();
      child.kill("SIGKILL");
      removeFixtureChild(nativePid);
    }
  });
}

test("fails without launching or disclosing a prompt when Codex is missing", () => {
  const directory = mkdtempSync(join(tmpdir(), "codex-profile-missing-"));
  directories.push(directory);
  const result = Bun.spawnSync(
    [process.execPath, entry, "gpt-6.1-sol", "high", "private prompt"],
    { env: { ...process.env, PATH: directory } },
  );
  expect(result.exitCode).not.toBe(0);
  expect(result.stderr.toString()).toContain("launch failed");
  expect(result.stderr.toString()).not.toContain("private prompt");
});

for (const scenario of [
  {
    input: "private malformed secret",
    diagnostic: "diagnostic local illisible",
  },
  {
    input: "private secret".repeat(oversizedRepetitions),
    diagnostic: "diagnostic local trop volumineux",
  },
]) {
  test(`rejects ${scenario.diagnostic} without exposing raw input`, () => {
    const directory = fixture();
    const result = Bun.spawnSync(
      [process.execPath, entry, "gpt-6.1-sol", "high"],
      {
        env: {
          ...process.env,
          PATH: `${directory}:${process.env.PATH ?? ""}`,
          CODEX_PROFILE_TEST_RELAY_INPUT: scenario.input,
        },
      },
    );
    expect(result.exitCode).not.toBe(0);
    expect(result.stderr.toString()).toContain(scenario.diagnostic);
    expect(
      `${result.stdout.toString()}${result.stderr.toString()}`,
    ).not.toContain("secret");
  });
}
