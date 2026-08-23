const targetPattern = /^[A-Za-z0-9][A-Za-z0-9._-]*$/;
const assignmentPattern =
  /^\s*(?:export\s+)?[A-Za-z_][A-Za-z0-9_.-]*\s*[?:+!]?=/;
const directivePattern =
  /^\s*(?:-?include|sinclude|ifeq|ifneq|ifdef|ifndef|else|endif|define|endef|override|undefine|vpath)(?:[\s(]|$)/;

type Range = Readonly<{ count: number; start: number }>;
type Rule = Readonly<{ target: string | null }>;

function targetAtLine(
  makefile: readonly string[],
  line: number,
): string | undefined {
  return makefile
    .slice(0, line)
    .findLast((content) => content.startsWith(".PHONY: "))
    ?.split(/\s+/)[1];
}

function parseRule(content: string): Rule | undefined {
  if (/^\s|^#|^\.PHONY:/.test(content) || !content.includes(":")) {
    return undefined;
  }
  const target = /^([A-Za-z0-9][A-Za-z0-9._-]*):(?!:)/.exec(content)?.[1];
  return { target: target ?? null };
}

function ruleAtLine(
  makefile: readonly string[],
  line: number,
): Rule | undefined {
  const throughLine = makefile.slice(0, line);
  const declaration = throughLine.findLastIndex((content) =>
    content.startsWith(".PHONY: "),
  );
  return throughLine
    .slice(declaration + 1)
    .map(parseRule)
    .findLast((rule) => rule !== undefined);
}

function classifyLine(makefile: readonly string[], line: number): string {
  const content = makefile[line - 1] ?? "";
  const target = targetAtLine(makefile, line);
  if (assignmentPattern.test(content) || directivePattern.test(content)) {
    return "all";
  }

  const rule = ruleAtLine(makefile, line);
  if (rule && (rule.target === null || rule.target !== target)) {
    return "all";
  }
  return target ?? "all";
}

function parseRange(start: string, count: string | undefined): Range {
  if (!/^\d+$/.test(start) || (count !== undefined && !/^\d+$/.test(count))) {
    throw new Error(`invalid diff range: ${start},${count ?? "1"}`);
  }
  return { count: Number(count ?? "1"), start: Number(start) };
}

function parseRanges(
  diff: string,
): readonly Readonly<{ newRange: Range; oldRange: Range }>[] {
  const hunkLines = diff.split("\n").filter((line) => line.startsWith("@@"));
  const ranges = hunkLines.map((line) => {
    const match = /^@@ -(\d+)(?:,(\d+))? \+(\d+)(?:,(\d+))? @@/.exec(line);
    if (!match?.[1] || !match[3]) {
      throw new Error(`invalid diff hunk: ${line}`);
    }
    return {
      oldRange: parseRange(match[1], match[2]),
      newRange: parseRange(match[3], match[4]),
    };
  });
  if (ranges.length === 0) {
    throw new Error("Makefile changed without readable diff ranges");
  }
  return ranges;
}

function targetsInRange(
  makefile: readonly string[],
  range: Range,
): readonly string[] {
  return Array.from({ length: range.count }, (_, index) =>
    classifyLine(makefile, range.start + index),
  );
}

export function targetsFromMakefileDiff(
  oldContents: string,
  newContents: string,
  diff: string,
): readonly string[] {
  const oldMakefile = oldContents.split("\n");
  const newMakefile = newContents.split("\n");
  const phonyDeclarations = newMakefile.filter((line) =>
    line.startsWith(".PHONY:"),
  );
  if (!phonyDeclarations.every((line) => /^\.PHONY: \S+$/.test(line))) {
    return ["all"];
  }

  const candidates = parseRanges(diff).flatMap(({ oldRange, newRange }) => [
    ...targetsInRange(oldMakefile, oldRange),
    ...targetsInRange(newMakefile, newRange),
  ]);
  const valid = candidates.every(
    (target) =>
      targetPattern.test(target) && newMakefile.includes(`.PHONY: ${target}`),
  );
  return valid ? candidates : ["all"];
}
