import { afterAll, expect, test } from "bun:test";
import {
  cleanupDockerInstallFixtures,
  runDockerInstallTarget,
} from "./docker-install-test-support.ts";
import { dockerImages } from "./docker-image.ts";

afterAll(cleanupDockerInstallFixtures);

const sha256HexadecimalLength = 64;
const verified = "docker-install target=scrapling result=verified";
const skipped = "docker-install target=scrapling result=skipped";

test("Scrapling verification inspects the default image without pulling or deploying", () => {
  const result = runDockerInstallTarget("scrapling", "artifact-present", {
    action: "verify",
  });

  expect(result.exitCode).toBe(0);
  expect(result.stdout).toContain(verified);
  expect(result.trace).toBe(
    `info\nimage inspect --format {{json .}} -- ${dockerImages.scrapling}\n`,
  );
  expect(result.scraplingLinkExists).toBe(false);
});

test("Scrapling verification fails on an absent image without pulling or deploying", () => {
  const result = runDockerInstallTarget("scrapling", "artifact-absent", {
    action: "verify",
  });

  expect(result.exitCode).not.toBe(0);
  expect(result.stdout).not.toContain(verified);
  expect(result.trace).toBe(
    `info\nimage inspect --format {{json .}} -- ${dockerImages.scrapling}\n`,
  );
  expect(result.scraplingLinkExists).toBe(false);
});

test("Scrapling verification honors an explicit image", () => {
  const result = runDockerInstallTarget("scrapling", "artifact-present", {
    action: "verify",
    imageOverride: `registry.example/scrapling:custom@sha256:${"a".repeat(sha256HexadecimalLength)}`,
  });

  expect(result.exitCode).toBe(0);
  expect(result.stdout).toContain(verified);
  expect(result.trace).toBe(
    `info\nimage inspect --format {{json .}} -- registry.example/scrapling:custom@sha256:${"a".repeat(sha256HexadecimalLength)}\n`,
  );
  expect(result.scraplingLinkExists).toBe(false);
});

test.each([undefined, "require-docker", "allow-skip"])(
  "Scrapling verification handles an unavailable daemon under policy %s",
  (policy) => {
    const result = runDockerInstallTarget("scrapling", "daemon-unavailable", {
      action: "verify",
      ...(policy === undefined ? {} : { policy }),
    });

    expect(result.exitCode === 0).toBe(policy === "allow-skip");
    expect(result.stdout.includes(skipped)).toBe(policy === "allow-skip");
    expect(result.stdout).not.toContain(verified);
    expect(result.trace).toBe("info\n");
    expect(result.scraplingLinkExists).toBe(false);
  },
);

test("Scrapling verification requires the Docker CLI even when skips are allowed", () => {
  const result = runDockerInstallTarget("scrapling", "artifact-present", {
    action: "verify",
    policy: "allow-skip",
    dockerProviderAvailable: false,
  });

  expect(result.exitCode).not.toBe(0);
  expect(result.stderr).toContain("Docker CLI unavailable");
  expect(result.stdout).not.toContain(skipped);
  expect(result.trace).toBe("");
  expect(result.scraplingLinkExists).toBe(false);
});

test.each(["", "unknown-policy"])(
  "Scrapling verification rejects policy %s before the Docker API",
  (policy) => {
    const result = runDockerInstallTarget("scrapling", "artifact-present", {
      action: "verify",
      policy,
    });

    expect(result.exitCode).not.toBe(0);
    expect(result.stdout).not.toContain(verified);
    expect(result.trace).toBe("");
    expect(result.scraplingLinkExists).toBe(false);
  },
);

test.each([
  "",
  "--help",
  "image with spaces",
  `image@sha256:${"a".repeat(sha256HexadecimalLength - 1)}`,
])(
  "Scrapling verification rejects image %s before the Docker API",
  (imageOverride) => {
    const result = runDockerInstallTarget("scrapling", "artifact-present", {
      action: "verify",
      imageOverride,
    });

    expect(result.exitCode).not.toBe(0);
    expect(result.stdout).not.toContain(verified);
    expect(result.trace).toBe("");
    expect(result.scraplingLinkExists).toBe(false);
  },
);

const cloakbrowserVerified =
  "docker-install target=cloakbrowser result=verified";
