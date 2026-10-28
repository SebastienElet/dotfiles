import { afterEach, expect, test } from "bun:test";
import { mkdirSync, mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { tmpdir } from "node:os";

const roots: string[] = [];
const section = "## Context Management\n\n- Read the source.\n";

afterEach(() => {
  for (const root of roots.splice(0)) {
    rmSync(root, { force: true, recursive: true });
  }
});

function check(
  agents: string,
  instructions: string,
): ReturnType<typeof invoke> {
  const root = mkdtempSync(join(tmpdir(), "harness-instructions-"));
  roots.push(root);
  writeFileSync(join(root, "AGENTS.md"), agents);
  writeFileSync(join(root, "instructions.md"), instructions);
  return invoke([root]);
}

function invoke(arguments_: readonly string[]): Readonly<{
  exitCode: number;
  stderr: string;
}> {
  const result = Bun.spawnSync([
    process.execPath,
    "--config=/dev/null",
    "--no-env-file",
    join(import.meta.dir, "check-harness-instructions.ts"),
    ...arguments_,
  ]);
  return { exitCode: result.exitCode, stderr: result.stderr.toString() };
}

test("accepts the frozen section independently of surrounding instructions", () => {
  expect(
    check(`# Instructions\n\n${section}\n## Other\n\nUnrelated.\n`, section),
  ).toEqual({
    exitCode: 0,
    stderr: "",
  });
});

test("rejects a divergent fixture", () => {
  const result = check(section, section.replace("Read", "Ignore"));
  expect(result.exitCode).toBe(1);
  expect(result.stderr).toContain(
    "instructions.md differs from Context Management in AGENTS.md",
  );
});

test("includes nested headings and accepts a section at end of file", () => {
  const nested = `${section}\n### Details\n\n- Verify the source.\n`;
  expect(check(nested, nested)).toEqual({ exitCode: 0, stderr: "" });
});

test.each(["```markdown", "~~~markdown", "<!--"])(
  "rejects a section present only inside %s",
  (opening) => {
    expect(check(`${opening}\n${section}\n## Other\n`, section).exitCode).toBe(
      1,
    );
  },
);

test("keeps inactive headings inside the compared section", () => {
  const body = `${section}\n\`\`\`markdown\n## Example\n\`\`\`\n\n- Verify this too.\n`;
  expect(check(`${body}\n## Other\n`, body).exitCode).toBe(0);
  expect(check(`${body}\n## Other\n`, section).exitCode).toBe(1);
});

test.each([
  section.replace("Read", "Ignore"),
  "# Instructions\n",
  section.replace("Context Management", "Context"),
  `${section}\n${section}`,
  "## Context Management\n\n## Other\ncontent\n",
])(
  "rejects changed, missing, duplicate or empty source sections: %s",
  (agents) => {
    expect(check(agents, section).exitCode).toBe(1);
  },
);

test.each(["", `${section}\n## Other\n`, section.replace("Read", " Read")])(
  "rejects an empty, extended or whitespace-divergent fixture: %s",
  (instructions) => {
    expect(check(section, instructions).exitCode).toBe(1);
  },
);

test.each(["AGENTS.md", "instructions.md"])(
  "refuses unavailable %s",
  (file) => {
    const root = mkdtempSync(join(tmpdir(), "harness-instructions-missing-"));
    roots.push(root);
    writeFileSync(
      join(root, file === "AGENTS.md" ? "instructions.md" : "AGENTS.md"),
      section,
    );
    const result = invoke([root]);
    expect(result.exitCode).toBe(1);
    expect(result.stderr).toContain(file);
  },
);

test("refuses a directory in place of an input file", () => {
  const root = mkdtempSync(join(tmpdir(), "harness-instructions-directory-"));
  roots.push(root);
  mkdirSync(join(root, "AGENTS.md"));
  expect(invoke([root]).exitCode).toBe(1);
});

test.each([
  { arguments: [] },
  { arguments: [""] },
  { arguments: ["harness", "extra"] },
])("refuses invalid arguments: %j", ({ arguments: arguments_ }) => {
  expect(invoke(arguments_).exitCode).toBe(1);
});
