import { afterEach, expect, test } from "bun:test";
import {
  cleanupDeploymentFixtures,
  createDeploymentFixture,
  project,
} from "./deployment-test-support.ts";
import { lstatSync, readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { z } from "zod";

const pathsSchema = z.array(z.string().min(1)).readonly();
// Git's tracked text exposes literal commands, not shell aliases or dynamically constructed argv.
function literalArgument(value: string): string {
  return `(?:${value}|"${value}"|'${value}')`;
}

const arnesExecutable = String.raw`(?:"(?:[^"\r\n]*/)?arnes"|'(?:[^'\r\n]*/)?arnes'|(?:[^\s"']*/)?arnes)`;
const environmentPrefix = String.raw`(?:env[ \t]+(?:(?:-u|--unset)[ \t]+[A-Za-z_]\w*[ \t]+)*)?(?:[A-Za-z_]\w*=(?:"[^"\r\n]*"|'[^'\r\n]*'|[^ \t\r\n"']+)[ \t]+)*`;
const writer = new RegExp(
  String.raw`(?:^|[;&|({,])[ \t]*(?:-[ \t]+)?(?:(?:["']?(?:script|command|run)["']?):[ \t]*)?['"]?${environmentPrefix}${arnesExecutable}[ \t]+(?:${literalArgument("sync")}|${literalArgument("setup")}[ \t]+${literalArgument("hooks")}|${literalArgument("measure")}[ \t]+(?:hook|finish|feedback|outcome|pr-verdict)|${literalArgument("eval")}[ \t]+run)(?=[ \t;|&"']|$)`,
  "gmu",
);

function arnesWriters(repository: string): readonly string[] {
  const result = Bun.spawnSync(["git", "ls-files", "-z"], {
    cwd: repository,
    stdout: "pipe",
    stderr: "pipe",
  });
  if (result.exitCode !== 0) {
    throw new Error(new TextDecoder().decode(result.stderr));
  }
  const output = new TextDecoder("utf-8", { fatal: true }).decode(
    result.stdout,
  );
  if (!output.endsWith("\0")) {
    throw new Error("Missing tracked file inventory");
  }
  const paths = pathsSchema.parse(output.slice(0, -1).split("\0"));
  return paths.flatMap((path) => {
    const absolute = join(repository, path);
    if (!lstatSync(absolute).isFile()) {
      return [];
    }
    const bytes = readFileSync(absolute);
    if (bytes.includes(0)) {
      return [];
    }
    const text = bytes.toString("utf8").replaceAll(/\\\r?\n/gu, " ");
    return [...text.matchAll(writer)].map(
      (match: readonly string[]) => `${path}: ${match[0]?.trim()}`,
    );
  });
}

function trackedFixture(contents: string): string {
  const { repository } = createDeploymentFixture("arnes-read-only");
  expect(Bun.spawnSync(["git", "init", "--quiet", repository]).exitCode).toBe(
    0,
  );
  writeFileSync(join(repository, "caller"), contents);
  expect(
    Bun.spawnSync(["git", "add", "caller"], { cwd: repository }).exitCode,
  ).toBe(0);
  return repository;
}

afterEach(cleanupDeploymentFixtures);

test("tracked files contain no literal invocation of Arnes writers", () => {
  const tracked = Bun.spawnSync(
    ["git", "ls-files", "--error-unmatch", "moon.yml"],
    { cwd: project, stdout: "pipe", stderr: "pipe" },
  );
  expect(tracked.exitCode).toBe(0);
  expect(arnesWriters(project)).toEqual([]);
});

test.each([
  ["arnes", "sync skills --agent codex"],
  ["arnes", '"sync" skills --agent codex'],
  ["- run: arnes", "sync skills"],
  ["run: arnes", "sync skills"],
  ["- arnes", "sync skills"],
  ["ARNES_SOURCE_ROOT=/tmp arnes", "setup hooks --agent claude"],
  ["env ARNES_SOURCE_ROOT=/tmp arnes", "sync skills"],
  ["env -u VOLTA_HOME arnes", "sync skills"],
  ['"$HOME/.local/bin/arnes"', "setup hooks --agent claude"],
  ["'/tmp/path with spaces/arnes'", "measure hook --agent cursor"],
  ["arnes", "measure outcome RUN --status pass"],
  ["arnes", "eval run"],
  ["arnes \\\n", "setup hooks"],
])("rejects a tracked literal writer: %s %s", (executable, arguments_) => {
  const repository = trackedFixture(`${executable} ${arguments_}`);
  expect(arnesWriters(repository)).toHaveLength(1);
});

test("permits read-only diagnostics and isolated exports", () => {
  const repository = trackedFixture(
    ["arnes doctor", "arnes export", "arnes export --check"].join("\n"),
  );
  expect(arnesWriters(repository)).toEqual([]);
});

test("does not assemble separate commands into a writer invocation", () => {
  const repository = trackedFixture(
    "echo arnes\nsync unrelated\necho arnes sync skills",
  );
  expect(arnesWriters(repository)).toEqual([]);
});

test("refuses an unavailable or empty Git inventory", () => {
  const { repository } = createDeploymentFixture("missing-git-inventory");
  expect(() => arnesWriters(repository)).toThrow();
  expect(Bun.spawnSync(["git", "init", "--quiet", repository]).exitCode).toBe(
    0,
  );
  expect(() => arnesWriters(repository)).toThrow(
    "Missing tracked file inventory",
  );
});
