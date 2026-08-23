import { afterEach, describe, expect, test } from "bun:test";
import {
  chmodSync,
  mkdirSync,
  readFileSync,
  statSync,
  symlinkSync,
  writeFileSync,
} from "node:fs";
import { join } from "node:path";
import {
  cleanupDeploymentFixtures,
  createDeploymentFixture,
  expectSuccess,
  fileIdentity,
  linkTarget,
  project,
  runMake,
} from "./deployment-test-support.ts";

afterEach(cleanupDeploymentFixtures);

describe("deployment area: daily routine", () => {
  test("copies a private default once and preserves a local configuration", () => {
    const fixture = createDeploymentFixture("daily-routine");
    const config = join(
      fixture.home,
      ".config",
      "daily-routine",
      "config.toml",
    );
    expectSuccess(runMake(fixture, [config], { repository: project }));
    expect(readFileSync(config, "utf8")).toBe(
      readFileSync(
        join(project, "tooling", "daily-routine", "config.example.toml"),
        "utf8",
      ),
    );
    expect(statSync(config).mode & 0o777).toBe(0o600);
    writeFileSync(config, "local configuration\n");
    chmodSync(config, 0o600);
    const before = fileIdentity(config);
    expectSuccess(runMake(fixture, [config], { repository: project }));
    expect(fileIdentity(config)).toEqual(before);
  });

  test("refuses a dangling destination link without replacing it", () => {
    const fixture = createDeploymentFixture("daily-routine-dangling");
    const config = join(
      fixture.home,
      ".config",
      "daily-routine",
      "config.toml",
    );
    mkdirSync(join(fixture.home, ".config", "daily-routine"), {
      recursive: true,
    });
    const missing = join(fixture.root, "missing");
    symlinkSync(missing, config);
    const result = runMake(fixture, [config], { repository: project });
    expect(result.exitCode).not.toBe(0);
    expect(linkTarget(config)).toBe(missing);
  });
});
