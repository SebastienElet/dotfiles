import { join } from "node:path";
import { mkdtempSync } from "node:fs";
import { tmpdir } from "node:os";

const bunExecutable = resolveBunExecutable();
const legacyEntryPoint = join(
  import.meta.dir,
  "agent-handoff-legacy",
  "agent-handoff",
);
const rustEntryPoint = join(
  import.meta.dir,
  "agent-handoff",
  "target",
  "debug",
  "agent-handoff",
);

function resolveBunExecutable(): string {
  const executable = Bun.which("bun");
  if (executable === null) {
    throw new Error("missing Bun executable");
  }
  return executable;
}

type Fixture = Readonly<{
  home: string;
  root: string;
  transcriptPath: string;
  xdgStateHome: string;
}>;

type InputBytes = Readonly<{
  readonly [index: number]: number;
  length: number;
}>;

type HookResult = Readonly<{
  exitCode: number;
  stdout: Uint8Array;
  stderr: Uint8Array;
}>;

const commandArgumentsByFixture = new WeakMap<Fixture, readonly string[]>();

function createParityFixture(): Fixture {
  const root = mkdtempSync(join(tmpdir(), "agent-handoff-parity-"));
  const fixture = {
    home: join(root, "home"),
    root,
    transcriptPath: join(root, "transcript.jsonl"),
    xdgStateHome: join(root, "state"),
  };
  commandArgumentsByFixture.set(fixture, []);
  return fixture;
}

function setFixtureCommandArguments(
  fixture: Fixture,
  arguments_: readonly string[],
): void {
  commandArgumentsByFixture.set(fixture, [...arguments_]);
}

async function runProcess(
  command: readonly string[],
  input: InputBytes,
  environment: Readonly<Record<string, string>>,
): Promise<HookResult> {
  const child = Bun.spawn([...command], {
    env: environment,
    stderr: "pipe",
    stdin: new Blob([Uint8Array.from(input)]),
    stdout: "pipe",
  });
  const [exitCode, stdout, stderr] = await Promise.all([
    child.exited,
    new Response(child.stdout).arrayBuffer(),
    new Response(child.stderr).arrayBuffer(),
  ]);
  return {
    exitCode,
    stdout: new Uint8Array(stdout),
    stderr: new Uint8Array(stderr),
  };
}

function runLegacy(
  input: InputBytes,
  environment: Readonly<Record<string, string>>,
  fixture: Fixture,
): Promise<HookResult> {
  return runProcess(
    [
      bunExecutable,
      legacyEntryPoint,
      ...(commandArgumentsByFixture.get(fixture) ?? []),
    ],
    input,
    environment,
  );
}

function runRust(
  input: InputBytes,
  environment: Readonly<Record<string, string>>,
  fixture: Fixture,
): Promise<HookResult> {
  return runProcess(
    [rustEntryPoint, ...(commandArgumentsByFixture.get(fixture) ?? [])],
    input,
    environment,
  );
}

export { createParityFixture, runLegacy, runRust, setFixtureCommandArguments };
export type { Fixture, HookResult };
