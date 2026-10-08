import { afterEach, expect, test } from "bun:test";
import { chmodSync, mkdtempSync, rmSync, writeFileSync } from "node:fs";
import type { IssueFixture } from "./dispatch-test-support.ts";
import type { NativePane } from "./native-state.ts";
import { createBindingStore } from "./binding-store.ts";
import { fileURLToPath } from "node:url";
import { issueFixture } from "./dispatch-test-support.ts";
import { join } from "node:path";
import { tmpdir } from "node:os";
import { workingIssueState } from "./session-test-support.ts";

const fixtures: IssueFixture[] = [];
const directories: string[] = [];
const executableMode = 0o755;
afterEach(() => {
  for (const fixture of fixtures.splice(0)) {
    fixture.dispose();
  }
  for (const directory of directories.splice(0)) {
    rmSync(directory, { force: true, recursive: true });
  }
});

test("the registration entry point uses the passed connection despite a different caller environment", async () => {
  const fixture = issueFixture();
  fixtures.push(fixture);
  const directory = mkdtempSync(join(tmpdir(), "herdr-register-entry-"));
  directories.push(directory);
  const binary = join(directory, "herdr-fixture.ts");
  writeNativeFixture(binary, fixture);
  const child = Bun.spawn(
    [
      process.execPath,
      "--config=/dev/null",
      "--no-env-file",
      fileURLToPath(new URL("register-work.ts", import.meta.url)),
      directory,
      "/verified/session.sock",
      binary,
    ],
    {
      env: {
        ...process.env,
        HERDR_ENV: "1",
        HERDR_BIN_PATH: "/unavailable/ambient-herdr",
        HERDR_SOCKET_PATH: "/different/caller.sock",
      },
      stdin: new Blob([registrationInput(fixture)]),
      stdout: "pipe",
      stderr: "pipe",
    },
  );
  const [status, errors] = await Promise.all([
    child.exited,
    new Response(child.stderr).text(),
  ]);
  expect({ status, errors }).toEqual({ status: 0, errors: "" });
  const bindings = await createBindingStore(directory).read();
  expect(bindings["w2:p1"]?.issue_role).toBe("work");
  expect(bindings["w2:p1"]?.issue_repo).toBe(
    fixture.selection.repository.commonDirectory,
  );
});

function writeNativeFixture(binary: string, fixture: IssueFixture): void {
  const snapshot = workingIssueState(fixture);
  const available = {
    ...snapshot,
    agents: [],
    panes: snapshot.panes.map(unoccupiedPane),
  };
  writeFileSync(
    binary,
    `#!${process.execPath}\nif (process.env.HERDR_SOCKET_PATH !== "/verified/session.sock") { process.stderr.write(JSON.stringify({error:{code:"wrong_session",message:"Another native session"}})); process.exitCode=1; } else if(process.argv.includes("snapshot")) { process.stdout.write(JSON.stringify({result:{snapshot:${JSON.stringify(available)}}})); }`,
  );
  chmodSync(binary, executableMode);
}

function unoccupiedPane(pane: NativePane): NativePane {
  return { ...pane, agent: null, tokens: {} };
}

function registrationInput(fixture: IssueFixture): string {
  return JSON.stringify({
    provider: "codex",
    clickJson: JSON.stringify(fixture.selection.click.context),
    sourceRoot: fixture.selection.repository.root,
    sourceCommonDirectory: fixture.selection.repository.commonDirectory,
    paneId: "w2:p1",
    issueId: "GH-17",
    issueTitle: "Fixture issue",
    branch: "issue-work",
  });
}
