import { afterEach, expect, test } from "bun:test";
import {
  appendFileSync,
  existsSync,
  mkdtempSync,
  readFileSync,
  rmSync,
  symlinkSync,
  writeFileSync,
} from "node:fs";
import { join } from "node:path";
import { tmpdir } from "node:os";
import { z } from "zod";

const fixtures: string[] = [];
const entry = join(import.meta.dir, "codex-profile.ts");
const agentExitCode = 17;

afterEach(() => {
  for (const directory of fixtures.splice(0)) {
    rmSync(directory, { recursive: true, force: true });
  }
});

function transcript(): string {
  const directory = mkdtempSync(join(tmpdir(), "codex-profile-"));
  fixtures.push(directory);
  const path = join(directory, "rollout.jsonl");
  writeFileSync(
    path,
    `${JSON.stringify({
      type: "turn_context",
      payload: { turn_id: "turn-1", model: "gpt-6.1-sol", effort: "high" },
    })}\n`,
  );
  return path;
}

function hook(payload: unknown): Bun.SyncSubprocess<"pipe", "pipe"> {
  return Bun.spawnSync(
    [process.execPath, entry, "hook", "gpt-6.1-sol", "high"],
    { stdin: Buffer.from(JSON.stringify(payload)) },
  );
}

test("reports a matching native turn as conforme", () => {
  const result = hook({
    hook_event_name: "PreToolUse",
    turn_id: "turn-1",
    model: "gpt-6.1-sol",
    transcript_path: transcript(),
  });
  expect(result.exitCode).toBe(0);
  expect(result.stdout.toString()).toContain("conforme");
  expect(result.stdout.toString()).toContain("gpt-6.1-sol/high");
});

test("launches with an explicit requested profile and preserves caller overrides and status", () => {
  const directory = mkdtempSync(join(tmpdir(), "codex-profile-launch-"));
  fixtures.push(directory);
  symlinkSync(
    join(import.meta.dir, "codex-profile-test-command.ts"),
    join(directory, "codex"),
  );
  const result = Bun.spawnSync(
    [
      process.execPath,
      entry,
      "gpt-6.1-sol",
      "high",
      "exec",
      "--model",
      "other-model",
      "a prompt with spaces",
    ],
    { env: { ...process.env, PATH: `${directory}:${process.env.PATH ?? ""}` } },
  );
  expect(result.exitCode).toBe(agentExitCode);
  expect(result.stdout.toString()).toContain(
    String.raw`"model=\"gpt-6.1-sol\""`,
  );
  expect(result.stdout.toString()).toContain(
    String.raw`"model_reasoning_effort=\"high\""`,
  );
  expect(result.stdout.toString()).toContain(
    '"exec","--model","other-model","a prompt with spaces"',
  );
  const report = z
    .object({ relay: z.string() })
    .parse(JSON.parse(result.stdout.toString()));
  expect(existsSync(report.relay)).toBe(false);
  expect(existsSync(join(report.relay, ".."))).toBe(false);
});

test("warns on an observed model mismatch even when effort is unavailable", () => {
  const result = hook({
    hook_event_name: "SessionStart",
    model: "other-model",
  });
  expect(result.exitCode).toBe(0);
  expect(result.stdout.toString()).toContain("AVERTISSEMENT — différent");
  expect(result.stdout.toString()).toContain("observé other-model/inconnu");
});

test("observes a voluntary effort change on the next turn without forcing it back", () => {
  const path = transcript();
  appendFileSync(
    path,
    `${JSON.stringify({
      type: "turn_context",
      payload: { turn_id: "turn-2", model: "gpt-6.1-sol", effort: "low" },
    })}\n`,
  );
  const result = hook({
    hook_event_name: "Stop",
    turn_id: "turn-2",
    model: "gpt-6.1-sol",
    transcript_path: path,
  });
  expect(result.exitCode).toBe(0);
  expect(result.stdout.toString()).toContain("AVERTISSEMENT — différent");
  expect(result.stdout.toString()).toContain("observé gpt-6.1-sol/low");
  expect(readFileSync(path, "utf8")).toContain('"effort":"low"');
});

test("does not compare a specialized subagent with the root profile", () => {
  const result = hook({
    hook_event_name: "PreToolUse",
    model: "other-model",
    agent_id: "child",
    agent_type: "design-claim-auditor",
    turn_id: "turn-1",
    transcript_path: transcript(),
  });
  expect(result.exitCode).toBe(0);
  expect(result.stdout.toString()).toContain("inconnu (profil spécialisé");
  expect(result.stdout.toString()).not.toContain("AVERTISSEMENT");
});

