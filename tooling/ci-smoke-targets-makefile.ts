const assignmentPattern =
  /^\s*(?:export\s+)?[A-Za-z_][A-Za-z0-9_.-]*\s*[?:+!]?=/u;
const directivePattern =
  /^\s*(?:-?include|sinclude|ifeq|ifneq|ifdef|ifndef|else|endif|define|endef|override|undefine|vpath)(?:[\s(]|$)/u;
const targetPattern = /^[A-Za-z0-9][A-Za-z0-9._-]*$/u;
const escapedBackslashPairLength = 2;

type Range = Readonly<{ count: number; start: number }>;
type Rule = Readonly<{ target: string | null }>;

function targetAtLine(
  makefile: readonly string[],
  line: number,
): string | undefined {
  return makefile
    .slice(0, line)
    .findLast((content) => content.startsWith(".PHONY: "))
    ?.split(/\s+/u)[1];
}

function parseRule(content: string): Rule | undefined {
  if (/^\s|^#|^\.PHONY:/u.test(content) || !content.includes(":")) {
    return undefined;
  }
  const target = /^(?<target>[A-Za-z0-9][A-Za-z0-9._-]*):(?!:)/u.exec(content)
    ?.groups?.target;
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
    .map((content) => parseRule(content))
    .findLast((rule) => rule !== undefined);
}

function endsWithUnescapedBackslash(content: string): boolean {
  const backslashes = /\\+$/u.exec(content.trimEnd())?.[0].length ?? 0;
  return backslashes % escapedBackslashPairLength === 1;
}

function logicalLineStart(makefile: readonly string[], line: number): number {
  let start = line - 1;
  while (start > 0 && endsWithUnescapedBackslash(makefile[start - 1] ?? "")) {
    start -= 1;
  }
  return start;
}

function isGlobalContinuation(
  makefile: readonly string[],
  line: number,
): boolean {
  const start = logicalLineStart(makefile, line);
  return start < line - 1 && makefile[start]?.startsWith("\t") !== true;
}

function isInsideDefine(makefile: readonly string[], line: number): boolean {
  return (
    makefile.slice(0, line - 1).reduce((depth, content, index) => {
      const startsLogicalLine =
        index === 0 || !endsWithUnescapedBackslash(makefile[index - 1] ?? "");
      if (!startsLogicalLine) {
        return depth;
      }
      if (/^[ ]*endef[ ]*$/u.test(content)) {
        return Math.max(0, depth - 1);
      }
      if (
        /^[ ]*(?:(?:export|override|private)\s+)*define(?:\s|$)/u.test(content)
      ) {
        return depth + 1;
      }
      return depth;
    }, 0) > 0
  );
}

function classifyLine(makefile: readonly string[], line: number): string {
  const content = makefile[line - 1] ?? "";
  const target = targetAtLine(makefile, line);
  if (isGlobalContinuation(makefile, line) || isInsideDefine(makefile, line)) {
    return "all";
  }
  if (assignmentPattern.test(content) || directivePattern.test(content)) {
    return "all";
  }

  const rule = ruleAtLine(makefile, line);
  if (rule && (rule.target === null || rule.target !== target)) {
    return "all";
  }
  if (content.startsWith("\t")) {
    return rule?.target === target ? (target ?? "all") : "all";
  }
  if (endsWithUnescapedBackslash(content)) {
    return "all";
  }
  const nonRuleTarget = targetForNonRuleLine(content, target);
  if (nonRuleTarget !== undefined) {
    return nonRuleTarget;
  }
  const declaredRule = parseRule(content);
  if (declaredRule?.target === target) {
    return target ?? "all";
  }
  return "all";
}

function targetForNonRuleLine(
  content: string,
  target: string | undefined,
): string | undefined {
  if (
    /^\s*$/u.test(content) ||
    content.startsWith(".PHONY: ") ||
    /^\s*#/u.test(content)
  ) {
    return target ?? "all";
  }
  return undefined;
}

function parseRange(start: string, count: string | undefined): Range {
  if (!/^\d+$/u.test(start) || (count !== undefined && !/^\d+$/u.test(count))) {
    throw new Error(`invalid diff range: ${start},${count ?? "1"}`);
  }
  return { count: Number(count ?? "1"), start: Number(start) };
}

function parseRanges(
  diff: string,
): readonly Readonly<{ newRange: Range; oldRange: Range }>[] {
  const hunkLines = diff.split("\n").filter((line) => line.startsWith("@@"));
  const ranges = hunkLines.map((line) => {
    const match =
      /^@@ -(?<oldStart>\d+)(?:,(?<oldCount>\d+))? \+(?<newStart>\d+)(?:,(?<newCount>\d+))? @@/u.exec(
        line,
      );
    const groups = match?.groups;
    if (
      groups?.oldStart === undefined ||
      groups.oldStart === "" ||
      groups.newStart === undefined ||
      groups.newStart === ""
    ) {
      throw new Error(`invalid diff hunk: ${line}`);
    }
    return {
      newRange: parseRange(groups.newStart, groups.newCount),
      oldRange: parseRange(groups.oldStart, groups.oldCount),
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
  return Array.from({ length: range.count }, (_unused, index) =>
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
  if (!phonyDeclarations.every((line) => /^\.PHONY: \S+$/u.test(line))) {
    return ["all"];
  }

  const candidates: string[] = [];
  for (const { oldRange, newRange } of parseRanges(diff)) {
    candidates.push(
      ...targetsInRange(oldMakefile, oldRange),
      ...targetsInRange(newMakefile, newRange),
    );
  }
  const valid = candidates.every(
    (target) =>
      targetPattern.test(target) && newMakefile.includes(`.PHONY: ${target}`),
  );
  return valid ? candidates : ["all"];
}
