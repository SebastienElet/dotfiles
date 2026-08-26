import { expect, test } from "bun:test";
import {
  findUnexpectedResourceFiles,
  parseResourceFilePolicy,
} from "./check-resource-files.ts";

const policy = parseResourceFilePolicy({
  resourceDirectories: {
    agents: { files: ["openai.yaml"], mode: "closed" },
    assets: { mode: "open" },
    evals: {
      files: ["evals.json", "trigger-queries.json"],
      mode: "closed",
    },
    references: { mode: "open" },
    scripts: { mode: "open" },
  },
  rootFiles: ["SKILL.md"],
  version: 1,
});

test("rejects a malformed policy instead of choosing a permissive default", () => {
  expect(() =>
    parseResourceFilePolicy({
      ...policy,
      resourceDirectories: {
        ...policy.resourceDirectories,
        evals: { mode: "permissive" },
      },
    }),
  ).toThrow();
});

test("rejects every file outside a closed directory policy", () => {
  expect(
    findUnexpectedResourceFiles(
      [
        "SKILL.md",
        "agents/provider.yaml",
        "evals/cases.md",
        "evals/files/input.csv",
      ],
      policy,
    ),
  ).toEqual([
    {
      convention: "agents/ admits only openai.yaml",
      path: "agents/provider.yaml",
    },
    {
      convention: "evals/ admits only evals.json and trigger-queries.json",
      path: "evals/cases.md",
    },
    {
      convention: "evals/ admits only evals.json and trigger-queries.json",
      path: "evals/files/input.csv",
    },
  ]);
});

test("accepts both eval variants and extensible resource directories", () => {
  expect(
    findUnexpectedResourceFiles(
      [
        "SKILL.md",
        "agents/openai.yaml",
        "assets/templates/report.md",
        "evals/evals.json",
        "evals/trigger-queries.json",
        "references/provider/contracts.md",
        "scripts/check.ts",
      ],
      policy,
    ),
  ).toEqual([]);
});

test("rejects unexpected root files and directories", () => {
  expect(
    findUnexpectedResourceFiles(
      ["SKILL.md", "README.md", "examples/case.md"],
      policy,
    ),
  ).toEqual([
    {
      convention:
        "skill root admits only SKILL.md and agents/, assets/, evals/, references/ and scripts/",
      path: "README.md",
    },
    {
      convention:
        "skill root admits only SKILL.md and agents/, assets/, evals/, references/ and scripts/",
      path: "examples/case.md",
    },
  ]);
});
