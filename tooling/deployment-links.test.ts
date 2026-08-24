import { afterEach, describe, expect, test } from "bun:test";
import {
  mkdirSync,
  readFileSync,
  readdirSync,
  symlinkSync,
  unlinkSync,
  utimesSync,
  writeFileSync,
} from "node:fs";
import { join } from "node:path";
import {
  cleanupDeploymentFixtures,
  createDeploymentFixture,
  expectSuccess,
  fileIdentity,
  installProvider,
  linkTarget,
  pathExists,
  project,
  requireCommand,
  runMake,
} from "./deployment-test-support.ts";

afterEach(cleanupDeploymentFixtures);

describe("deployment area: managed links", () => {
  test("refuses existing directories without linking inside them", () => {
    const fixture = createDeploymentFixture("existing-directory");
    const source = join(fixture.repository, "home", ".config", "fish");
    const destination = join(fixture.home, ".config", "fish");
    mkdirSync(source, { recursive: true });
    mkdirSync(destination, { recursive: true });
    utimesSync(destination, new Date("2020-01-01"), new Date("2020-01-01"));
    utimesSync(source, new Date("2021-01-01"), new Date("2021-01-01"));

    const result = runMake(fixture, [destination]);

    expect(result.exitCode).not.toBe(0);
    expect(readdirSync(destination)).toEqual([]);
  });

  test("refuses a destination symlink to a directory without mutating it", () => {
    const fixture = createDeploymentFixture("directory-link");
    const source = join(fixture.repository, "home", ".config", "fish");
    const destination = join(fixture.home, ".config", "fish");
    const actual = join(fixture.root, "actual");
    mkdirSync(source, { recursive: true });
    mkdirSync(join(fixture.home, ".config"), { recursive: true });
    mkdirSync(actual);
    symlinkSync(actual, destination);
    utimesSync(actual, new Date("2020-01-01"), new Date("2020-01-01"));
    utimesSync(source, new Date("2021-01-01"), new Date("2021-01-01"));

    const result = runMake(fixture, [destination]);

    expect(result.exitCode).not.toBe(0);
    expect(linkTarget(destination)).toBe(actual);
    expect(readdirSync(actual)).toEqual([]);
  });

  test("deploys Starship and tmux links, replays idempotently, and preserves a wrong link", () => {
    const fixture = createDeploymentFixture("starship");
    const tmux = join(fixture.home, ".config", "tmux", "tmux.conf");
    const starship = join(fixture.home, ".config", "starship.toml");
    expectSuccess(runMake(fixture, [tmux], { repository: project }));
    expect(linkTarget(tmux)).toBe(
      join(project, "home", ".config", "tmux", "tmux.conf"),
    );
    expectSuccess(runMake(fixture, [starship], { repository: project }));
    expect(linkTarget(starship)).toBe(
      join(project, "home", ".config", "starship.toml"),
    );

    const marker = join(fixture.root, "ln-called");
    installProvider(fixture, "ln");
    expectSuccess(
      runMake(fixture, [starship], {
        repository: project,
        environment: {
          PATH: `${fixture.bin}:${process.env.PATH ?? ""}`,
          DEPLOYMENT_PROVIDER_MODE: "ln",
          DEPLOYMENT_MARKER: marker,
          DEPLOYMENT_REAL_COMMAND: requireCommand("ln"),
        },
      }),
    );
    expect(pathExists(marker)).toBeFalse();

    unlinkSync(starship);
    const unexpected = join(fixture.root, "unexpected");
    mkdirSync(unexpected);
    symlinkSync(unexpected, starship);
    expectSuccess(runMake(fixture, [starship], { repository: project }));
    expect(linkTarget(starship)).toBe(unexpected);
  });
});

describe("deployment area: agent instructions and skills", () => {
  test("deploys shared instructions and skills, preserves local rules, and replays idempotently", () => {
    const fixture = createDeploymentFixture("agent-instructions");
    const claudeRule = join(
      fixture.home,
      ".claude",
      "rules",
      "agent-instructions.md",
    );
    const codexInstructions = join(fixture.home, ".codex", "AGENTS.md");
    const codexSkill = join(
      fixture.home,
      ".agents",
      "skills",
      "agent-instructions",
    );
    expectSuccess(
      runMake(fixture, [claudeRule, codexInstructions, codexSkill], {
        repository: project,
      }),
    );
    expect(
      linkTarget(join(project, "harness", "rules", "agent-instructions.md")),
    ).toBe("../skills/agent-instructions/references/maintenance.md");
    expect(linkTarget(claudeRule)).toBe(
      join(project, "harness", "rules", "agent-instructions.md"),
    );
    expect(linkTarget(codexSkill)).toBe(
      join(project, "harness", "skills", "agent-instructions"),
    );
    const expected =
      readFileSync(join(project, "harness", "AGENTS.md"), "utf8").replace(
        /^@.*\n/gm,
        "",
      ) +
      readFileSync(join(project, "harness", "SOUL.md"), "utf8") +
      readFileSync(join(project, "harness", "USER.md"), "utf8");
    expect(readFileSync(codexInstructions, "utf8")).toBe(expected);
    const before = fileIdentity(codexInstructions);
    expectSuccess(
      runMake(fixture, [claudeRule, codexInstructions], {
        repository: project,
      }),
    );
    expect(fileIdentity(codexInstructions)).toEqual(before);
    unlinkSync(claudeRule);
    writeFileSync(claudeRule, "keep\n");
    expectSuccess(runMake(fixture, [claudeRule], { repository: project }));
    expect(readFileSync(claudeRule, "utf8")).toBe("keep\n");
  });

  test("deploys every public user skill from the shared collection", () => {
    const fixture = createDeploymentFixture("user-skills");
    const destinations = [
      [".agents", "agent-instructions"],
      ...[
        "obsidian-retrieval",
        "codegraph",
        "enforcement-code",
        "harness-reflection",
        "handoff",
        "issue-creation",
        "linear-issue-spec",
        "linear-sync",
        "pr-fix",
        "pr-verdict",
        "skill-manager",
        "workflow-automation",
      ].map((slug) => [".claude", slug]),
      ...[
        "claude-developer",
        "obsidian-retrieval",
        "codegraph",
        "enforcement-code",
        "harness-reflection",
        "issue-creation",
        "linear-issue-spec",
        "linear-sync",
        "pr-fix",
        "pr-verdict",
        "skill-manager",
        "workflow-automation",
      ].map((slug) => [".cursor", slug]),
      ...[
        "claude-developer",
        "obsidian-retrieval",
        "codegraph",
        "enforcement-code",
        "harness-reflection",
        "handoff",
        "issue-creation",
        "linear-issue-spec",
        "linear-sync",
        "pr-fix",
        "pr-verdict",
        "skill-manager",
        "workflow-automation",
      ].map((slug) => [".agents", slug]),
    ] as const;
    const destinationPaths = destinations.map(([owner, slug]) =>
      join(fixture.home, owner, "skills", slug),
    );
    expectSuccess(runMake(fixture, destinationPaths, { repository: project }));
    for (const [owner, slug] of destinations) {
      const destination = join(fixture.home, owner, "skills", slug);
      expect(linkTarget(destination)).toBe(
        join(project, "harness", "skills", slug),
      );
      expect(pathExists(join(project, ".agents", "skills", slug))).toBeFalse();
    }
  }, 30_000);
});
