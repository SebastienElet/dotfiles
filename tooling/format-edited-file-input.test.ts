import { expect, test } from "bun:test";
import { editedPaths } from "./format-edited-file-input.ts";

function postToolUse(toolName: string, toolInput: unknown): string {
  return JSON.stringify({
    cwd: "/work/repository",
    hook_event_name: "PostToolUse",
    session_id: "session-1",
    tool_input: toolInput,
    tool_name: toolName,
    tool_response: {},
  });
}

test("reads the file path of a Claude Edit", () => {
  expect(
    editedPaths(
      postToolUse("Edit", {
        file_path: "/work/repository/notes.md",
        new_string: "b",
        old_string: "a",
      }),
    ),
  ).toEqual({ paths: ["/work/repository/notes.md"], success: true });
});

test("reads the file path of a Claude Write", () => {
  expect(
    editedPaths(
      postToolUse("Write", {
        content: "text",
        file_path: "/work/repository/notes.md",
      }),
    ),
  ).toEqual({ paths: ["/work/repository/notes.md"], success: true });
});

test("reads every file a Codex patch adds, updates or moves, relative to cwd", () => {
  const patch = [
    "*** Begin Patch",
    "*** Add File: docs/new.md",
    "+text",
    "*** Update File: tooling/old.ts",
    "*** Move to: tooling/renamed.ts",
    "@@",
    "-a",
    "+b",
    "*** Update File: /elsewhere/absolute.md",
    "@@",
    "-a",
    "+b",
    "*** Delete File: docs/removed.md",
    "*** End Patch",
  ].join("\n");

  expect(editedPaths(postToolUse("apply_patch", { command: patch }))).toEqual({
    paths: [
      "/work/repository/docs/new.md",
      "/work/repository/tooling/renamed.ts",
      "/elsewhere/absolute.md",
    ],
    success: true,
  });
});

test("ignores tools that do not edit files", () => {
  expect(editedPaths(postToolUse("Bash", { command: "ls" }))).toEqual({
    paths: [],
    success: true,
  });
});

test("rejects input that is not JSON", () => {
  const outcome = editedPaths("not json");

  expect(outcome.success).toBe(false);
});

test("rejects an Edit without a file path", () => {
  const outcome = editedPaths(postToolUse("Edit", { old_string: "a" }));

  expect(outcome.success).toBe(false);
});

test("rejects a relative Claude file path", () => {
  const outcome = editedPaths(
    postToolUse("Write", { content: "", file_path: "notes.md" }),
  );

  expect(outcome.success).toBe(false);
});
