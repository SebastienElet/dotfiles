import { expect, test } from "bun:test";
import { parseIgnoredPathOutput } from "./git-ignore-audit.ts";

const encode = (value: string): Uint8Array => new TextEncoder().encode(value);
const invalidUtf8Byte = 0xff;

test("accepts exact NUL-delimited ignored path evidence", () => {
  expect(
    parseIgnoredPathOutput(
      encode("assets/runtime.log\0scripts/cache/output.json\0"),
      ["assets/runtime.log", "scripts/cache/output.json", "SKILL.md"],
    ),
  ).toEqual(new Set(["assets/runtime.log", "scripts/cache/output.json"]));
});

test("rejects malformed or unknown Git path evidence", () => {
  expect(() =>
    parseIgnoredPathOutput(encode("assets/runtime.log"), []),
  ).toThrow("malformed path evidence");
  expect(() =>
    parseIgnoredPathOutput(encode("unknown\0"), ["SKILL.md"]),
  ).toThrow("malformed path evidence");
  expect(() =>
    parseIgnoredPathOutput(encode("SKILL.md\0SKILL.md\0"), ["SKILL.md"]),
  ).toThrow("malformed path evidence");
});

test("rejects invalid UTF-8 Git path evidence", () => {
  expect(() =>
    parseIgnoredPathOutput(Uint8Array.from([invalidUtf8Byte, 0]), ["SKILL.md"]),
  ).toThrow("The encoded data was not valid");
});
