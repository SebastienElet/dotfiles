import { afterEach, test } from "bun:test";
import {
  cleanupMoonDeploymentFixtures,
  createMoonDeploymentFixture,
  runMoon,
} from "./deployment-moon-test-support.ts";
import { expectSuccess } from "./deployment-test-support.ts";

afterEach(cleanupMoonDeploymentFixtures);

const githubEnvironment = {
  CI: "true",
  GITHUB_ACTIONS: "true",
  GITHUB_BASE_REF: "codex/moon-retire-obsidian-test",
  GITHUB_HEAD_REF: "codex/moon-retire-global-cspell",
  GITHUB_REF: "refs/pull/379/merge",
  GITHUB_SHA: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
};

test.each([
  { source: "GitHub", environment: githubEnvironment },
  {
    source: "caller",
    environment: {
      ...githubEnvironment,
      MOON_BASE: "foreign-base",
      MOON_HEAD: "foreign-head",
    },
  },
])(
  "executes fixture tasks with foreign $source references",
  ({ environment }: Readonly<{ environment: Readonly<NodeJS.ProcessEnv> }>) => {
    const fixture = createMoonDeploymentFixture("agent-memory");

    expectSuccess(
      runMoon(fixture, "repository:rust", { cache: "off", environment }),
    );
  },
);
