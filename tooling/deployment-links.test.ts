import { afterEach, expect, setDefaultTimeout, test } from "bun:test";
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

afterEach(cleanupDeploymentFixtures);

const deploymentTimeoutMilliseconds = 15_000;
const extendedDeploymentTimeoutMilliseconds = 30_000;
setDefaultTimeout(deploymentTimeoutMilliseconds);
const userSkillDestinations = [
  [".agents", "agent-instructions"],
  ...[
    "obsidian-retrieval",
    "code-search",
    "enforcement-code",
    "harness-reflection",
    "handoff",
    "issue-creation",
    "linear-issue-spec",
    "linear-sync",
    "memory-governance",
    "pr-fix",
    "pr-feedback",
    "pr-verdict",
    "requirements-clarification",
    "skill-manager",
    "workflow-automation",
  ].map((slug) => [".claude", slug]),
  ...[
    "claude-developer",
    "obsidian-retrieval",
    "code-search",
    "enforcement-code",
    "harness-reflection",
    "issue-creation",
    "linear-issue-spec",
    "linear-sync",
    "memory-governance",
    "pr-fix",
    "pr-feedback",
    "pr-verdict",
    "requirements-clarification",
    "skill-manager",
    "workflow-automation",
  ].map((slug) => [".cursor", slug]),
  ...[
    "claude-developer",
    "obsidian-retrieval",
    "code-search",
    "design-claim-audit",
    "enforcement-code",
    "harness-reflection",
    "handoff",
    "issue-creation",
    "linear-issue-spec",
    "linear-sync",
    "memory-governance",
    "pr-fix",
    "pr-feedback",
    "pr-verdict",
    "requirements-clarification",
    "skill-manager",
    "workflow-automation",
  ].map((slug) => [".agents", slug] as const),
] as const;

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

test(
  "deploys Starship and tmux links, replays idempotently, and preserves a wrong link",
  () => {
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
        environment: {
          DEPLOYMENT_MARKER: marker,
          DEPLOYMENT_PROVIDER_MODE: "ln",
          DEPLOYMENT_REAL_COMMAND: requireCommand("ln"),
          PATH: `${fixture.bin}:${process.env.PATH ?? ""}`,
        },
        repository: project,
      }),
    );
    expect(pathExists(marker)).toBeFalse();

    expectDivergentSymlinkRejected(fixture, starship);
  },
  deploymentTimeoutMilliseconds,
);

test("deploys the guarded ColGrep entry point without replacing a destination", () => {
  const fixture = createDeploymentFixture("colgrep-search");
  const destination = join(fixture.home, ".local", "bin", "colgrep-search");

  expectSuccess(runMake(fixture, [destination], { repository: project }));
  expect(linkTarget(destination)).toBe(
    join(project, "tooling", "colgrep-search-cli.ts"),
  );
  expectSuccess(runMake(fixture, [destination], { repository: project }));

  unlinkSync(destination);
  writeFileSync(destination, "keep\n");
  const divergent = runMake(fixture, [destination], { repository: project });
  expect(divergent.exitCode).not.toBe(0);
  expect(readFileSync(destination, "utf8")).toBe("keep\n");
});

function expectDivergentSymlinkRejected(
  fixture: ReturnType<typeof createDeploymentFixture>,
  destination: string,
): void {
  unlinkSync(destination);
  const unexpected = join(fixture.root, "unexpected");
  mkdirSync(unexpected);
  symlinkSync(unexpected, destination);
  const result = runMake(fixture, [destination], { repository: project });
  expect(result.exitCode).not.toBe(0);
  expect(result.stderr).toContain(
    `exists and is not the expected symbolic link`,
  );
  expect(linkTarget(destination)).toBe(unexpected);
}

test("deploys shared instructions and skills, rejects divergent rules, and replays idempotently", () => {
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
  expect(readFileSync(codexInstructions, "utf8")).toBe(
    expectedCodexInstructions(),
  );
  const before = fileIdentity(codexInstructions);
  expectSuccess(
    runMake(fixture, [claudeRule, codexInstructions], {
      repository: project,
    }),
  );
  expect(fileIdentity(codexInstructions)).toEqual(before);
  unlinkSync(claudeRule);
  writeFileSync(claudeRule, "keep\n");
  const divergent = runMake(fixture, [claudeRule], { repository: project });
  expect(divergent.exitCode).not.toBe(0);
  expect(divergent.stderr).toContain(
    "exists and is not the expected symbolic link",
  );
  expect(readFileSync(claudeRule, "utf8")).toBe("keep\n");
});

function expectedCodexInstructions(): string {
  return (
    readFileSync(join(project, "harness", "AGENTS.md"), "utf8").replaceAll(
      /^@.*\n/gmu,
      "",
    ) +
    readFileSync(join(project, "harness", "SOUL.md"), "utf8") +
    readFileSync(join(project, "harness", "USER.md"), "utf8")
  );
}

test(
  "deploys every public user skill from the shared collection",
  () => {
    const fixture = createDeploymentFixture("user-skills");
    const destinationPaths: string[] = [];
    for (const [owner, slug] of userSkillDestinations) {
      destinationPaths.push(join(fixture.home, owner, "skills", slug));
    }
    expectSuccess(runMake(fixture, destinationPaths, { repository: project }));
    for (const [owner, slug] of userSkillDestinations) {
      const destination = join(fixture.home, owner, "skills", slug);
      expect(linkTarget(destination)).toBe(
        join(project, "harness", "skills", slug),
      );
      expect(pathExists(join(project, ".agents", "skills", slug))).toBeFalse();
    }
  },
  extendedDeploymentTimeoutMilliseconds,
);
test("agent aggregate targets deploy memory governance to every agent", () => {
  const fixture = createDeploymentFixture("memory-governance");
  const codexDestination = join(
    fixture.home,
    ".agents/skills/memory-governance",
  );
  const claudeDestination = join(
    fixture.home,
    ".claude/skills/memory-governance",
  );
  const cursorDestination = join(
    fixture.home,
    ".cursor/skills/memory-governance",
  );
  for (const [target, included] of [
    ["codex", codexDestination],
    ["claude-code", claudeDestination],
    ["cursor", cursorDestination],
  ] as const) {
    const result = runMake(fixture, [target], {
      dryRun: true,
      repository: project,
    });
    expectSuccess(result);
    expect(result.stdout).toContain(included);
  }
});