test("never uses a previous turn or an old resumed context as current effort evidence", () => {
  const path = transcript();
  for (const event of [
    { hook_event_name: "SessionStart", turn_id: "turn-1" },
    { hook_event_name: "PreToolUse", turn_id: "turn-2" },
  ]) {
    const result = hook({
      ...event,
      model: "gpt-6.1-sol",
      transcript_path: path,
    });
    expect(result.exitCode).toBe(0);
    expect(result.stdout.toString()).toContain("inconnu");
    expect(result.stdout.toString()).not.toContain("conforme");
  }
});

test("reports missing, unreadable and malformed native observations without disclosing content", () => {
  const path = transcript();
  writeFileSync(path, "private conversation: secret\n");
  for (const transcriptPath of [
    undefined,
    `${path}.missing`,
    path,
    join(path, ".."),
  ]) {
    const result = hook({
      hook_event_name: "Stop",
      turn_id: "turn-1",
      model: "gpt-6.1-sol",
      transcript_path: transcriptPath,
      last_assistant_message: "private response: secret",
    });
    expect(result.exitCode).toBe(0);
    expect(result.stdout.toString()).toContain("inconnu");
    expect(
      `${result.stdout.toString()}${result.stderr.toString()}`,
    ).not.toContain("secret");
  }
});

test("does not reuse known effort when the newest context omits it or is incomplete", () => {
  for (const suffix of [
    `${JSON.stringify({ type: "turn_context", payload: { turn_id: "turn-1", model: "gpt-6.1-sol" } })}\n`,
    '{"type":"turn_context","payload":',
  ]) {
    const path = transcript();
    appendFileSync(path, suffix);
    const result = hook({
      hook_event_name: "Stop",
      turn_id: "turn-1",
      model: "gpt-6.1-sol",
      transcript_path: path,
    });
    expect(result.exitCode).toBe(0);
    expect(result.stdout.toString()).toContain("observé gpt-6.1-sol/inconnu");
  }
});

test("handles malformed and oversized hook input with an unknown diagnostic", () => {
  const maximumInputBytes = 1_048_576;
  for (const input of [
    "private malformed secret",
    "x".repeat(maximumInputBytes + 1),
  ]) {
    const result = Bun.spawnSync(
      [process.execPath, entry, "hook", "gpt-6.1-sol", "high"],
      { stdin: Buffer.from(input) },
    );
    expect(result.exitCode).toBe(0);
    expect(result.stdout.toString()).toContain("inconnu");
    expect(
      `${result.stdout.toString()}${result.stderr.toString()}`,
    ).not.toContain("secret");
  }
});

test("surfaces a native mismatch on launcher stderr even when Codex discards hook output", () => {
  const directory = mkdtempSync(join(tmpdir(), "codex-profile-native-"));
  fixtures.push(directory);
  symlinkSync(
    join(import.meta.dir, "codex-profile-test-command.ts"),
    join(directory, "codex"),
  );
  const result = Bun.spawnSync(
    [process.execPath, entry, "gpt-6.1-sol", "high", "exec"],
    {
      env: {
        ...process.env,
        PATH: `${directory}:${process.env.PATH ?? ""}`,
        CODEX_PROFILE_TEST_NATIVE: JSON.stringify({
          hook_event_name: "Stop",
          model: "other-model",
        }),
      },
    },
  );
  expect(result.exitCode).toBe(agentExitCode);
  expect(result.stderr.toString()).toContain("AVERTISSEMENT — différent");
});

test("reads a recent context after a large native record with multibyte content", () => {
  const repeatedCharacters = 600_000;
  for (const padding of ["", " "]) {
    const path = transcript();
    const context = readFileSync(path, "utf8");
    writeFileSync(
      path,
      `${JSON.stringify({ type: "response_item", payload: "é".repeat(repeatedCharacters) })}\n${context.trimEnd()}${padding}\n`,
    );
    const result = hook({
      hook_event_name: "Stop",
      turn_id: "turn-1",
      model: "gpt-6.1-sol",
      transcript_path: path,
    });
    expect(result.exitCode).toBe(0);
    expect(result.stdout.toString()).toContain("conforme");
  }
});

test("reports unavailable delivery instead of claiming compliance when the relay is missing", () => {
  const path = transcript();
  const result = Bun.spawnSync(
    [process.execPath, entry, "hook", "gpt-6.1-sol", "high"],
    {
      env: { ...process.env, DOTFILES_CODEX_PROFILE_RELAY: `${path}.sock` },
      stdin: Buffer.from(
        JSON.stringify({
          hook_event_name: "Stop",
          model: "gpt-6.1-sol",
          turn_id: "turn-1",
          transcript_path: path,
        }),
      ),
    },
  );
  expect(result.exitCode).not.toBe(0);
  expect(result.stdout.toString()).toContain(
    "inconnu (relais local indisponible)",
  );
  expect(result.stdout.toString()).not.toContain("conforme");
});
