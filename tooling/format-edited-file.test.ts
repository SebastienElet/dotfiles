import { afterEach, expect, test } from "bun:test";
import {
  chmod,
  mkdir,
  mkdtemp,
  readFile,
  realpath,
  rm,
  symlink,
  writeFile,
} from "node:fs/promises";
import { dirname, join } from "node:path";
import { formatEditedFiles } from "./format-edited-file.ts";
import { tmpdir } from "node:os";

const temporaryRoots: string[] = [];
const defaultTimeoutMilliseconds = 10_000;
const shortTimeoutMilliseconds = 300;
const executableMode = 0o755;
const patchedFileCount = 2;

afterEach(async () => {
  await Promise.all(
    temporaryRoots
      .splice(0)
      .map((root) => rm(root, { force: true, recursive: true })),
  );
});

async function temporaryDirectory(): Promise<string> {
  const root = await realpath(
    await mkdtemp(join(tmpdir(), "format-edited-file-")),
  );
  temporaryRoots.push(root);
  return root;
}

async function repository(): Promise<string> {
  const root = await temporaryDirectory();
  const init = Bun.spawnSync(["git", "init", "--quiet", root]);
  expect(init.exitCode).toBe(0);
  return root;
}

async function writeFixture(
  root: string,
  path: string,
  content: string,
): Promise<string> {
  const absolutePath = join(root, path);
  await mkdir(dirname(absolutePath), { recursive: true });
  await writeFile(absolutePath, content);
  return absolutePath;
}

function format(
  root: string,
  paths: readonly string[],
  overrides: Partial<{ searchPath: string; timeoutMilliseconds: number }> = {},
): Promise<readonly string[]> {
  const {
    searchPath = process.env.PATH ?? "",
    timeoutMilliseconds = defaultTimeoutMilliseconds,
  } = overrides;
  return formatEditedFiles(paths, {
    environment: { ...process.env, PATH: searchPath },
    repositoryCommonDirectory: join(root, ".git"),
    timeoutMilliseconds,
  });
}

test("reformats a TypeScript file with the repository oxfmt configuration", async () => {
  const root = await repository();
  const path = await writeFixture(root, "tooling/sample.ts", "const a  =  1\n");

  const report = await format(root, [path]);

  expect(await readFile(path, "utf8")).toBe("const a = 1;\n");
  expect(report).toEqual([
    "format-edited-file: tooling/sample.ts was reformatted with oxfmt; read it again before editing it.",
  ]);
});

test("reformats Markdown with Prettier", async () => {
  const root = await repository();
  const path = await writeFixture(root, "docs/notes.md", "*  item\n");

  const report = await format(root, [path]);

  expect(await readFile(path, "utf8")).toBe("- item\n");
  expect(report).toHaveLength(1);
  expect(report[0]).toContain("docs/notes.md was reformatted with prettier");
});

test("leaves a file excluded by .prettierignore untouched", async () => {
  const root = await repository();
  await writeFixture(root, ".prettierignore", "generated.json\n");
  const path = await writeFixture(root, "generated.json", '{"a":1}\n');

  const report = await format(root, [path]);

  expect(await readFile(path, "utf8")).toBe('{"a":1}\n');
  expect(report).toEqual([]);
});

test("reformats a Fish file under the managed Fish configuration", async () => {
  const root = await repository();
  const path = await writeFixture(
    root,
    "home/.config/fish/functions/greet.fish",
    "function greet\necho hi\nend\n",
  );

  const report = await format(root, [path]);

  expect(await readFile(path, "utf8")).toBe(
    "function greet\n    echo hi\nend\n",
  );
  expect(report[0]).toContain("was reformatted with fish_indent");
});

test("reformats only the edited Rust file, not its child modules", async () => {
  const root = await repository();
  await writeFixture(root, "rustfmt.toml", 'edition = "2024"\n');
  const child = await writeFixture(root, "src/child.rs", "fn  child( ){}\n");
  const path = await writeFixture(
    root,
    "src/lib.rs",
    "mod child;\nfn  main( ){}\n",
  );

  const report = await format(root, [path]);

  expect(await readFile(path, "utf8")).toBe("mod child;\nfn main() {}\n");
  expect(await readFile(child, "utf8")).toBe("fn  child( ){}\n");
  expect(report[0]).toContain("src/lib.rs was reformatted with rustfmt");
});

test("stays silent when the file is already formatted", async () => {
  const root = await repository();
  const path = await writeFixture(root, "tooling/sample.ts", "const a = 1;\n");

  expect(await format(root, [path])).toEqual([]);
});

