import {
  type SourceMeasurement,
  aggregateMeasurements,
  parseTokeiOutput,
} from "./codegraph-repository-measurement.ts";
import { describe, expect, test } from "bun:test";

const belowLineThreshold = 49_999;
const belowFileThreshold = 499;
const lineThreshold = 50_000;
const fileThreshold = 500;
const cjsLines = 2;
const tofuLines = 3;
const ignoredHclLines = 5;
const ignoredYamlLines = 7;
const expectedKeptFileCount = 2;
const expectedKeptSourceLines = cjsLines + tofuLines;

describe("repository measurement", () => {
  test("aggregates source records at both initialization boundaries", () => {
    expect(
      aggregateMeasurements(records(belowLineThreshold, belowFileThreshold)),
    ).toEqual({
      files: belowFileThreshold,
      initialize: false,
      loc: belowLineThreshold,
    });
    expect(aggregateMeasurements(records(lineThreshold, 1)).initialize).toBe(
      true,
    );
    expect(aggregateMeasurements(records(1, fileThreshold)).initialize).toBe(
      true,
    );
  });

  test("accepts supported extensions case-insensitively and rejects other records", () => {
    expect(
      aggregateMeasurements(
        parseTokeiOutput(
          [
            record("kept.CJS", cjsLines),
            record("kept.tofu.tf", tofuLines),
            record("ignored.hcl", ignoredHclLines),
            record("ignored.yaml", ignoredYamlLines),
          ].join(""),
        ),
      ),
    ).toEqual({
      files: expectedKeptFileCount,
      initialize: false,
      loc: expectedKeptSourceLines,
    });
  });

  test("rejects malformed or unsafe Tokei records", () => {
    expect(() => parseTokeiOutput("not-json\n")).toThrow("invalid Tokei JSON");
    expect(() =>
      parseTokeiOutput('{"stats":{"name":"fixture.ts","stats":{}}}\n'),
    ).toThrow("invalid Tokei record");
    expect(() =>
      aggregateMeasurements(
        parseTokeiOutput(
          `${record("fixture.ts", Number.MAX_SAFE_INTEGER)}${record("second.ts", 1)}`,
        ),
      ),
    ).toThrow("safe integer");
  });
});

function records(loc: number, files: number): SourceMeasurement[] {
  return Array.from({ length: files }, (_unusedValue, index) => ({
    code: index === 0 ? loc : 0,
    name: `fixture-${index}.ts`,
  }));
}

function record(name: string, code: number): string {
  return `${JSON.stringify({ stats: { name, stats: { code } } })}\n`;
}
