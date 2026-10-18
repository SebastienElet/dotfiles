import { z } from "zod";

const fixtureSchema = z
  .object({
    failures: z
      .record(z.string(), z.object({ status: z.number(), stderr: z.string() }))
      .optional(),
    localBranches: z.array(z.string()).optional(),
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
const commandNotFoundExitCode = 127;
const command = process.argv[commandArgumentIndex];
const commandArguments = process.argv.slice(commandOptionsIndex);
const failure = fixture.failures?.[`${command} ${commandArguments.join(" ")}`];

if (failure !== undefined) {
  process.stderr.write(failure.stderr);
  process.exit(failure.status);
}

if (command === "git") {
  const operation = commandArguments.findIndex((value) =>
    ["rev-parse", "show-ref"].includes(value),
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
}

process.exit(commandNotFoundExitCode);
