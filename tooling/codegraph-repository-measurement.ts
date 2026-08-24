import { z } from "zod";

interface SourceMeasurement {
  readonly name: string;
  readonly code: number;
}
interface RepositoryMeasurement {
  readonly loc: number;
  readonly files: number;
  readonly initialize: boolean;
}

const sourceExtension =
  /\.(?:astro|c|cc|cbl|cob|cobol|cfc|cfm|cfs|cjs|cpp|cpy|cs|cshtml|cts|cu|cuh|cxx|dart|dfm|dpk|dpr|erl|escript|ets|fmx|go|h|hpp|hrl|hxx|inc|install|java|js|jsx|kt|kts|liquid|lpr|lua|luau|m|metal|mjs|mm|module|mts|nix|pas|php|py|pyw|r|rake|razor|rb|rs|sc|scala|sol|svelte|swift|tf|tfvars|theme|ts|tsx|vb|vue|xsjs|xsjslib)$/iu;
const tokeiRecordSchema = z.object({
  stats: z.object({
    name: z.string(),
    stats: z.object({ code: z.number().int().nonnegative() }),
  }),
});
const measurementErrorExitCode = 2;
const sourceLineThreshold = 50_000;
const sourceFileThreshold = 500;

class MeasurementError extends Error {
  public override readonly name = "MeasurementError";

  public constructor(
    message: string,
    public readonly exitCode = measurementErrorExitCode,
  ) {
    super(message);
  }
}

function parseTokeiOutput(output: string): SourceMeasurement[] {
  if (output === "") {
    return [];
  }
  const lines = output.endsWith("\n")
    ? output.slice(0, -1).split("\n")
    : output.split("\n");
  return lines.map((line) => {
    const value = parseTokeiLine(line);
    const parsed = tokeiRecordSchema.safeParse(value);
    if (!parsed.success) {
      throw new MeasurementError("invalid Tokei record");
    }
    return {
      code: parsed.data.stats.stats.code,
      name: parsed.data.stats.name,
    };
  });
}

function parseTokeiLine(line: string): unknown {
  try {
    return JSON.parse(line);
  } catch {
    throw new MeasurementError("invalid Tokei JSON");
  }
}

function aggregateMeasurements(
  measurements: readonly SourceMeasurement[],
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
  return {
    loc,
    files,
    initialize: loc >= sourceLineThreshold || files >= sourceFileThreshold,
  };
}

export {
  aggregateMeasurements,
  MeasurementError,
  type RepositoryMeasurement,
  type SourceMeasurement,
  parseTokeiOutput,
};