test("stays silent for a file type without a repository formatter", async () => {
  const root = await repository();
  const path = await writeFixture(root, "init.lua", "local  a=1\n");

  expect(await format(root, [path])).toEqual([]);
  expect(await readFile(path, "utf8")).toBe("local  a=1\n");
});

test("leaves a Git-ignored file untouched", async () => {
  const root = await repository();
  await writeFixture(root, ".gitignore", "ignored/\n");
  const path = await writeFixture(root, "ignored/sample.ts", "const a  =  1\n");

  expect(await format(root, [path])).toEqual([]);
  expect(await readFile(path, "utf8")).toBe("const a  =  1\n");
});

test("leaves a file of another repository untouched", async () => {
  const root = await repository();
  const other = await repository();
  const path = await writeFixture(other, "sample.ts", "const a  =  1\n");

  expect(await format(root, [path])).toEqual([]);
  expect(await readFile(path, "utf8")).toBe("const a  =  1\n");
});

test("leaves a file outside any repository untouched", async () => {
  const root = await repository();
  const outside = await temporaryDirectory();
  const path = await writeFixture(outside, "sample.ts", "const a  =  1\n");

  expect(await format(root, [path])).toEqual([]);
  expect(await readFile(path, "utf8")).toBe("const a  =  1\n");
});

test("reports a file deleted before formatting in one line", async () => {
  const root = await repository();

  const report = await format(root, [join(root, "tooling/removed.ts")]);

  expect(report).toEqual([
    "format-edited-file: tooling/removed.ts was not formatted: the file no longer exists.",
  ]);
});

test("reports a formatter failure in one line and keeps the edit", async () => {
  const root = await repository();
  const path = await writeFixture(root, "tooling/broken.ts", "const = ;\n");

  const report = await format(root, [path]);

  expect(await readFile(path, "utf8")).toBe("const = ;\n");
  expect(report).toHaveLength(1);
  expect(report[0]).toStartWith(
    "format-edited-file: tooling/broken.ts was not formatted: oxfmt failed:",
  );
  expect(report[0]).not.toContain("\n");
});

test("reports a missing formatter in one line", async () => {
  const root = await repository();
  const emptyPath = await temporaryDirectory();
  const path = await writeFixture(
    root,
    "home/.config/fish/config.fish",
    "if true\necho hi\nend\n",
  );

  const report = await format(root, [path], { searchPath: emptyPath });

  expect(report).toEqual([
    "format-edited-file: home/.config/fish/config.fish was not formatted: fish_indent is not installed.",
  ]);
  expect(await readFile(path, "utf8")).toBe("if true\necho hi\nend\n");
});

test("abandons a formatter slower than the timeout and keeps the edit", async () => {
  const root = await repository();
  const slowBin = await temporaryDirectory();
  const slowFormatter = await writeFixture(
    slowBin,
    "fish_indent",
    "#!/usr/bin/env bun\nawait Bun.sleep(5000);\n",
  );
  await chmod(slowFormatter, executableMode);
  const path = await writeFixture(
    root,
    "home/.config/fish/config.fish",
    "if true\necho hi\nend\n",
  );

  const report = await format(root, [path], {
    searchPath: `${slowBin}:${process.env.PATH ?? ""}`,
    timeoutMilliseconds: shortTimeoutMilliseconds,
  });

  expect(report).toEqual([
    "format-edited-file: home/.config/fish/config.fish was not formatted: fish_indent timed out after 300 ms.",
  ]);
  expect(await readFile(path, "utf8")).toBe("if true\necho hi\nend\n");
});

test("reports each edited file of a multi-file patch", async () => {
  const root = await repository();
  const typescript = await writeFixture(root, "a.ts", "const a  =  1\n");
  const markdown = await writeFixture(root, "b.md", "*  item\n");

  const report = await format(root, [typescript, markdown]);

  expect(report).toHaveLength(patchedFileCount);
  expect(report[0]).toContain("a.ts was reformatted with oxfmt");
  expect(report[1]).toContain("b.md was reformatted with prettier");
});

test("leaves the target of a symbolic link leaving the repository untouched", async () => {
  const root = await repository();
  const outside = await temporaryDirectory();
  const target = await writeFixture(outside, "notes.md", "*  item\n");
  const link = join(root, "notes.md");
  await symlink(target, link);

  expect(await format(root, [link])).toEqual([]);
  expect(await readFile(target, "utf8")).toBe("*  item\n");
});
