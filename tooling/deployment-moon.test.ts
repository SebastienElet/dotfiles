import { afterEach, test } from "bun:test";
import {
  cleanupMoonDeploymentFixtures,
  createMoonDeploymentFixture,
  foreignGitHubEnvironment,
  runMoon,
} from "./deployment-moon-test-support.ts";
import { expectSuccess } from "./deployment-test-support.ts";

afterEach(cleanupMoonDeploymentFixtures);

test.each([
  { source: "GitHub", environment: foreignGitHubEnvironment },
  {
    source: "caller",
    environment: {
      ...foreignGitHubEnvironment,
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
