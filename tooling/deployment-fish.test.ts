import { afterEach, expect, setDefaultTimeout, test } from "bun:test";
import {
  cleanupDeploymentFixtures,
  createDeploymentFixture,
  expectSuccess,
  installProvider,
  linkTarget,
  runMake,
} from "./deployment-test-support.ts";
import { existsSync, mkdirSync, readFileSync, rmSync } from "node:fs";
import { join } from "node:path";

afterEach(cleanupDeploymentFixtures);
const deploymentTimeoutMilliseconds = 15_000;
setDefaultTimeout(deploymentTimeoutMilliseconds);

test("installs through the real Make target and replays without invoking Fish", () => {
  const fixture = createDeploymentFixture("fish-success");
  const source = join(fixture.repository, "home", ".config", "fish");
  const bindings = join(
    fixture.home,
    ".config",
    "fish",
    "functions",
    "fzf_configure_bindings.fish",
  );
  const marker = join(fixture.root, "fish-called");
  mkdirSync(join(fixture.home, ".config"), { recursive: true });
  mkdirSync(source, { recursive: true });
  installProvider(fixture, "fish");
  const environment = {
    DEPLOYMENT_MARKER: marker,
    DEPLOYMENT_PROVIDER_MODE: "fish-success",
    PATH: `${fixture.bin}:${process.env.PATH ?? ""}`,
  };

  expectSuccess(
    runMake(fixture, [bindings], {
      environment,
      variables: { BREW_BIN: fixture.bin },
    }),
  );
  expect(linkTarget(join(fixture.home, ".config", "fish"))).toBe(source);
  expect(existsSync(bindings)).toBeTrue();
  expect(readFileSync(marker, "utf8")).toBe(
    "-c fisher install PatrickF1/fzf.fish\n",
  );

  rmSync(marker);
  expectSuccess(
    runMake(fixture, [bindings], {
      environment,
      variables: { BREW_BIN: fixture.bin },
    }),
  );
  expect(existsSync(marker)).toBeFalse();
});

test("fails explicitly when Fish returns success without producing bindings", () => {
  const fixture = createDeploymentFixture("fish-empty");
  const source = join(fixture.repository, "home", ".config", "fish");
  const bindings = join(
    fixture.home,
    ".config",
    "fish",
    "functions",
    "fzf_configure_bindings.fish",
  );
  mkdirSync(join(fixture.home, ".config"), { recursive: true });
  mkdirSync(source, { recursive: true });
  installProvider(fixture, "fish");
  const result = runMake(fixture, [bindings], {
    environment: {
      DEPLOYMENT_PROVIDER_MODE: "fish-empty",
      PATH: `${fixture.bin}:${process.env.PATH ?? ""}`,
    },
    variables: { BREW_BIN: fixture.bin },
  });
  expect(result.exitCode).not.toBe(0);
  expect(result.stderr).toContain(`Error: Fisher did not install ${bindings}`);
});
