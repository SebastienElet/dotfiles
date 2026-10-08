import { expect, test } from "bun:test";
import { createHerdrCommand } from "./herdr-command.ts";

const environment = {
  ...process.env,
  HERDR_BIN_PATH: process.execPath,
  HERDR_ENV: "1",
  HERDR_SOCKET_PATH: "/fixture/socket",
};

test("accepts a native metadata command with successful silent output", async () => {
  const run = createHerdrCommand(environment);

  expect(
    await run(["--eval", "process.exitCode = 0;"], { output: "silent" }),
  ).toBeUndefined();
});

test("passes metacharacters and newlines as one subprocess argument", async () => {
  const run = createHerdrCommand(environment);
  const payload =
    "URL/title data ' $(printf forbidden) `forbidden`\nsecond line";
  const result = await run([
    "--eval",
    "process.stdout.write(JSON.stringify({result:{value:process.argv.slice(1)}}));",
    payload,
  ]);
  expect(result).toEqual({ value: [payload] });
});

test("preserves a native refusal code and message", async () => {
  const run = createHerdrCommand(environment);
  const result = await run([
    "--eval",
    'process.stderr.write(JSON.stringify({error:{code:"agent_not_ready",message:"Trust is blocking startup"}})); process.exitCode=1;',
  ]).catch((error: unknown) => error);
  expect(result).toMatchObject({
    code: "agent_not_ready",
    message: "Trust is blocking startup",
  });
});

test("rejects malformed success output and empty JSON command output", async () => {
  const run = createHerdrCommand(environment);
  const results = await Promise.allSettled([
    run(["--eval", 'process.stdout.write("not-json");']),
    run(["--eval", "process.exitCode=0;"]),
  ]);
  expect(
    results.map(
      ({ status }: Readonly<Pick<PromiseSettledResult<unknown>, "status">>) =>
        status,
    ),
  ).toEqual(["rejected", "rejected"]);
});

test("requires inherited native session configuration before spawning", () => {
  expect(() =>
    createHerdrCommand({ ...environment, HERDR_ENV: "0" }),
  ).toThrow();
  expect(() =>
    createHerdrCommand({ ...environment, HERDR_SOCKET_PATH: "relative" }),
  ).toThrow();
});
