import { z } from "zod";

export type SourceMeasurement = { name: string; code: number };
export type RepositoryMeasurement = {
  loc: number;
  files: number;
  initialize: boolean;
};

const sourceExtension =
  /\.(astro|c|cc|cbl|cob|cobol|cfc|cfm|cfs|cjs|cpp|cpy|cs|cshtml|cts|cu|cuh|cxx|dart|dfm|dpk|dpr|erl|escript|ets|fmx|go|h|hpp|hrl|hxx|inc|install|java|js|jsx|kt|kts|liquid|lpr|lua|luau|m|metal|mjs|mm|module|mts|nix|pas|php|py|pyw|r|rake|razor|rb|rs|sc|scala|sol|svelte|swift|tf|tfvars|theme|ts|tsx|vb|vue|xsjs|xsjslib)$/i;

const tokeiRecordSchema = z.object({
  stats: z.object({
    name: z.string(),
    stats: z.object({ code: z.number().int().nonnegative() }),
  }),
});

export function parseTokeiOutput(output: string): SourceMeasurement[] {
  if (output === "") {
    return [];
  }
  const lines = output.endsWith("\n")
    ? output.slice(0, -1).split("\n")
    : output.split("\n");
  return lines.map((line) => {
    let value: unknown;
    try {
      value = JSON.parse(line);
    } catch {
      throw new MeasurementError("invalid Tokei JSON");
    }
    const parsed = tokeiRecordSchema.safeParse(value);
    if (!parsed.success) {
      throw new MeasurementError("invalid Tokei record");
    }
    return {
      name: parsed.data.stats.name,
      code: parsed.data.stats.stats.code,
    };
  });
}

export function aggregateMeasurements(
  measurements: SourceMeasurement[],
): RepositoryMeasurement {
  const source = measurements.filter(({ name }) => sourceExtension.test(name));
  const loc = source.reduce((total, measurement) => {
    const next = total + measurement.code;
    if (!Number.isSafeInteger(next)) {
      throw new MeasurementError("Tokei LOC total is not a safe integer");
    }
    return next;
  }, 0);
  const files = source.length;
  return { loc, files, initialize: loc >= 50_000 || files >= 500 };
}

export class MeasurementError extends Error {
  constructor(
    message: string,
    readonly exitCode = 2,
  ) {
    super(message);
  }
}