const cloakbrowserSkipped = "docker-install target=cloakbrowser result=skipped";

test("CloakBrowser verification inspects the default image without pulling or deploying", () => {
  const result = runDockerInstallTarget("cloakbrowser", "artifact-present", {
    action: "verify",
  });

  expect(result.exitCode).toBe(0);
  expect(result.stdout).toContain(cloakbrowserVerified);
  expect(result.trace).toBe(
    `info\nimage inspect --format {{json .}} -- ${dockerImages.cloakbrowser}\n`,
  );
  expect(result.scraplingLinkExists).toBe(false);
});

test("CloakBrowser verification fails on an absent image without pulling or deploying", () => {
  const result = runDockerInstallTarget("cloakbrowser", "artifact-absent", {
    action: "verify",
  });

  expect(result.exitCode).not.toBe(0);
  expect(result.stdout).not.toContain(cloakbrowserVerified);
  expect(result.trace).toBe(
    `info\nimage inspect --format {{json .}} -- ${dockerImages.cloakbrowser}\n`,
  );
  expect(result.scraplingLinkExists).toBe(false);
});

test("CloakBrowser verification honors an explicit image", () => {
  const result = runDockerInstallTarget("cloakbrowser", "artifact-present", {
    action: "verify",
    imageOverride: `registry.example/cloakbrowser:custom@sha256:${"a".repeat(sha256HexadecimalLength)}`,
  });

  expect(result.exitCode).toBe(0);
  expect(result.stdout).toContain(cloakbrowserVerified);
  expect(result.trace).toBe(
    `info\nimage inspect --format {{json .}} -- registry.example/cloakbrowser:custom@sha256:${"a".repeat(sha256HexadecimalLength)}\n`,
  );
  expect(result.scraplingLinkExists).toBe(false);
});

test.each([undefined, "require-docker", "allow-skip"])(
  "CloakBrowser verification handles an unavailable daemon under policy %s",
  (policy) => {
    const result = runDockerInstallTarget(
      "cloakbrowser",
      "daemon-unavailable",
      {
        action: "verify",
        ...(policy === undefined ? {} : { policy }),
      },
    );

    expect(result.exitCode === 0).toBe(policy === "allow-skip");
    expect(result.stdout.includes(cloakbrowserSkipped)).toBe(
      policy === "allow-skip",
    );
    expect(result.stdout).not.toContain(cloakbrowserVerified);
    expect(result.trace).toBe("info\n");
    expect(result.scraplingLinkExists).toBe(false);
  },
);

test("CloakBrowser verification requires the Docker CLI even when skips are allowed", () => {
  const result = runDockerInstallTarget("cloakbrowser", "artifact-present", {
    action: "verify",
    policy: "allow-skip",
    dockerProviderAvailable: false,
  });

  expect(result.exitCode).not.toBe(0);
  expect(result.stderr).toContain("Docker CLI unavailable");
  expect(result.stdout).not.toContain(cloakbrowserSkipped);
  expect(result.trace).toBe("");
  expect(result.scraplingLinkExists).toBe(false);
});

test.each(["", "unknown-policy"])(
  "CloakBrowser verification rejects policy %s before the Docker API",
  (policy) => {
    const result = runDockerInstallTarget("cloakbrowser", "artifact-present", {
      action: "verify",
      policy,
    });

    expect(result.exitCode).not.toBe(0);
    expect(result.stdout).not.toContain(cloakbrowserVerified);
    expect(result.stderr).toContain("docker-install:");
    expect(result.trace).toBe("");
    expect(result.scraplingLinkExists).toBe(false);
  },
);

test.each([
  "",
  "--help",
  "image with spaces",
  `image@sha256:${"a".repeat(sha256HexadecimalLength - 1)}`,
])(
  "CloakBrowser verification rejects image %s before the Docker API",
  (imageOverride) => {
    const result = runDockerInstallTarget("cloakbrowser", "artifact-present", {
      action: "verify",
      imageOverride,
    });

    expect(result.exitCode).not.toBe(0);
    expect(result.stdout).not.toContain(cloakbrowserVerified);
    expect(result.stderr).toContain("docker-install:");
    expect(result.trace).toBe("");
    expect(result.scraplingLinkExists).toBe(false);
  },
);
