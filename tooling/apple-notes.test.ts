import { afterEach, describe, expect, test } from "bun:test";
import { readFileSync } from "node:fs";
import { join } from "node:path";
import { cleanupNotesFixtures, runNotes } from "./apple-notes-test-support.ts";

afterEach(cleanupNotesFixtures);

describe("Apple Notes shipped entrypoint", () => {
  test("creates a folder in the default account and escapes input", async () => {
    const result = await runNotes(
      ["folder", 'A "folder"'],
      [{ stdout: "A folder\n" }],
    );
    expect([result.exitCode, result.stdout, result.stderr]).toEqual([
      0,
      "A folder\n",
      "",
    ]);
    expect(result.calls[0]).toContain('account "iCloud"');
    expect(result.calls[0]).toContain('A \\"folder\\"');
  });

  test("creates a note with its title first and refuses an empty body before mutation", async () => {
    const created = await runNotes(
      ["note", "Inbox", "Title", "Work"],
      [{ stdout: "Title\n" }],
      "<div>Body</div>\n",
    );
    expect(created.exitCode).toBe(0);
    expect(created.calls[0]).toContain('account "Work"');
    expect(created.calls[0]).toContain("<div><h1>Title</h1></div>");
    const empty = await runNotes(["note", "Inbox", "Title"], []);
    expect([empty.exitCode, empty.calls.length]).toEqual([1, 0]);
    expect(empty.stderr).toContain("empty note body");
  });

  test.each([
    ["true\t1\t0", "shared folder"],
    ["false\t0\t0", "exactly one"],
    ["false\t2\t0", "exactly one"],
    ["false\t1\tunknown", "unexpected AppleScript move evidence"],
  ])(
    "refuses invalid move evidence before mutation",
    async (stdout, diagnostic) => {
      const result = await runNotes(
        ["move", "Notes", "Title", "3 Resources/Test"],
        [{ stdout }],
      );
      expect([result.exitCode, result.calls.length]).toEqual([1, 1]);
      expect(result.stderr).toContain(diagnostic);
    },
  );

  test("moves and renames attachments without reading or rewriting the body", async () => {
    const result = await runNotes(
      ["move", "Notes", "Old", "3 Resources/Test", "New"],
      [{ stdout: "false\t1\t2" }, { stdout: "New\n" }],
    );
    expect(result.exitCode).toBe(0);
    expect(result.calls).toHaveLength(2);
    expect(result.calls[1]).toContain("set name");
    expect(result.calls[1]).toContain("move n");
    expect(result.calls[1]).toContain("return name of n");
    expect(result.calls.join("\n")).not.toContain("get body");
    expect(result.stderr).toContain("2 attachment(s) preserved");
  });

  test("makes a post-move failure explicit when Notes cannot provide a transaction", async () => {
    const result = await runNotes(
      ["move", "Notes", "Old", "3 Resources/Test", "New"],
      [
        { stdout: "false\t1\t2" },
        {
          status: 42,
          stderr:
            "note moved but its post-move update failed; inspect the destination\n",
        },
      ],
    );
    expect(result.exitCode).toBe(42);
    expect(result.calls[1]).toContain("set moveCompleted to true");
    expect(result.calls[1]).toContain("if moveCompleted then error");
    expect(result.stderr).toContain("inspect the destination");
  });

  test("prepares a nonempty rewritten body before one move mutation", async () => {
    const result = await runNotes(
      ["move", "Notes", "Old", "3 Resources/Test", "New"],
      [
        { stdout: "false\t1\t0" },
        { stdout: "<div>Old</div><div>Body</div>" },
        { stdout: "New\n" },
      ],
    );
    expect(result.exitCode).toBe(0);
    expect(result.calls).toHaveLength(3);
    const mutation = result.calls[2] ?? "";
    expect(mutation).toContain("move n");
    expect(mutation).toContain("set body");
    expect(mutation).toContain("count of attachments of n) is not 0");
    expect(mutation).toContain("source note changed before the move");
    expect(mutation.indexOf("source note changed")).toBeLessThan(
      mutation.indexOf("make new folder"),
    );
  });

  test("refuses an empty body read before moving the note", async () => {
    const result = await runNotes(
      ["move", "Notes", "Old", "3 Resources/Test", "New"],
      [{ stdout: "false\t1\t0" }, { stdout: "" }],
    );
    expect([result.exitCode, result.calls.length]).toEqual([1, 2]);
    expect(result.stderr).toContain("note is unchanged");
    expect(result.calls.join("\n")).not.toContain("move note");
  });

  test("refuses invalid UTF-8 body evidence before mutation", async () => {
    const result = await runNotes(
      ["move", "Notes", "Old", "3 Resources/Test", "New"],
      [{ stdout: "false\t1\t0" }, { stdoutBytes: [0xff] }],
    );
    expect([result.exitCode, result.calls.length]).toEqual([1, 2]);
    expect(result.stderr).toContain("invalid UTF-8");
    expect(result.calls.join("\n")).not.toContain("move n");
  });

  test("propagates AppleScript errors without another mutation", async () => {
    const result = await runNotes(
      ["folder", "Test"],
      [
        {
          status: 42,
          stdout: "partial output\n",
          stderr: "automation denied\n",
        },
      ],
    );
    expect([result.exitCode, result.calls.length]).toEqual([42, 1]);
    expect([result.stdout, result.stderr]).toEqual([
      "partial output\n",
      "automation denied\n",
    ]);
    const binary = await runNotes(
      ["folder", "Test"],
      [{ status: 42, stdoutBytes: [0xff], stderr: "automation denied\n" }],
    );
    expect([binary.exitCode, binary.stdoutBytes, binary.stderr]).toEqual([
      42,
      [0xff],
      "automation denied\n",
    ]);
  });

  test("is wired into the cross-platform deployment gate", () => {
    const workflow = readFileSync(
      join(import.meta.dir, "../.github/workflows/test-deployment.yml"),
      "utf8",
    );
    expect(workflow).toContain("os: [macos-latest, ubuntu-latest]");
    expect(workflow).toContain("bun test tooling/apple-notes*.test.ts");
    expect(workflow).toContain(".agents/skills/apple-notes/scripts/**");
  });
});
