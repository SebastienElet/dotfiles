import { afterEach, expect, test } from "bun:test";
import {
  cleanupDeploymentFixtures,
  createDeploymentFixture,
  expectSuccess,
  installProvider,
  linkTarget,
  runMake,
} from "./deployment-test-support.ts";
import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";

afterEach(cleanupDeploymentFixtures);

test("provisions locked dependencies before deploying skill-manager", () => {
  const fixture = createDeploymentFixture("skill-manager-dependencies");
  const source = join(fixture.repository, "harness", "skills", "skill-manager");
  const destination = join(fixture.home, ".agents", "skills", "skill-manager");
  const marker = join(fixture.root, "bun-called");
  mkdirSync(source, { recursive: true });
  writeFileSync(join(fixture.repository, "package.json"), "{}\n");
  writeFileSync(join(fixture.repository, "bun.lock"), "");
  installProvider(fixture, "brew");
  installProvider(fixture, "bun");

  expectSuccess(
    runMake(fixture, [destination], {
      environment: {
        DEPLOYMENT_MARKER: marker,
        DEPLOYMENT_PROVIDER_MODE: "bun-install",
      },
      variables: { BREW_BIN: fixture.bin },
    }),
  );

  expect(readFileSync(marker, "utf8")).toBe(
    "--config=/dev/null --no-env-file install --frozen-lockfile --ignore-scripts\n",
  );
  expect(
    existsSync(join(fixture.repository, "node_modules", "zod", "package.json")),
  ).toBeTrue();
  expect(linkTarget(destination)).toBe(source);
});
