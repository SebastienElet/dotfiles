import { afterEach, describe, expect, test } from "bun:test";
import { cleanupNotesFixtures, runNotes } from "./apple-notes-test-support.ts";
import {
  existsSync,
  lstatSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  readdirSync,
  rmSync,
  statSync,
  symlinkSync,
  writeFileSync,
} from "node:fs";
import { join } from "node:path";
import { publishDirectoryExclusively } from "../.agents/skills/apple-notes/scripts/notes-publish.ts";
import { tmpdir } from "node:os";

afterEach(cleanupNotesFixtures);

describe("Apple Notes attachment export", () => {
  test("stages exports and refuses an empty export without creating the target", async () => {
    const success = await runNotes(
      ["attachments", "Notes", "Photo", "exports"],
      [{ exportFile: "1-photo.jpg" }],
    );
    expect(success.exitCode).toBe(0);
    expect(
      readFileSync(join(success.root, "exports", "1-photo.jpg"), "utf8"),
    ).toBe("attachment");
    expect(success.calls[0]).toContain("count of matches) is not 1");
    expect(success.calls[0]).toContain("set n to item 1 of matches");
    expect(success.calls[0]).toContain('set exportPath to "');
    expect(success.calls[0]).toContain(
      "save a in file ((POSIX file exportPath)",
    );
    expect(success.calls[0]).toContain("log exportPath");
    const empty = await runNotes(
      ["attachments", "Notes", "Photo", "empty"],
      [{}],
    );
    expect(empty.exitCode).toBe(1);
    expect(existsSync(join(empty.root, "empty"))).toBe(false);
    expect(empty.stderr).toContain("nothing was exported");
  });

  test("leaves the target absent when AppleScript cannot export an attachment", async () => {
    const result = await runNotes(
      ["attachments", "Notes", "Photo", "exports"],
      [
        {
          exportFile: "partial.jpg",
          status: 1,
          stderr: "unsupported attachment\n",
        },
      ],
    );
    expect(result.exitCode).toBe(1);
    expect(existsSync(join(result.root, "exports"))).toBe(false);
    expect(result.stderr).toContain("unsupported attachment");
  });
});

describe("Apple Notes attachment destination safety", () => {
  test("refuses an existing or symlinked destination before AppleScript", async () => {
    for (const kind of ["directory", "symlink"] as const) {
      let originalInode = 0;
      const result = await runNotes(
        ["attachments", "Notes", "Photo", "exports"],
        [],
        {
          prepare: (root) => {
            if (kind === "directory") {
              mkdirSync(join(root, "exports"));
              writeFileSync(join(root, "exports", "keep.txt"), "keep");
            } else {
              mkdirSync(join(root, "foreign"));
              symlinkSync(join(root, "foreign"), join(root, "exports"));
            }
            originalInode = lstatSync(join(root, "exports")).ino;
          },
        },
      );
      expect([result.exitCode, result.calls.length]).toEqual([1, 0]);
      expect(lstatSync(join(result.root, "exports")).ino).toBe(originalInode);
      if (kind === "directory") {
        expect(
          readFileSync(join(result.root, "exports", "keep.txt"), "utf8"),
        ).toBe("keep");
      } else {
        expect(
          lstatSync(join(result.root, "exports")).isSymbolicLink(),
        ).toBeTrue();
        expect(statSync(join(result.root, "exports")).isDirectory()).toBeTrue();
      }
    }
  });

  test("does not replace a destination that appears during export", async () => {
    const result = await runNotes(
      ["attachments", "Notes", "Photo", "exports"],
      [{ appearDirectory: "exports", exportFile: "partial.jpg" }],
    );
    expect(result.exitCode).toBe(1);
    expect(
      readFileSync(join(result.root, "exports", "foreign.txt"), "utf8"),
    ).toBe("foreign");
    expect(
      readdirSync(result.root).some((name) => name.includes("notes-export")),
    ).toBeFalse();
  });
});

describe("Apple Notes attachment publication", () => {
  test("exclusive publication cannot replace an empty destination", () => {
    const root = mkdtempSync(join(tmpdir(), "notes-publish-test-"));
    try {
      const source = join(root, "source");
      const destination = join(root, "destination");
      mkdirSync(source);
      mkdirSync(destination);
      writeFileSync(join(source, "attachment.jpg"), "attachment");
      const destinationInode = lstatSync(destination).ino;
      expect(() => {
        publishDirectoryExclusively(source, destination);
      }).toThrow();
      expect(lstatSync(destination).ino).toBe(destinationInode);
      expect(readFileSync(join(source, "attachment.jpg"), "utf8")).toBe(
        "attachment",
      );
    } finally {
      rmSync(root, { force: true, recursive: true });
    }
  });

  test("refuses ambiguous attachment evidence before saving", async () => {
    const result = await runNotes(
      ["attachments", "Notes", "Photo", "exports"],
      [{ status: 1, stderr: "expected exactly one source note\n" }],
    );
    expect(result.exitCode).toBe(1);
    expect(result.calls[0]).toContain("set n to item 1 of matches");
    expect(existsSync(join(result.root, "exports"))).toBeFalse();
  });
});
