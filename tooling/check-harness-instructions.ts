import { join } from "node:path";
import { readFileSync } from "node:fs";
import { reportCheckFailure } from "./check-command.ts";
import { z } from "zod";

const argumentsSchema = z.tuple([z.string().min(1)]);
const argumentOffset = 2;
const sectionHeadingLevel = 2;
type SectionHeading = Readonly<{ index: number; text: string }>;

function countSectionHeadings(document: string): number {
  let count = 0;
  Bun.markdown.render(document, {
    heading: (_children, { level }: Readonly<Bun.markdown.HeadingMeta>) => {
      if (level <= sectionHeadingLevel) {
        count += 1;
      }
      return "";
    },
  });
  return count;
}

function contextManagement(document: string): string {
  const headings = [...document.matchAll(/^#{1,2} [^\n]*(?:\n|$)/gmu)]
    .map((heading: Readonly<{ index: number; 0: string }>): SectionHeading => ({
      index: heading.index,
      text: heading[0],
    }))
    .filter(
      (heading) =>
        countSectionHeadings(
          document.slice(0, heading.index + heading.text.length),
        ) ===
        countSectionHeadings(document.slice(0, heading.index)) + 1,
    );
  const contextHeadings = headings.filter(
    (heading) => heading.text === "## Context Management\n",
  );
  const [heading] = contextHeadings;
  if (heading === undefined || contextHeadings.length !== 1) {
    throw new Error(
      "AGENTS.md must contain exactly one Context Management section",
    );
  }
  const nextHeading = headings.find(
    (candidate) => candidate.index > heading.index,
  );
  const body = document.slice(
    heading.index + heading.text.length,
    nextHeading?.index,
  );
  if (body.trim().length === 0) {
    throw new Error("Context Management in AGENTS.md is empty");
  }
  return (heading.text + body).replace(/\n+$/u, "\n");
}

if (import.meta.main) {
  try {
    const [harness] = argumentsSchema.parse(process.argv.slice(argumentOffset));
    const expected = contextManagement(
      readFileSync(join(harness, "AGENTS.md"), "utf8"),
    );
    if (readFileSync(join(harness, "instructions.md"), "utf8") !== expected) {
      throw new Error(
        "instructions.md differs from Context Management in AGENTS.md",
      );
    }
  } catch (error) {
    reportCheckFailure(error);
  }
}
