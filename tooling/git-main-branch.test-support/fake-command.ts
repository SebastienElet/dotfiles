import { z } from "zod";

const fixtureSchema = z
  .object({
    contexts: z.unknown().optional(),
    failures: z
      .record(z.string(), z.object({ status: z.number(), stderr: z.string() }))
      .optional(),
    localBranches: z.array(z.string()).optional(),
    remotes: z
      .record(
        z.string(),
        z.object({
          head: z.string().optional(),
          headOutput: z.string().optional(),
          urls: z.array(z.string()),
        }),
      )
      .optional(),
    repositories: z.record(z.string(), z.unknown()).optional(),
    repository: z.boolean().optional(),
    showRefStatus: z.number().optional(),
    showRefStderr: z.string().optional(),
  })
  .strict();

const fixture = fixtureSchema.parse(
  await Bun.file(process.env.FIXTURE_PATH ?? "").json(),
);
const commandArgumentIndex = 2;
const commandOptionsIndex = 3;
const repositoryFailureExitCode = 128;
const missingRemoteExitCode = 2;
const invalidContextExitCode = 3;
const commandNotFoundExitCode = 127;
const penultimateArgumentOffset = -2;
const command = process.argv[commandArgumentIndex];
const commandArguments = process.argv.slice(commandOptionsIndex);
const failure = fixture.failures?.[`${command} ${commandArguments.join(" ")}`];

if (failure !== undefined) {
  process.stderr.write(failure.stderr);
  process.exit(failure.status);
}

if (command === "git") {
  const operation = commandArguments.findIndex((value) =>
    [
      "rev-parse",
      "show-ref",
      "remote",
      "ls-remote",
      "check-ref-format",
    ].includes(value),
  );
  const tail = commandArguments.slice(operation);
  if (tail[0] === "rev-parse") {
    process.exit(fixture.repository === false ? repositoryFailureExitCode : 0);
  }
  if (tail[0] === "show-ref") {
    if (fixture.showRefStatus !== undefined) {
      if (fixture.showRefStderr !== undefined) {
        process.stderr.write(fixture.showRefStderr);
      }
      process.exit(fixture.showRefStatus);
    }
    const branch = tail.at(-1)?.replace("refs/heads/", "");
    process.exit(
      branch !== undefined && fixture.localBranches?.includes(branch) === true
        ? 0
        : 1,
    );
  }
  if (tail[0] === "remote" && tail.length === 1) {
    process.stdout.write(`${Object.keys(fixture.remotes ?? {}).join("\n")}\n`);
    process.exit(0);
  }
  if (tail[0] === "remote" && tail[1] === "get-url") {
    const remote = fixture.remotes?.[tail.at(-1) ?? ""];
    if (!remote) {
      process.exit(missingRemoteExitCode);
    }
    process.stdout.write(`${remote.urls.join("\n")}\n`);
    process.exit(0);
  }
  if (tail[0] === "ls-remote") {
    const remote = fixture.remotes?.[tail.at(penultimateArgumentOffset) ?? ""];
    if (!remote) {
      process.exit(missingRemoteExitCode);
    }
    process.stdout.write(
      remote.headOutput ??
        (remote.head !== undefined && remote.head !== ""
          ? `ref: refs/heads/${remote.head}\tHEAD\nabc\tHEAD\n`
          : ""),
    );
    process.exit(0);
  }
  if (tail[0] === "check-ref-format") {
    const branch = tail.at(-1);
    process.exit(
      branch !== undefined &&
        branch !== "" &&
        !branch.startsWith("-") &&
        !branch.includes("..")
        ? 0
        : 1,
    );
  }
}

if (command === "bkt" && commandArguments.includes("context")) {
  process.stdout.write(
    JSON.stringify(
      fixture.contexts ?? {
        contexts: [{ host: "api.bitbucket.org", name: "cloud" }],
      },
    ),
  );
  process.exit(0);
}

if (command === "bkt" && commandArguments.includes("api")) {
  if (commandArguments[0] !== "--context" || commandArguments[1] !== "cloud") {
    process.exit(invalidContextExitCode);
  }
  const identity = commandArguments
    .find((value) => value.startsWith("/repositories/"))
    ?.replace("/repositories/", "");
  const response = fixture.repositories?.[identity ?? ""];
  if (response === undefined) {
    process.exit(missingRemoteExitCode);
  }
  process.stdout.write(JSON.stringify(response));
  process.exit(0);
}

process.exit(commandNotFoundExitCode);
