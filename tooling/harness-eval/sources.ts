import {
  type BehavioralCase,
  type Fixture,
  type Trigger,
  caseSchema,
  casesSchema,
  fixtureSchema,
  pathSchema,
  triggerSchema,
} from "./contracts.ts";
import { dirname, relative, resolve, sep } from "node:path";
import { readFileSync, realpathSync } from "node:fs";
import { createHash } from "node:crypto";

const fingerprint = (value: string): string =>
  createHash("sha256").update(value).digest("hex");

function sourcePath(root: string, path: string): string {
  pathSchema.parse(path);
  const canonical = realpathSync(resolve(root, path));
  const child = relative(realpathSync(root), canonical);
  if (child.startsWith(`..${sep}`) || child === ".." || child.startsWith(sep)) {
    throw new Error(`Escaping source: ${path}`);
  }
  return canonical;
}

function readSource(root: string, path: string): string {
  return readFileSync(sourcePath(root, path), "utf8");
}

function section(markdown: string, heading: string): string {
  const start = markdown.indexOf(`## ${heading}\n`);
  if (start === -1) {
    throw new Error(`Missing section: ${heading}`);
  }
  const end = markdown.indexOf("\n## ", start + 1);
  return markdown.slice(start, end === -1 ? undefined : end);
}

function loadTrigger(root: string, path: string): Trigger {
  const trigger = triggerSchema.parse(JSON.parse(readSource(root, path)));
  if (trigger.skill !== dirname(dirname(path)).split("/").at(-1)) {
    throw new Error(`Skill slug mismatch: ${path}`);
  }
  return trigger;
}

function loadCases(root: string): readonly LoadedCase[] {
  const definitions: readonly BehavioralCase[] = casesSchema.parse(
    JSON.parse(readSource(root, "harness/evals/cases.json")),
  );
  return definitions.map((definition) => resolveCase(root, definition));
}

function resolveCase(root: string, raw: unknown): LoadedCase {
  const definition = caseSchema.parse(raw);
  const sources = definition.sources.map((source) => {
    const content = readSource(root, source.path);
    section(content, source.heading);
    return { ...source, fingerprint: fingerprint(content) };
  });
  const prompt =
    "text" in definition.prompt
      ? definition.prompt.text
      : loadTrigger(root, definition.prompt.triggerFile).queries[
          definition.prompt.queryIndex
        ]?.query;
  if (prompt === undefined || prompt === "") {
    throw new Error(`Unresolved prompt: ${definition.id}`);
  }
  const fixture = fixtureSchema.parse(
    JSON.parse(
      readSource(root, `harness/evals/fixtures/${definition.fixture}.json`),
    ),
  );
  if (fixture.id !== definition.fixture) {
    throw new Error(`Fixture identity mismatch: ${definition.id}`);
  }
  return {
    definition,
    sources,
    prompt,
    promptFingerprint: fingerprint(prompt),
    fixture,
  };
}

type LoadedCase = Readonly<{
  definition: BehavioralCase;
  sources: readonly Readonly<{
    path: string;
    heading: string;
    fingerprint: string;
  }>[];
  prompt: string;
  promptFingerprint: string;
  fixture: Fixture;
}>;

export {
  fingerprint,
  sourcePath,
  readSource,
  section,
  loadTrigger,
  loadCases,
  type LoadedCase,
};
