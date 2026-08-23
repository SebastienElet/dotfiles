import { describe, expect, test } from "bun:test";
import {
  aggregateMeasurements,
  parseTokeiOutput,
} from "./codegraph-repository-measurement.ts";

describe("repository measurement", () => {
  test("aggregates source records at both initialization boundaries", () => {
    expect(aggregateMeasurements(records(49_999, 499))).toEqual({
      loc: 49_999,
      files: 499,
      initialize: false,
    });
    expect(aggregateMeasurements(records(50_000, 1)).initialize).toBe(true);
    expect(aggregateMeasurements(records(1, 500)).initialize).toBe(true);
  });

  test("accepts supported extensions case-insensitively and rejects other records", () => {
    expect(
      aggregateMeasurements(
        parseTokeiOutput(
          [
            record("kept.CJS", 2),
            record("kept.tofu.tf", 3),
            record("ignored.hcl", 5),
            record("ignored.yaml", 7),
          ].join(""),
        ),
      ),
    ).toEqual({ loc: 5, files: 2, initialize: false });
  });

  test("rejects malformed or unsafe Tokei records", () => {
    expect(() => parseTokeiOutput("not-json\n")).toThrow("invalid Tokei JSON");
    expect(() =>
      parseTokeiOutput('{"stats":{"name":"fixture.ts","stats":{}}}\n'),
    ).toThrow("invalid Tokei record");
    expect(() =>
      aggregateMeasurements(
        parseTokeiOutput(
          record("fixture.ts", Number.MAX_SAFE_INTEGER).concat(
            record("second.ts", 1),
          ),
        ),
      ),
    ).toThrow("safe integer");
  });
});

function records(loc: number, files: number) {
  return Array.from({ length: files }, (_, index) => ({
    name: `fixture-${index}.ts`,
    code: index === 0 ? loc : 0,
  }));
}

function record(name: string, code: number): string {
  return `${JSON.stringify({ stats: { name, stats: { code } } })}\n`;
}
