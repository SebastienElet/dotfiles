import { afterAll, describe, expect, test } from "bun:test";
import { rmSync } from "node:fs";
import { join } from "node:path";
import {
  cleanup,
  commit,
  expectTargets,
  git,
  makefileWithAmbiguousRule,
  newRepository,
  select,
  write,
} from "./ci-smoke-targets-test-support.ts";

afterAll(cleanup);

describe("ci-smoke-targets entry point", () => {
  test("refuses invalid invocation without publishing a matrix", () => {
    const repository = newRepository("invalid-invocation");

    for (const arguments_ of [[], ["missing", "missing"], ["HEAD", "HEAD"]]) {
      const result = select(repository, ...arguments_);
      expect(result.exitCode).not.toBe(0);
      expect(result.stdout).toBe("");
      expect(result.stderr).toContain("ci-smoke-targets:");
    }
  });

  test("returns an empty matrix for unrelated changes", () => {
    const repository = newRepository("unrelated");
    write(repository, "README.md", "text\n");
    commit(repository, "docs");

    expectTargets(repository, []);
  });

  test("returns unique targets in lexical order", () => {
    const repository = newRepository("targets");
    write(
      repository,
      "Makefile",
      [
        "SHARED=value",
        ".PHONY: all",
        "all: alpha beta",
        ".PHONY: alpha",
        "alpha:",
        "\t@echo changed-alpha",
        "\t@echo alpha-again",
        ".PHONY: beta",
        "beta:",
        "\t@echo changed-beta",
        "",
      ].join("\n"),
    );
    commit(repository, "targets");

    expectTargets(repository, ["alpha", "beta"]);
  });

  test("returns one changed target", () => {
    const repository = newRepository("one-target");
    write(
      repository,
      "Makefile",
      "SHARED=value\n.PHONY: all\nall: alpha beta\n.PHONY: alpha\nalpha:\n\t@echo changed-alpha\n.PHONY: beta\nbeta:\n\t@echo beta\n",
    );
    commit(repository, "alpha");

    expectTargets(repository, ["alpha"]);
  });

  test.each([
    ["installer", "install.sh", "#!/usr/bin/env bash\nmake all twice\n"],
    ["workflow", ".github/workflows/test.yml", "name: changed\n"],
    ["selector", "tooling/ci-smoke-targets", "changed\n"],
    ["selector-source", "tooling/ci-smoke-targets.ts", "changed\n"],
    ["selector-parser", "tooling/ci-smoke-targets-makefile.ts", "changed\n"],
  ])("selects all when %s infrastructure changes", (name, path, contents) => {
    const repository = newRepository(`infrastructure-${name}`);
    write(repository, path, contents);
    commit(repository, name);

    expectTargets(repository, ["all"]);
  });

  test.each([
    [
      "assignment",
      "SHARED=changed\n.PHONY: all\nall: alpha beta\n.PHONY: alpha\nalpha:\n\t@echo alpha\n.PHONY: beta\nbeta:\n\t@echo beta\n",
    ],
    [
      "directive",
      "SHARED=value\n.PHONY: all\nall: alpha beta\n.PHONY: alpha\nalpha:\n\t@echo alpha\ninclude shared.mk\n.PHONY: beta\nbeta:\n\t@echo beta\n",
    ],
    [
      "missing-phony",
      "SHARED=value\n.PHONY: all\nall: alpha beta\n.PHONY: alpha\nalpha:\n\t@echo alpha\ngamma:\n\t@echo gamma\n.PHONY: beta\nbeta:\n\t@echo beta\n",
    ],
    [
      "invalid-phony",
      "SHARED=value\n.PHONY: all\nall: alpha beta\n.PHONY: alpha beta\nalpha:\n\t@echo alpha\n.PHONY: beta\nbeta:\n\t@echo beta\n",
    ],
  ])("selects all for ambiguous %s changes", (name, makefile) => {
    const repository = newRepository(`ambiguous-${name}`);
    write(repository, "Makefile", makefile);
    commit(repository, name);

    expectTargets(repository, ["all"]);
  });

  test.each([
    ["multi-target", "alpha beta:"],
    ["pattern", "%.out: %.in"],
    ["special", ".DEFAULT:"],
  ])("selects all when an ambiguous %s rule recipe changes", (name, rule) => {
    const repository = newRepository(`ambiguous-${name}-recipe`);
    write(repository, "Makefile", makefileWithAmbiguousRule(rule, "original"));
    commit(repository, "add ambiguous rule");
    write(repository, "Makefile", makefileWithAmbiguousRule(rule, "changed"));
    commit(repository, "change ambiguous recipe");

    expectTargets(repository, ["all"]);
  });

  test.each(["export SHARED", "unexport SHARED"])(
    "selects all when the global directive %s is added",
    (directive) => {
      const repository = newRepository(`global-${directive.split(" ")[0]}`);
      write(
        repository,
        "Makefile",
        `SHARED=value\n.PHONY: all\nall: alpha beta\n.PHONY: alpha\nalpha:\n\t@echo alpha\n.PHONY: beta\nbeta:\n\t@echo beta\n${directive}\n`,
      );
      commit(repository, "add global directive");

      expectTargets(repository, ["all"]);
    },
  );

  test("selects all when a global assignment continuation changes", () => {
    const repository = newRepository("global-continuation");
    const makefile = (value: string) =>
      `SHARED=value\n.PHONY: all\nall: alpha beta\n.PHONY: alpha\nalpha:\n\t@echo alpha\nGLOBAL = one \\\n  ${value}\n.PHONY: beta\nbeta:\n\t@echo beta\n`;
    write(repository, "Makefile", makefile("original"));
    commit(repository, "add global continuation");
    write(repository, "Makefile", makefile("changed"));
    commit(repository, "change global continuation");

    expectTargets(repository, ["all"]);
  });

  test("selects all when a tab-indented global continuation changes", () => {
    const repository = newRepository("tabbed-global-continuation");
    const makefile = (value: string) =>
      `SHARED=value\n.PHONY: all\nall: alpha beta\n.PHONY: alpha\nalpha:\n\t@echo alpha\nGLOBAL = one \\\n\t${value}\n.PHONY: beta\nbeta:\n\t@echo beta\n`;
    write(repository, "Makefile", makefile("original"));
    commit(repository, "add tabbed global continuation");
    write(repository, "Makefile", makefile("changed"));
    commit(repository, "change tabbed global continuation");

    expectTargets(repository, ["all"]);
  });

  test("selects all when a comment starts a logical-line continuation", () => {
    const repository = newRepository("comment-continuation");
    const makefile = (suffix: string) =>
      `SHARED=value\n.PHONY: all\nall: alpha beta\n.PHONY: alpha\nalpha:\n\t@echo alpha\n# shared${suffix}\n.PHONY: beta\nbeta:\n\t@echo beta\n`;
    write(repository, "Makefile", makefile(""));
    commit(repository, "add comment");
    write(repository, "Makefile", makefile(" \\"));
    commit(repository, "continue comment");

    expectTargets(repository, ["all"]);
  });

  test("selects all when an inline comment continues a logical line", () => {
    const repository = newRepository("inline-comment-continuation");
    const makefile = (suffix: string) =>
      `SHARED=value\n.PHONY: all\nall: alpha beta\n.PHONY: alpha\nalpha:\n\t@echo alpha\nalpha: # shared${suffix}\ninclude shared.mk\n.PHONY: beta\nbeta:\n\t@echo beta\n`;
    write(repository, "Makefile", makefile(""));
    commit(repository, "add inline comment");
    write(repository, "Makefile", makefile(" \\"));
    commit(repository, "continue inline comment");

    expectTargets(repository, ["all"]);
  });

  test("selects all when a tab-indented define body changes", () => {
    const repository = newRepository("define-body");
    const makefile = (value: string) =>
      `SHARED=value\n.PHONY: all\nall: alpha beta\n.PHONY: alpha\nalpha:\n\t@echo alpha\ndefine SHARED_RECIPE\n\t${value}\nendef\n.PHONY: beta\nbeta:\n\t@echo beta\n`;
    write(repository, "Makefile", makefile("original"));
    commit(repository, "add define body");
    write(repository, "Makefile", makefile("changed"));
    commit(repository, "change define body");

    expectTargets(repository, ["all"]);
  });

  test("selects all when a continued define body changes", () => {
    const repository = newRepository("continued-define-body");
    const makefile = (value: string) =>
      `SHARED=value\n.PHONY: all\nall: alpha beta\n.PHONY: alpha\nalpha:\n\t@echo alpha\ndefine GLOBAL\none \\\nendef\n\t${value}\nendef\n.PHONY: beta\nbeta:\n\t@echo beta\n`;
    write(repository, "Makefile", makefile("original"));
    commit(repository, "add continued define body");
    write(repository, "Makefile", makefile("changed"));
    commit(repository, "change continued define body");

    expectTargets(repository, ["all"]);
  });

  test("selects all when a target is deleted", () => {
    const repository = newRepository("deleted-target");
    write(
      repository,
      "Makefile",
      "SHARED=value\n.PHONY: all\nall: alpha\n.PHONY: alpha\nalpha:\n\t@echo alpha\n",
    );
    commit(repository, "delete beta");

    expectTargets(repository, ["all"]);
  });

  test("selects all when the installer is renamed", () => {
    const repository = newRepository("renamed-installer");
    git(repository, "mv", "install.sh", "setup.sh");
    commit(repository, "rename installer");

    expectTargets(repository, ["all"]);
  });

  test("selects all when Makefile evidence is unavailable", () => {
    const repository = newRepository("missing-makefile");
    rmSync(join(repository, "Makefile"));
    commit(repository, "delete Makefile");

    const result = select(repository, "HEAD^", "HEAD");
    expect(result.exitCode).toBe(0);
    expect(result.stdout).toBe('["all"]\n');
    expect(result.stderr).toContain("does not exist in");
  });

  test("fails closed when Git has no merge base", () => {
    const repository = newRepository("no-merge-base");
    const base = git(repository, "rev-parse", "HEAD");
    git(repository, "checkout", "-q", "--orphan", "unrelated");
    rmSync(join(repository, "Makefile"));
    rmSync(join(repository, "install.sh"));
    write(repository, "README.md", "unrelated history\n");
    const head = commit(repository, "unrelated history");

    const result = select(repository, base, head);
    expect(result.exitCode).not.toBe(0);
    expect(result.stdout).toBe("");
  });
});
