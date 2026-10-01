import { expect, test } from "bun:test";
import { join } from "node:path";
import { readFileSync } from "node:fs";
import { runInNewContext } from "node:vm";
import { z } from "zod";

const source = readFileSync(
  join(import.meta.dir, "clean-deployment-worker-inspect.js"),
  "utf8",
);
const label = "dev.remem.worker";

function inspect(snapshot: unknown): unknown {
  const script = `${source}\nrun([${JSON.stringify(label)}]);`;
  const context = {
    ObjC: {
      import: (name: string): string => name,
      deepUnwrap: (value: unknown): unknown => value,
      castRefToObject: (value: unknown): unknown => value,
    },
  };
  Object.defineProperty(context, "$", {
    value: {
      SMCopyAllJobDictionaries: (): unknown => snapshot,
      kSMDomainUserLaunchd: "user",
    },
  });
  const result: unknown = runInNewContext(script, context);
  const document: unknown = JSON.parse(z.string().parse(result));
  return document;
}

test("reports absence only from a complete valid native snapshot", () => {
  expect(inspect([])).toEqual({ state: "absent" });
  expect(inspect([{ Label: "foreign" }])).toEqual({ state: "absent" });
});

test("selects only the worker identity without returning environment or other jobs", () => {
  const definition = {
    Label: label,
    Program: "remem",
    ProgramArguments: ["remem", "worker", "--once"],
  };
  const result = inspect([
    { Label: "foreign", secret: "keep" },
    { ...definition, EnvironmentVariables: { SECRET: "keep" } },
  ]);
  expect(result).toEqual({ state: "loaded", definition });
  expect(JSON.stringify(result)).not.toContain("SECRET");
});

test.each([
  null,
  undefined,
  {},
  [null],
  ["foreign"],
  [{}],
  [{ Label: 2 }],
  [{ Label: "foreign" }, { Label: undefined }],
])("refuses incomplete or malformed native snapshot %j", (snapshot) => {
  expect(() => inspect(snapshot)).toThrow();
});

test("refuses duplicate native service labels", () => {
  expect(() => inspect([{ Label: label }, { Label: label }])).toThrow();
});

test.each([
  { Label: label },
  { Label: label, ProgramArguments: "worker" },
  { Label: label, ProgramArguments: [] },
  { Label: label, ProgramArguments: [1] },
  { Label: label, ProgramArguments: ["remem"], Program: 1 },
])(
  "refuses an unknown selected service definition %j",
  (
    definition: Readonly<{
      Label: string;
      Program?: unknown;
      ProgramArguments?: unknown;
    }>,
  ) => {
    expect(() => inspect([definition])).toThrow();
  },
);
