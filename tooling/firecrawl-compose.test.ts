import { expect, setDefaultTimeout, test } from "bun:test";
import { resolve } from "node:path";
import { z } from "zod";

const repositoryRoot = resolve(import.meta.dir, "..");
const composePath = resolve(repositoryRoot, "harness/firecrawl/compose.yml");
const composeTimeoutMilliseconds = 15_000;
const firecrawlPort = 3002;
setDefaultTimeout(composeTimeoutMilliseconds);
const renderedPortSchema = z
  .object({
    host_ip: z.string().optional(),
    mode: z.literal("ingress"),
    protocol: z.literal("tcp"),
    published: z.string(),
    target: z.number(),
  })
  .strict();
const loopbackFirecrawlPortSchema = renderedPortSchema.extend({
  host_ip: z.literal("127.0.0.1"),
  published: z.literal("3002"),
  target: z.literal(firecrawlPort),
});
const renderedComposeSchema = z.object({
  networks: z.object({
    backend: z
      .object({
        driver: z.literal("bridge"),
        external: z.literal(false).optional(),
      })
      .loose(),
  }),
  services: z.object({
    api: z.object({
      environment: z
        .object({
          HOST: z.literal("0.0.0.0"),
          USE_DB_AUTHENTICATION: z.literal("false"),
        })
        .loose(),
      networks: z.object({ backend: z.null() }).loose(),
      ports: z.array(renderedPortSchema),
    }),
  }),
});

function renderComposeConfiguration(): z.infer<typeof renderedComposeSchema> {
  const result = Bun.spawnSync(
    ["docker", "compose", "-f", composePath, "config", "--format", "json"],
    { cwd: repositoryRoot, stderr: "pipe", stdout: "pipe" },
  );

  expect(result.exitCode, result.stderr.toString()).toBe(0);
  return renderedComposeSchema.parse(JSON.parse(result.stdout.toString()));
}

test("the unauthenticated Firecrawl API is published only on host loopback", () => {
  const configuration = renderComposeConfiguration();
  const publishedApiPorts = configuration.services.api.ports.filter(
    ({ target }) => target === firecrawlPort,
  );

  expect(publishedApiPorts).toHaveLength(1);
  expect(
    loopbackFirecrawlPortSchema.safeParse(publishedApiPorts[0]).success,
  ).toBeTrue();
});

test.each([
  ["absent", {}],
  ["empty", { host_ip: "" }],
  ["IPv4 wildcard", { host_ip: "0.0.0.0" }],
  ["IPv6 wildcard", { host_ip: "::" }],
] as const)(
  "rejects an absent or wildcard host address: %s",
  (_scenario, hostAddress) => {
    const renderedPort = {
      ...hostAddress,
      mode: "ingress",
      protocol: "tcp",
      published: "3002",
      target: firecrawlPort,
    };

    expect(
      loopbackFirecrawlPortSchema.safeParse(renderedPort).success,
    ).toBeFalse();
  },
);

test("rejects host networking that bypasses published port confinement", () => {
  const configuration = renderComposeConfiguration();
  const hostNetworkConfiguration = {
    ...configuration,
    networks: {
      backend: { driver: "bridge", external: true, name: "host" },
    },
  };

  expect(
    renderedComposeSchema.safeParse(hostNetworkConfiguration).success,
  ).toBeFalse();
});
